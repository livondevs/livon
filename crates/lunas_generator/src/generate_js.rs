use lunas_parser::{DetailedBlock, DetailedMetaData, PropsInput, UseComponentStatement};

use crate::{
    consts::ROUTER_VIEW,
    generate_statements::{
        gen_create_anchors::gen_create_anchor_statements,
        gen_create_event_listener::generate_create_event_listener,
        gen_create_fragments::gen_create_fragments,
        gen_custom_component::gen_render_custom_component_statements,
        gen_for_blk::gen_render_for_blk_func, gen_if_blk::gen_render_if_blk_func,
        gen_reference_getter::gen_reference_getter, utils::create_indent,
    },
    orig_html_struct::structs::{Node, NodeContent},
    structs::{
        ctx::ContextCategories,
        transform_info::{sort_if_blocks, TextNodeRendererGroup, VariableNameAndAssignedNumber},
        transform_targets::{sort_elm_and_reactive_info, NodeAndReactiveInfo},
    },
    transformers::{
        html_utils::{check_html_elms, create_lunas_internal_component_statement},
        imports::generate_import_string,
        inputs::generate_input_variable_decl,
        js_utils::{analyze_js, load_lunas_script_variables},
        router::generate_router_initialization_code,
    },
};

pub fn generate_js_from_blocks(
    blocks: &DetailedBlock,
    engine_path: Option<String>,
) -> Result<(String, Option<String>), String> {
    let use_component_statements = blocks
        .detailed_meta_data
        .iter()
        .filter_map(|meta_data| match meta_data {
            DetailedMetaData::UseComponentStatement(use_component) => Some(use_component),
            _ => None,
        })
        .collect::<Vec<&UseComponentStatement>>();
    let inputs = blocks
        .detailed_meta_data
        .iter()
        .filter_map(|meta_data| match meta_data {
            DetailedMetaData::PropsInput(use_component) => Some(use_component),
            _ => None,
        })
        .collect::<Vec<&PropsInput>>();

    let mut component_names = use_component_statements
        .iter()
        .map(|use_component| use_component.component_name.clone())
        .collect::<Vec<String>>();

    let mut imports = vec![];

    #[cfg(not(feature = "playground"))]
    {
        imports.push("import { $$lunasRouter } from \"lunas/router\";".to_string());
    }

    let using_auto_routing = blocks
        .detailed_meta_data
        .iter()
        .any(|meta_data| match meta_data {
            DetailedMetaData::UseAutoRoutingStatement => true,
            _ => false,
        });

    if using_auto_routing {
        imports.push(
            "import { routes as $$lunasGeneratedRoutes } from \"virtual:generated-routes\";"
                .to_string(),
        );
        component_names.push(ROUTER_VIEW.to_string());
    }

    // TODO: add manual routing
    // let using_routing = blocks
    //     .detailed_meta_data
    //     .iter()
    //     .any(|meta_data| match meta_data {
    //         DetailedMetaData::UseRoutingStatement => true,
    //         _ => false,
    //     });

    let engine_path = match engine_path.is_none() {
        true => "lunas/engine".to_string(),
        false => engine_path.unwrap(),
    };

    let mut variables = vec![];

    let props_assignment = generate_input_variable_decl(&inputs, &mut variables);

    let mut codes = vec![];

    let (imports_in_script, (js_output, js_output_tail), js_func_deps, lun_imports) =
        analyze_js(blocks, inputs.len() as u32, &mut variables);

    codes.push(js_output);

    if lun_imports.len() > 0 {
        codes.push(load_lunas_script_variables(&lun_imports));
    }

    if js_output_tail.len() > 0 {
        codes.push(js_output_tail);
    }

    imports.extend(imports_in_script.clone());
    for use_component in use_component_statements {
        imports.push(format!(
            "import {} from \"{}\";",
            use_component.component_name, use_component.component_path
        ));
    }

    // Clone HTML as mutable reference
    let mut ref_map = vec![];

    let mut elm_and_var_relation = vec![];
    let mut action_and_target = vec![];
    let mut if_blocks_info = vec![];
    let mut for_blocks_info = vec![];
    let mut custom_component_blocks_info = vec![];
    let mut text_node_renderer = vec![];
    let mut ctx_cats = ContextCategories {
        if_ctx: vec![],
        for_ctx: vec![],
    };

    let mut ref_node_ids = vec![];
    let mut new_node = Node::new_from_dom(&blocks.detailed_language_blocks.dom)?;

    let variable_names = &variables
        .iter()
        .map(|v| v.name.clone())
        .collect::<Vec<String>>();

    // Analyze HTML
    check_html_elms(
        variable_names,
        &component_names,
        &js_func_deps,
        &mut new_node,
        &mut ref_map,
        &mut elm_and_var_relation,
        &mut action_and_target,
        None, // parent_uuid
        &mut vec![],
        &mut if_blocks_info,
        &mut for_blocks_info,
        &mut custom_component_blocks_info,
        &mut text_node_renderer,
        &mut ctx_cats,
        &vec![],  // ctx
        &vec![0], // ctx_num
        1,        // ctx_num_index
        false,    // is_root
    )?;

    sort_if_blocks(&mut if_blocks_info);
    sort_elm_and_reactive_info(&mut elm_and_var_relation);

    // TODO: reconsider about this unwrap
    let new_elm = match new_node.content {
        NodeContent::Element(elm) => elm,
        _ => panic!(),
    };

    ref_map.sort_by(|a, b| a.elm_loc().cmp(b.elm_loc()));

    // Generate JavaScript
    let html_insert = format!(
        "{};",
        create_lunas_internal_component_statement(&new_elm, "$$lunasSetComponentElement")
    );
    codes.push(html_insert);

    match props_assignment.is_some() {
        true => codes.insert(0, props_assignment.unwrap()),
        false => {}
    }

    let text_node_renderer_group = TextNodeRendererGroup::new(
        &if_blocks_info,
        &for_blocks_info,
        &text_node_renderer,
        &custom_component_blocks_info,
    );

    // Generate AfterMount
    let mut after_mount_code_array = vec![];
    let ref_getter_expression = gen_reference_getter(&ref_map, &None, &mut ref_node_ids, false);
    if let Some(ref_getter_expression) = ref_getter_expression {
        after_mount_code_array.push(ref_getter_expression);
    }
    let create_anchor_statements =
        gen_create_anchor_statements(&text_node_renderer_group, &vec![], &mut ref_node_ids, false);
    if let Some(create_anchor_statements) = create_anchor_statements {
        after_mount_code_array.push(create_anchor_statements);
    }
    let event_listener_code =
        generate_create_event_listener(&action_and_target, &vec![], &ref_node_ids, false);

    if let Some(code) = event_listener_code {
        after_mount_code_array.push(code);
    }

    let fragments = create_fragments_func(&elm_and_var_relation, &variables, &ref_node_ids, false);

    if let Some(fragments) = fragments {
        after_mount_code_array.push(fragments);
    }

    let render_if = gen_render_if_blk_func(
        &if_blocks_info,
        &ref_map,
        &action_and_target,
        &text_node_renderer_group,
        &custom_component_blocks_info,
        &variable_names,
        &variables,
        &elm_and_var_relation,
        &mut ref_node_ids,
        &ctx_cats,
        None,
        false,
    )?;
    let (render_for, _) = gen_render_for_blk_func(
        &for_blocks_info,
        &ref_map,
        &action_and_target,
        &text_node_renderer_group,
        &custom_component_blocks_info,
        &variable_names,
        &variables,
        &elm_and_var_relation,
        &mut ref_node_ids,
        &ctx_cats,
        &if_blocks_info,
        &for_blocks_info,
        None,
        false,
    )?;
    after_mount_code_array.extend(render_if);
    after_mount_code_array.extend(render_for);
    let render_component = gen_render_custom_component_statements(
        &custom_component_blocks_info,
        &vec![],
        &variable_names,
        &mut ref_node_ids,
        false,
    )?;
    if using_auto_routing {
        after_mount_code_array.push(generate_router_initialization_code(
            &custom_component_blocks_info,
            &ref_node_ids,
        )?);
    }
    after_mount_code_array.extend(render_component);
    let after_mount_code = after_mount_code_array
        .iter()
        .map(|c| create_indent(c))
        .collect::<Vec<String>>()
        .join("\n");
    let after_mount_func_code = format!(
        r#"$$lunasApplyEnhancement(function () {{
{}
}});
"#,
        after_mount_code
    );
    codes.push(after_mount_func_code);

    codes.push("return $$lunasComponentReturn;".to_string());

    let full_js_code = gen_full_code(engine_path, imports, codes, inputs);
    let css_code = blocks.detailed_language_blocks.css.clone();

    Ok((full_js_code, css_code))
}

fn gen_full_code(
    engine_path: String,
    imports_string: Vec<String>,
    codes: Vec<String>,
    inputs: Vec<&PropsInput>,
) -> String {
    let imports_string = generate_import_string(&imports_string);
    let arg_names_array = match inputs.len() == 0 {
        true => "".to_string(),
        false => {
            let arr = inputs
                .iter()
                .map(|i| format!("\"{}\"", i.variable_name.clone()))
                .collect::<Vec<String>>();
            format!(", [{}]", arr.join(", "))
        }
    };

    // codesにcreate_indentを適用して、\nでjoinする -> code
    let code = codes
        .iter()
        .map(|c| create_indent(c))
        .collect::<Vec<String>>()
        .join("\n");
    format!(
        r#"import {{ $$lunasEscapeHtml, $$lunasInitComponent, $$lunasReplaceText, $$lunasReplaceAttr, $$createLunasElement, $$lunasCreateNonReactive }} from "{}";{}

export default function(args = {{}}) {{
    const {{ $$lunasGetElm, $$lunasSetImportVars, $$lunasSetComponentElement, $$lunasComponentReturn, $$lunasAfterMount, $$lunasAfterUnmount, $$lunasApplyEnhancement, $$lunasReactive, $$lunasCreateIfBlock, $$lunasCreateForBlock, $$lunasInsertEmpty, $$lunasGetElmRefs, $$lunasAddEvListener, $$lunasInsertTextNodes, $$lunasCreateFragments, $$lunasInsertComponent, $$lunasMountComponent, $$lunasWatch }} = new $$lunasInitComponent(args{});
{}
}}
"#,
        engine_path, imports_string, arg_names_array, code,
    )
}

pub fn create_fragments_func(
    elm_and_variable_relations: &Vec<NodeAndReactiveInfo>,
    variable_name_and_assigned_numbers: &Vec<VariableNameAndAssignedNumber>,
    ref_node_ids: &Vec<String>,
    under_for: bool,
) -> Option<String> {
    let fragments_str = gen_create_fragments(
        elm_and_variable_relations,
        variable_name_and_assigned_numbers,
        &ref_node_ids,
        &vec![],
        under_for,
        &None,
    );

    if fragments_str.is_none() {
        return None;
    }

    Some(format!(
        r#"
$$lunasCreateFragments({});"#,
        fragments_str.unwrap()
    ))
}

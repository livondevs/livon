use lunas_parser::{DetailedBlock, DetailedMetaData, PropsInput, UseComponentStatement};
use std::collections::HashSet;

use crate::{
    consts::ROUTER_VIEW,
    generate_statements::{
        gen_if_blk::gen_render_if_blk_func,
        utils::{create_indent, gen_binary_map_from_bool},
    },
    orig_html_struct::structs::{Node, NodeContent},
    structs::{
        transform_info::{
            sort_if_blocks, ActionAndTarget, CustomComponentBlockInfo, IfBlockInfo, NeededIdName,
            TextNodeRendererGroup, VariableNameAndAssignedNumber,
        },
        transform_targets::{sort_elm_and_reactive_info, NodeAndReactiveInfo},
    },
    transformers::{
        html_utils::{check_html_elms, create_lunas_internal_component_statement},
        imports::generate_import_string,
        inputs::generate_input_variable_decl,
        js_utils::analyze_js,
        router::generate_router_initialization_code,
    },
};

pub fn generate_js_from_blocks(
    blocks: &DetailedBlock,
    runtime_path: Option<String>,
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
        imports.push("import { $$lunasRouter } from \"lunas/dist/runtime/router\";".to_string());
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

    let runtime_path = match runtime_path.is_none() {
        true => "lunas/dist/runtime".to_string(),
        false => runtime_path.unwrap(),
    };

    let mut variables = vec![];

    let props_assignment = generate_input_variable_decl(&inputs, &mut variables);

    let (variable_names, imports_in_script, js_output) =
        analyze_js(blocks, inputs.len() as u32, &mut variables);

    let mut codes = vec![js_output];

    imports.extend(imports_in_script.clone());
    for use_component in use_component_statements {
        imports.push(format!(
            "import {} from \"{}\";",
            use_component.component_name, use_component.component_path
        ));
    }

    // Clone HTML as mutable reference
    let mut needed_id = vec![];

    let mut elm_and_var_relation = vec![];
    let mut action_and_target = vec![];
    let mut if_blocks_info = vec![];
    let mut custom_component_blocks_info = vec![];
    let mut text_node_renderer = vec![];

    let mut new_node = Node::new_from_dom(&blocks.detailed_language_blocks.dom)?;

    // Analyze HTML
    check_html_elms(
        &variable_names,
        &component_names,
        &mut new_node,
        &mut needed_id,
        &mut elm_and_var_relation,
        &mut action_and_target,
        None,
        &mut vec![],
        &mut if_blocks_info,
        &mut custom_component_blocks_info,
        &mut text_node_renderer,
        &vec![],
        &vec![0],
        1,
        false,
    )?;

    sort_if_blocks(&mut if_blocks_info);
    sort_elm_and_reactive_info(&mut elm_and_var_relation);

    // TODO: reconsider about this unwrap
    let new_elm = match new_node.content {
        NodeContent::Element(elm) => elm,
        _ => panic!(),
    };

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
        &text_node_renderer,
        &custom_component_blocks_info,
    );

    // Generate AfterMount
    let mut after_mount_code_array = vec![];
    let ref_getter_expression = gen_ref_getter_from_needed_ids(&needed_id, &None, &None);
    after_mount_code_array.push(ref_getter_expression);
    let if_block_elm_decl =
        generate_if_block_ref_var_decl(&if_blocks_info, &needed_id, &text_node_renderer_group);
    after_mount_code_array.extend(if_block_elm_decl);
    let create_anchor_statements = gen_create_anchor_statements(&text_node_renderer_group, &vec![]);
    after_mount_code_array.extend(create_anchor_statements);
    let event_listener_codes = create_event_listener(&action_and_target, &vec![]);
    after_mount_code_array.extend(event_listener_codes);
    let render_if = gen_render_if_blk_func(
        &if_blocks_info,
        &needed_id,
        &action_and_target,
        &text_node_renderer_group,
        &custom_component_blocks_info,
        &variable_names,
    );
    after_mount_code_array.extend(render_if);
    let render_component = gen_render_custom_component_statements(
        &custom_component_blocks_info,
        &vec![],
        &variable_names,
    );
    if using_auto_routing {
        after_mount_code_array.push(generate_router_initialization_code(
            custom_component_blocks_info,
        )?);
    }
    after_mount_code_array.extend(render_component);
    after_mount_code_array.push("this.blkUpdateMap = 0".to_string());
    let update_func_code = gen_on_update_func(elm_and_var_relation, variables, if_blocks_info);
    after_mount_code_array.push(update_func_code);
    let after_mount_code = after_mount_code_array
        .iter()
        .map(|c| create_indent(c))
        .collect::<Vec<String>>()
        .join("\n");
    let after_mount_func_code = format!(
        r#"$$lunasAfterMount(function () {{
{}
}});
"#,
        after_mount_code
    );
    codes.push(after_mount_func_code);

    codes.push("return $$lunasComponentReturn;".to_string());

    let full_js_code = gen_full_code(runtime_path, imports, codes, inputs);
    let css_code = blocks.detailed_language_blocks.css.clone();

    Ok((full_js_code, css_code))
}

fn gen_full_code(
    runtime_path: String,
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
        r#"import {{ $$lunasAddEvListener, $$lunasEscapeHtml, $$lunasGetElmRefs, $$lunasInitComponent, $$lunasReplaceInnerHtml, $$lunasReplaceText, $$lunasReplaceAttr, $$lunasInsertEmpty, $$lunasInsertContent, $$createLunasElement, $$lunasCreateNonReactive, $$lunasShouldRender }} from "{}";{}

export default function(args = {{}}) {{
    const {{ $$lunasSetComponentElement, $$lunasUpdateComponent, $$lunasComponentReturn, $$lunasAfterMount, $$lunasReactive, $$lunasRenderIfBlock, $$lunasCreateIfBlock }} = new $$lunasInitComponent(args{});
{}
}}"#,
        runtime_path, imports_string, arg_names_array, code,
    )
}

pub fn gen_ref_getter_from_needed_ids(
    needed_ids: &Vec<NeededIdName>,
    if_blk: &Option<&IfBlockInfo>,
    ctx: &Option<&Vec<String>>,
) -> String {
    let needed_ids_to_get_here = needed_ids
        .iter()
        .filter(|needed_elm: &&NeededIdName| match *if_blk == None {
            true => needed_elm.ctx.len() == 0,
            false => &needed_elm.ctx == ctx.unwrap(),
        })
        // As of now, we get ref of if block on the first render of the block
        // in future, we will store ref to if blk on generation
        // // do not get the Ref of the IF block itself
        // // .filter(|needed_elm: &&NeededIdName| match *if_blk == None {
        // //     true => true,
        // //     false => needed_elm.node_id != if_blk.unwrap().if_block_id,
        // // })
        .collect::<Vec<&NeededIdName>>();

    // TODO:format!などを使ってもっとみやすいコードを書く
    let mut ref_getter_str = match if_blk == &None {
        true => "const ".to_string(),
        false => "".to_string(),
    };

    ref_getter_str.push_str("[");

    ref_getter_str.push_str(
        needed_ids_to_get_here
            .iter()
            .map(|id| format!("$$lunas{}Ref", id.node_id))
            .collect::<Vec<String>>()
            .join(", ")
            .as_str(),
    );
    ref_getter_str.push_str("] = $$lunasGetElmRefs([");
    ref_getter_str.push_str(
        needed_ids_to_get_here
            .iter()
            .map(|id| format!("\"{}\"", id.id_name))
            .collect::<Vec<String>>()
            .join(", ")
            .as_str(),
    );
    let delete_id_bool_map = needed_ids_to_get_here
        .iter()
        .map(|id| id.to_delete)
        .collect::<Vec<bool>>();
    let delete_id_map = gen_binary_map_from_bool(delete_id_bool_map);
    ref_getter_str.push_str(format!("], {map});", map = delete_id_map).as_str());
    ref_getter_str
}

pub fn create_event_listener(
    actions_and_targets: &Vec<ActionAndTarget>,
    current_ctx: &Vec<String>,
) -> Vec<String> {
    let mut result = vec![];
    for action_and_target in actions_and_targets {
        if action_and_target.ctx != *current_ctx {
            continue;
        }
        result.push(format!(
            "$$lunasAddEvListener($$lunas{}Ref, \"{}\", {});",
            action_and_target.target,
            action_and_target.action_name,
            action_and_target.action.to_string()
        ));
    }
    result
}

fn generate_if_block_ref_var_decl(
    if_blocks_info: &Vec<IfBlockInfo>,
    needed_id: &Vec<NeededIdName>,
    text_node_renderer_group: &TextNodeRendererGroup,
) -> Vec<String> {
    let mut codes = vec![];
    if if_blocks_info.len() > 0 {
        let mut variables_to_declare = HashSet::new();
        for if_block_info in if_blocks_info.iter() {
            variables_to_declare.insert(format!("$$lunas{}Ref", if_block_info.if_blk_id));
        }

        for needed_id in needed_id.iter() {
            if needed_id.ctx.len() != 0 {
                variables_to_declare.insert(format!("$$lunas{}Ref", needed_id.node_id.clone()));
            }
        }

        for text_node_renderer in text_node_renderer_group.renderers.iter() {
            match text_node_renderer {
                crate::structs::transform_info::TextNodeRenderer::ManualRenderer(txt_renderer) => {
                    if txt_renderer.ctx.len() != 0 {
                        variables_to_declare
                            .insert(format!("$$lunas{}Text", txt_renderer.text_node_id.clone()));
                    }
                }
                crate::structs::transform_info::TextNodeRenderer::IfBlockRenderer(if_renderer) => {
                    if if_renderer.ctx_over_if.len() != 0 {
                        variables_to_declare
                            .insert(format!("$$lunas{}Anchor", if_renderer.if_blk_id.clone()));
                    }
                }
                crate::structs::transform_info::TextNodeRenderer::CustomComponentRenderer(
                    custom_renderer,
                ) => {
                    if custom_renderer.ctx.len() != 0 {
                        variables_to_declare.insert(format!(
                            "$$lunas{}Anchor",
                            custom_renderer.custom_component_block_id.clone()
                        ));
                    }
                }
            }
        }

        if variables_to_declare.len() != 0 {
            let decl = format!("let {};", itertools::join(variables_to_declare, ", "));
            codes.push(decl);
        }
    }
    return codes;
}

fn gen_on_update_func(
    elm_and_variable_relations: Vec<NodeAndReactiveInfo>,
    variable_name_and_assigned_numbers: Vec<VariableNameAndAssignedNumber>,
    if_blocks_infos: Vec<IfBlockInfo>,
) -> String {
    let mut replace_statements = vec![];

    for (index, if_block_info) in if_blocks_infos.iter().enumerate() {
        let if_blk_rendering_cond = if if_block_info.ctx_over_if.len() != 0 {
            format!(
                "(!((this.blkRenderedMap & {0}) ^ {0})) && ",
                if_block_info.generate_ctx_num(&if_blocks_infos)
            )
        } else {
            "".to_string()
        };

        let dep_vars = &if_block_info.condition_dep_vars;

        // TODO: データバインディングと同じコードを使っているので共通化する
        let dep_vars_assined_numbers = variable_name_and_assigned_numbers
            .iter()
            .filter(|v| {
                dep_vars
                    .iter()
                    .map(|d| *d == v.name)
                    .collect::<Vec<bool>>()
                    .contains(&true)
            })
            .map(|v| v.assignment)
            .collect::<Vec<u32>>();

        let combined_number = get_combined_binary_number(dep_vars_assined_numbers);

        let render_check = format!(
            "$$lunasShouldRender({}, this.blkRenderedMap, {})",
            if_block_info.condition,
            index + 1,
        );

        replace_statements.push(format!(
            "{}this.valUpdateMap & {} && {} && ( {} ? {} : ({}, {}, {}) );",
            if_blk_rendering_cond,
            combined_number,
            render_check,
            if_block_info.condition,
            format!("$$lunasRenderIfBlock(\"{}\")", &if_block_info.if_blk_id),
            format!("$$lunas{}Ref.remove()", &if_block_info.if_blk_id),
            format!("$$lunas{}Ref = null", &if_block_info.if_blk_id),
            format!("this.blkRenderedMap ^= {}", index + 1),
        ));
    }

    for elm_and_variable_relation in elm_and_variable_relations {
        match elm_and_variable_relation {
            NodeAndReactiveInfo::ElmAndReactiveAttributeRelation(elm_and_attr_relation) => {
                let _elm_and_attr_relation = elm_and_attr_relation.clone();
                for c in elm_and_attr_relation.reactive_attr {
                    let dep_vars_assined_numbers = variable_name_and_assigned_numbers
                        .iter()
                        .filter(|v| {
                            c.variable_names
                                .iter()
                                .map(|d| *d == v.name)
                                .collect::<Vec<bool>>()
                                .contains(&true)
                        })
                        .map(|v| v.assignment)
                        .collect::<Vec<u32>>();

                    let if_blk_rendering_cond = if elm_and_attr_relation.ctx.len() != 0 {
                        format!(
                            "(!((this.blkRenderedMap & {0}) ^ {0})) && ",
                            _elm_and_attr_relation.generate_ctx_num(&if_blocks_infos)
                        )
                    } else {
                        "".to_string()
                    };

                    replace_statements.push(format!(
                        "{}this.valUpdateMap & {:?} && $$lunasReplaceAttr(\"{}\", {}, $$lunas{}Ref);",
                        if_blk_rendering_cond,
                        get_combined_binary_number(dep_vars_assined_numbers),
                        c.attribute_key,
                        c.content_of_attr,
                        elm_and_attr_relation.elm_id
                    ));
                }
            }
            NodeAndReactiveInfo::ElmAndVariableRelation(elm_and_variable_relation) => {
                let depending_variables = elm_and_variable_relation.dep_vars.clone();
                let target_id = elm_and_variable_relation.elm_id.clone();

                let dep_vars_assined_numbers = variable_name_and_assigned_numbers
                    .iter()
                    .filter(|v| {
                        depending_variables
                            .iter()
                            .map(|d| *d == v.name)
                            .collect::<Vec<bool>>()
                            .contains(&true)
                    })
                    .map(|v| v.assignment)
                    .collect::<Vec<u32>>();
                let under_if_blk = elm_and_variable_relation.ctx.len() != 0;

                let if_blk_rendering_cond = if under_if_blk {
                    format!(
                        "(!((this.blkRenderedMap & {0}) ^ {0})) && ",
                        elm_and_variable_relation.generate_ctx_num(&if_blocks_infos)
                    )
                } else {
                    "".to_string()
                };

                let combined_number = get_combined_binary_number(dep_vars_assined_numbers);

                let to_update_cond = if under_if_blk {
                    format!(
                        "(this.valUpdateMap & {:?} && ((this.blkUpdateMap & {1}) ^ {1}) )",
                        combined_number,
                        elm_and_variable_relation.generate_ctx_num(&if_blocks_infos)
                    )
                } else {
                    format!("this.valUpdateMap & {:?}", combined_number)
                };

                replace_statements.push(format!(
                    "{}{} && $$lunasReplaceText(`{}`, $$lunas{}Ref);",
                    if_blk_rendering_cond,
                    to_update_cond,
                    elm_and_variable_relation.content_of_element.trim(),
                    target_id
                ));
            }
            NodeAndReactiveInfo::TextAndVariableContentRelation(txt_and_var_content) => {
                // TODO: Elementとほとんど同じなので、共通化

                let depending_variables = txt_and_var_content.dep_vars.clone();
                let target_id = txt_and_var_content.text_node_id.clone();

                let dep_vars_assined_numbers = variable_name_and_assigned_numbers
                    .iter()
                    .filter(|v| {
                        depending_variables
                            .iter()
                            .map(|d| *d == v.name)
                            .collect::<Vec<bool>>()
                            .contains(&true)
                    })
                    .map(|v| v.assignment)
                    .collect::<Vec<u32>>();
                let under_if_blk = txt_and_var_content.ctx.len() != 0;

                let if_blk_rendering_cond = if under_if_blk {
                    format!(
                        "(!((this.blkRenderedMap & {0}) ^ {0})) && ",
                        txt_and_var_content.generate_ctx_num(&if_blocks_infos)
                    )
                } else {
                    "".to_string()
                };

                let combined_number = get_combined_binary_number(dep_vars_assined_numbers);

                let to_update_cond = if under_if_blk {
                    format!(
                        "(this.valUpdateMap & {:?} && ((this.blkUpdateMap & {1}) ^ {1}) )",
                        combined_number,
                        txt_and_var_content.generate_ctx_num(&if_blocks_infos)
                    )
                } else {
                    format!("this.valUpdateMap & {:?}", combined_number)
                };

                replace_statements.push(format!(
                    "{}{} && $$lunasReplaceText(`{}`, $$lunas{}Text);",
                    if_blk_rendering_cond,
                    to_update_cond,
                    txt_and_var_content.content_of_element.trim(),
                    target_id
                ));
            }
        }
    }

    let code = replace_statements
        .iter()
        .map(|c| create_indent(c))
        .collect::<Vec<String>>()
        .join("\n");

    let result = format!(
        r#"$$lunasUpdateComponent(function () {{
{code}
}});"#,
        code = code
    );

    result
}

pub fn gen_create_anchor_statements(
    text_node_renderer: &TextNodeRendererGroup,
    ctx_condition: &Vec<String>,
) -> Vec<String> {
    let mut create_anchor_statements = vec![];
    for render in &text_node_renderer.renderers {
        match render {
            crate::structs::transform_info::TextNodeRenderer::ManualRenderer(txt_renderer) => {
                if &txt_renderer.ctx != ctx_condition {
                    continue;
                }
                let anchor_id = match &txt_renderer.target_anchor_id {
                    Some(anchor_id) => format!("$$lunas{}Ref", anchor_id),
                    None => "null".to_string(),
                };
                let variable_declaration_word = match ctx_condition.len() != 0 {
                    // when under if block, we don't need to declare the variable
                    true => "",
                    false => "const ",
                };
                let create_anchor_statement = format!(
                    "{}$$lunas{}Text = $$lunasInsertContent(`{}`,$$lunas{}Ref,{});",
                    &variable_declaration_word,
                    &txt_renderer.text_node_id,
                    &txt_renderer.content.trim(),
                    &txt_renderer.parent_id,
                    anchor_id
                );
                create_anchor_statements.push(create_anchor_statement);
            }
            crate::structs::transform_info::TextNodeRenderer::IfBlockRenderer(if_block) => {
                match if_block.distance_to_next_elm > 1 {
                    true => {
                        if &if_block.ctx_over_if != ctx_condition {
                            continue;
                        }
                        let anchor_id = match &if_block.target_anchor_id {
                            Some(anchor_id) => format!("$$lunas{}Ref", anchor_id),
                            None => "null".to_string(),
                        };
                        let variable_declaration_word = match ctx_condition.len() != 0 {
                            // when under if block, we don't need to declare the variable
                            true => "",
                            false => "const ",
                        };
                        let create_anchor_statement = format!(
                            "{}$$lunas{}Anchor = $$lunasInsertEmpty($$lunas{}Ref,{});",
                            variable_declaration_word,
                            if_block.if_blk_id,
                            if_block.parent_id,
                            anchor_id
                        );
                        create_anchor_statements.push(create_anchor_statement);
                    }
                    false => {}
                }
            }
            crate::structs::transform_info::TextNodeRenderer::CustomComponentRenderer(
                custom_component,
            ) => match custom_component.distance_to_next_elm > 1 {
                true => {
                    if &custom_component.ctx != ctx_condition {
                        continue;
                    }
                    let anchor_id = match &custom_component.target_anchor_id {
                        Some(anchor_id) => format!("$$lunas{}Ref", anchor_id),
                        None => "null".to_string(),
                    };
                    let variable_declaration_word = match ctx_condition.len() != 0 {
                        // when under if block, we don't need to declare the variable
                        true => "",
                        false => "const ",
                    };
                    let create_anchor_statement = format!(
                        "{}$$lunas{}Anchor = $$lunasInsertEmpty($$lunas{}Ref,{});",
                        variable_declaration_word,
                        custom_component.custom_component_block_id,
                        custom_component.parent_id,
                        anchor_id
                    );
                    create_anchor_statements.push(create_anchor_statement);
                }
                false => {}
            },
        }
    }
    create_anchor_statements
}

pub fn gen_render_custom_component_statements(
    custom_component_block_info: &Vec<CustomComponentBlockInfo>,
    ctx: &Vec<String>,
    variable_names: &Vec<String>,
) -> Vec<String> {
    let mut render_custom_statements = vec![];

    for custom_component_block in custom_component_block_info.iter() {
        if custom_component_block.is_routing_component {
            continue;
        }
        if custom_component_block.ctx != *ctx {
            continue;
        }
        if custom_component_block.have_sibling_elm {
            match custom_component_block.distance_to_next_elm > 1 {
                true => {
                    render_custom_statements.push(format!(
                        "const $$lunas{}Comp = {}({}).insert($$lunas{}Ref, $$lunas{}Anchor);",
                        custom_component_block.custom_component_block_id,
                        custom_component_block.component_name,
                        custom_component_block.args.to_object(variable_names),
                        custom_component_block.parent_id,
                        custom_component_block.custom_component_block_id
                    ));
                }
                false => {
                    let anchor_ref_name = match &custom_component_block.target_anchor_id {
                        Some(anchor_id) => format!("$$lunas{}Ref", anchor_id),
                        None => "null".to_string(),
                    };
                    render_custom_statements.push(format!(
                        "const $$lunas{}Comp = {}({}).insert($$lunas{}Ref, {});",
                        custom_component_block.custom_component_block_id,
                        custom_component_block.component_name,
                        custom_component_block.args.to_object(variable_names),
                        custom_component_block.parent_id,
                        anchor_ref_name
                    ));
                }
            }
        } else {
            render_custom_statements.push(format!(
                "const $$lunas{}Comp = {}({}).mount($$lunas{}Ref);",
                custom_component_block.custom_component_block_id,
                custom_component_block.component_name,
                custom_component_block.args.to_object(variable_names),
                custom_component_block.parent_id
            ));
        }
    }
    render_custom_statements
}

/// Returns a binary number that is the result of ORing all the numbers in the argument.
/// ```
/// let numbers = vec![0b0001, 0b0010, 0b0100];
/// let result = get_combined_binary_number(numbers);
/// assert_eq!(result, 0b0111);
/// ```
fn get_combined_binary_number(numbers: Vec<u32>) -> u32 {
    let mut result = 0;
    for (_, &value) in numbers.iter().enumerate() {
        result |= value;
    }
    result
}

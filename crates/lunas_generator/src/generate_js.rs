use lunas_parser::{DetailedBlock, DetailedMetaData, PropsInput, UseComponentStatement};

use crate::{
    consts::ROUTER_VIEW,
    generate_statements::{
        gen_for_blk::gen_render_for_blk_func,
        gen_if_blk::gen_render_if_blk_func,
        utils::{create_indent, gen_binary_map_from_bool},
    },
    orig_html_struct::structs::{Node, NodeContent},
    structs::{
        ctx::ContextCategories,
        transform_info::{
            sort_if_blocks, ActionAndTarget, CustomComponentBlockInfo, IdBasedElementAccess,
            RefMap, TextNodeRendererGroup, VariableNameAndAssignedNumber,
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

    // Analyze HTML
    check_html_elms(
        &variable_names,
        &component_names,
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
    let ref_getter_expression =
        gen_ref_getter_from_needed_ids(&ref_map, &None, &mut ref_node_ids, &ctx_cats);
    if let Some(ref_getter_expression) = ref_getter_expression {
        after_mount_code_array.push(ref_getter_expression);
    }
    let create_anchor_statements =
        gen_create_anchor_statements(&text_node_renderer_group, &vec![], &mut ref_node_ids, false);
    if let Some(create_anchor_statements) = create_anchor_statements {
        after_mount_code_array.push(create_anchor_statements);
    }
    let event_listener_code =
        create_event_listener(&action_and_target, &vec![], &ref_node_ids, false);

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
    );
    let render_for = gen_render_for_blk_func(
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
    );
    after_mount_code_array.extend(render_if);
    after_mount_code_array.extend(render_for);
    let render_component = gen_render_custom_component_statements(
        &custom_component_blocks_info,
        &vec![],
        &variable_names,
        &mut ref_node_ids,
        false,
    );
    if using_auto_routing {
        after_mount_code_array.push(generate_router_initialization_code(
            &custom_component_blocks_info,
        )?);
    }
    after_mount_code_array.extend(render_component);
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
        r#"import {{ $$lunasEscapeHtml, $$lunasInitComponent, $$lunasReplaceText, $$lunasReplaceAttr, $$createLunasElement, $$lunasCreateNonReactive }} from "{}";{}

export default function(args = {{}}) {{
    const {{ $$lunasSetComponentElement, $$lunasComponentReturn, $$lunasAfterMount, $$lunasReactive, $$lunasCreateIfBlock, $$lunasCreateForBlock, $$lunasInsertEmpty, $$lunasGetElmRefs, $$lunasAddEvListener, $$lunasInsertTextNodes, $$lunasCreateFragments, $$lunasInsertComponent, $$lunasMountComponent }} = new $$lunasInitComponent(args{});
{}
}}
"#,
        runtime_path, imports_string, arg_names_array, code,
    )
}

pub fn gen_ref_getter_from_needed_ids(
    ref_maps: &Vec<RefMap>,
    ctx: &Option<&Vec<String>>,
    ref_node_ids: &mut Vec<String>,
    ctx_cats: &ContextCategories,
) -> Option<String> {
    let ref_node_ids_count = ref_node_ids.len();
    let ctx = match ctx.is_none() {
        true => vec![],
        false => ctx.unwrap().clone(), // Clone the Vec<String> to avoid borrowing issues
    };
    let refs_for_current_context = ref_maps
        .iter()
        .filter(|needed_elm| needed_elm.ctx() == &ctx)
        .collect::<Vec<&RefMap>>();

    for ref_obj in refs_for_current_context.iter() {
        match ref_obj {
            RefMap::NodeCreationMethod(node_creation_method) => {
                ref_node_ids.push(node_creation_method.node_id.clone())
            }
            RefMap::IdBasedElementAccess(id_based_element_access) => {
                ref_node_ids.push(id_based_element_access.node_id.clone())
            }
        }
    }

    // TODO: Use format! to improve code readability
    let node_creation_method_count = refs_for_current_context
        .iter()
        .filter(|id| match id {
            RefMap::NodeCreationMethod(_) => true,
            _ => false,
        })
        .count();

    let id_based_elements = refs_for_current_context
        .iter()
        .filter_map(|id| match id {
            RefMap::IdBasedElementAccess(id) => Some(id),
            _ => None,
        })
        .collect::<Vec<&IdBasedElementAccess>>();

    if id_based_elements.is_empty() {
        return None;
    }

    let mut ref_getter_str = String::from("$$lunasGetElmRefs([");
    ref_getter_str.push_str(
        &id_based_elements
            .iter()
            .map(|id| format!("\"{}\"", id.id_name))
            .collect::<Vec<String>>()
            .join(", "),
    );
    let delete_id_bool_map = id_based_elements
        .iter()
        .map(|id| id.to_delete)
        .collect::<Vec<bool>>();
    let delete_id_map = gen_binary_map_from_bool(delete_id_bool_map);
    // TODO: 超重要、2重forに対応する
    let is_under_for_clause = ctx_cats.is_under_for_clause(&ctx);
    let offset = if !is_under_for_clause {
        if ref_node_ids_count + node_creation_method_count == 0 {
            "".to_string()
        } else {
            format!(", {}", ref_node_ids_count + node_creation_method_count)
        }
    } else {
        format!(
            ", [{}, ...$$lunasForIndices]",
            ref_node_ids_count + node_creation_method_count
        )
    };
    ref_getter_str.push_str(&format!(
        "], {map}{offset});",
        map = delete_id_map,
        offset = offset.as_str()
    ));
    Some(ref_getter_str)
}

pub fn create_event_listener(
    actions_and_targets: &Vec<ActionAndTarget>,
    current_ctx: &Vec<String>,
    ref_node_ids: &Vec<String>,
    under_for: bool,
) -> Option<String> {
    let filtered_targets = actions_and_targets
        .iter()
        .filter(|action_and_target| action_and_target.ctx == *current_ctx)
        .collect::<Vec<&ActionAndTarget>>();

    if filtered_targets.is_empty() {
        return None;
    }
    let mut result = vec![];
    for (index, action_and_target) in filtered_targets.iter().enumerate() {
        let reference_node_idx = ref_node_ids
            .iter()
            .position(|id| id == &action_and_target.target)
            .unwrap();
        let reference_string = match under_for {
            true => format!("[{}, ...$$lunasForIndices]", reference_node_idx),
            false => reference_node_idx.to_string(),
        };
        result.push(format!(
            "[{}, \"{}\", {}]{}",
            reference_string,
            action_and_target.action_name,
            action_and_target.action.to_string(),
            if index != filtered_targets.len() - 1 {
                ","
            } else {
                ""
            }
        ));
    }
    let formatted_result = create_indent(result.join("\n").as_str());
    Some(format!(
        r#"$$lunasAddEvListener([
{}
]);"#,
        formatted_result
    ))
}

pub fn create_fragments_func(
    elm_and_variable_relations: &Vec<NodeAndReactiveInfo>,
    variable_name_and_assigned_numbers: &Vec<VariableNameAndAssignedNumber>,
    ref_node_ids: &Vec<String>,
    under_for: bool,
) -> Option<String> {
    let fragments_str = create_fragments(
        elm_and_variable_relations,
        variable_name_and_assigned_numbers,
        &ref_node_ids,
        &vec![],
        under_for,
    );

    if fragments_str.is_none() {
        return None;
    }

    Some(format!(
        r#"$$lunasCreateFragments([
{}
]);"#,
        create_indent(fragments_str.unwrap().as_str())
    ))
}

pub fn create_fragments(
    elm_and_variable_relations: &Vec<NodeAndReactiveInfo>,
    variable_name_and_assigned_numbers: &Vec<VariableNameAndAssignedNumber>,
    ref_node_ids: &Vec<String>,
    current_ctx: &Vec<String>,
    under_for: bool,
) -> Option<String> {
    let mut fragments = vec![];

    for elm_and_variable_relation in elm_and_variable_relations {
        let ctx = match elm_and_variable_relation {
            NodeAndReactiveInfo::ElmAndVariableRelation(elm_and_var) => elm_and_var.ctx.clone(),
            NodeAndReactiveInfo::ElmAndReactiveAttributeRelation(elm_and_reactive_attr) => {
                elm_and_reactive_attr.ctx.clone()
            }
            NodeAndReactiveInfo::TextAndVariableContentRelation(text_and_var) => {
                text_and_var.ctx.clone()
            }
        };
        if ctx != *current_ctx {
            continue;
        }
        match elm_and_variable_relation {
            NodeAndReactiveInfo::ElmAndReactiveAttributeRelation(elm_and_attr_relation) => {
                let _elm_and_attr_relation = elm_and_attr_relation.clone();
                for c in _elm_and_attr_relation.reactive_attr {
                    let dep_vars_assigned_numbers = variable_name_and_assigned_numbers
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

                    let target_node_idx = ref_node_ids
                        .iter()
                        .position(|id| id == &elm_and_attr_relation.elm_id)
                        .unwrap();

                    fragments.push(format!(
                        "[[() => {}, \"{}\"], {}, {}, {}]",
                        c.content_of_attr,
                        c.attribute_key,
                        target_node_idx,
                        get_combined_binary_number(dep_vars_assigned_numbers),
                        "0" // FragmentType.ATTRIBUTE
                    ));
                }
            }
            _ => {
                let (depending_variables, target_id, content) = match elm_and_variable_relation {
                    NodeAndReactiveInfo::TextAndVariableContentRelation(
                        text_and_variable_content_relation,
                    ) => (
                        text_and_variable_content_relation.dep_vars.clone(),
                        text_and_variable_content_relation.text_node_id.clone(),
                        text_and_variable_content_relation
                            .content_of_element
                            .clone(),
                    ),
                    NodeAndReactiveInfo::ElmAndVariableRelation(
                        elm_and_variable_content_relation,
                    ) => (
                        elm_and_variable_content_relation.dep_vars.clone(),
                        elm_and_variable_content_relation.elm_id.clone(),
                        elm_and_variable_content_relation.content_of_element.clone(),
                    ),
                    _ => panic!(),
                };

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

                let combined_number = get_combined_binary_number(dep_vars_assined_numbers);

                let target_node_index = {
                    let idx = ref_node_ids.iter().position(|id| id == &target_id).unwrap();
                    match under_for {
                        true => format!("[{}, ...$$lunasForIndices]", idx),
                        false => idx.to_string(),
                    }
                };

                fragments.push(format!(
                    "[[() => `{}`], {}, {}, {}]",
                    content,
                    target_node_index,
                    combined_number,
                    "1" // FragmentType.TEXT
                ));
            }
        }
    }
    if fragments.is_empty() {
        return None;
    }
    Some(fragments.join(",\n"))
}

pub fn gen_create_anchor_statements(
    text_node_renderer: &TextNodeRendererGroup,
    ctx_condition: &Vec<String>,
    ref_node_ids: &mut Vec<String>,
    under_for: bool,
) -> Option<String> {
    let ref_node_ids_count_before_creating_anchors = ref_node_ids.len();
    let mut create_anchor_statements = vec![];
    let mut iter = text_node_renderer.renderers.iter().peekable();
    let mut amount_of_next_elm = 1;
    while let Some(render) = iter.next() {
        match render {
            crate::structs::transform_info::TextNodeRenderer::ManualRenderer(txt_renderer) => {
                if &txt_renderer.ctx != ctx_condition {
                    continue;
                }
                let anchor_idx = match &txt_renderer.target_anchor_id {
                    Some(anchor_id) => {
                        let reference_node_idx = ref_node_ids.iter().position(|id| id == anchor_id);
                        match reference_node_idx {
                            Some(idx) => idx.to_string(),
                            None => "null".to_string(),
                        }
                    }
                    None => "null".to_string(),
                };
                let parent_node_idx = ref_node_ids
                    .iter()
                    .position(|id| id == &txt_renderer.parent_id)
                    .unwrap()
                    .to_string();
                let create_anchor_statement = format!(
                    "[1, {}, {}, `{}`],",
                    &parent_node_idx,
                    &anchor_idx,
                    &txt_renderer.content.trim(),
                );
                ref_node_ids.push(txt_renderer.text_node_id.clone());
                create_anchor_statements.push(create_anchor_statement);
            }
            _ => {
                let next_render = iter.peek();
                let (distance_to_next_elm, ctx, target_anchor_id, block_id, parent_id) =
                    render.get_empty_text_node_info();
                if distance_to_next_elm <= 1 {
                    continue;
                }
                if &ctx != ctx_condition {
                    continue;
                }
                if let Some(next_renderer) = next_render {
                    if render.is_next_elm_the_same_anchor(next_renderer) {
                        ref_node_ids.push(format!("{}-anchor", block_id));
                        amount_of_next_elm += 1;
                        continue;
                    }
                }

                let anchor_node_idx = match &target_anchor_id {
                    Some(anchor_id) => {
                        let reference_node_idx =
                            ref_node_ids.iter().position(|id| id == anchor_id).unwrap();
                        let reference_node_idx = reference_node_idx.to_string();
                        match under_for {
                            true => format!("[{}, ...$$lunasForIndices]", reference_node_idx),
                            false => reference_node_idx,
                        }
                    }
                    None => "null".to_string(),
                };
                let parent_node_idx = {
                    let parent_node_idx = ref_node_ids
                        .iter()
                        .position(|id| id == &parent_id)
                        .unwrap()
                        .to_string();
                    match under_for {
                        true => format!("[{}, ...$$lunasForIndices]", parent_node_idx),
                        false => parent_node_idx,
                    }
                };

                ref_node_ids
                    .iter()
                    .position(|id| id == &parent_id)
                    .unwrap()
                    .to_string();
                let create_anchor_statement = format!(
                    "[{}, {}, {}],",
                    amount_of_next_elm, parent_node_idx, anchor_node_idx
                );
                create_anchor_statements.push(create_anchor_statement);
                ref_node_ids.push(format!("{}-anchor", block_id));
                amount_of_next_elm = 1;
            }
        }
    }

    let anchor_offset = if ref_node_ids_count_before_creating_anchors == 0 {
        "".to_string()
    } else {
        format!(
            ", {}",
            ref_node_ids_count_before_creating_anchors
                .to_string()
                .as_str()
        )
    };

    if create_anchor_statements.is_empty() {
        return None;
    }

    Some(
        format!(
            r#"$$lunasInsertTextNodes([
{}
]{});"#,
            create_indent(create_anchor_statements.join("\n").as_str()),
            anchor_offset
        )
        .to_string(),
    )
}

// TODO: move to gen_custom_component.rs
pub fn gen_render_custom_component_statements(
    custom_component_block_info: &Vec<CustomComponentBlockInfo>,
    ctx: &Vec<String>,
    variable_names: &Vec<String>,
    ref_node_ids: &mut Vec<String>,
    under_for: bool,
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
            let anchor = match custom_component_block.distance_to_next_elm > 1 {
                true => {
                    let anchor_idx = ref_node_ids
                        .iter()
                        .position(|id| {
                            id == &format!(
                                "{}-anchor",
                                custom_component_block.custom_component_block_id
                            )
                        })
                        .unwrap()
                        .to_string();
                    match under_for {
                        true => format!("[{}, ...$$lunasForIndices]", anchor_idx),
                        false => anchor_idx.to_string(),
                    }
                }
                false => match &custom_component_block.target_anchor_id {
                    Some(anchor_id) => {
                        let reference_node_idx =
                            ref_node_ids.iter().position(|id| id == anchor_id).unwrap();
                        match under_for {
                            true => format!("[{}, ...$$lunasForIndices]", reference_node_idx),
                            false => reference_node_idx.to_string(),
                        }
                    }
                    None => "null".to_string(),
                },
            };
            let parent_idx = {
                let custom_component_parent_index = ref_node_ids
                    .iter()
                    .position(|id| id == &custom_component_block.parent_id)
                    .unwrap()
                    .to_string();
                match under_for {
                    true => format!("[{}, ...$$lunasForIndices]", custom_component_parent_index),
                    false => custom_component_parent_index,
                }
            };
            let ref_idx = match under_for {
                true => format!("[{}, ...$$lunasForIndices]", ref_node_ids.len()),
                false => ref_node_ids.len().to_string(),
            };
            render_custom_statements.push(format!(
                "$$lunasInsertComponent({}({}), {}, {}, {});",
                custom_component_block.component_name,
                custom_component_block.args.to_object(variable_names),
                parent_idx,
                anchor,
                ref_idx
            ));
            ref_node_ids.push(format!(
                "{}-component",
                custom_component_block.custom_component_block_id
            ));
        } else {
            let parent_idx = ref_node_ids
                .iter()
                .position(|id| id == &custom_component_block.parent_id)
                .unwrap()
                .to_string();
            let ref_idx = ref_node_ids.len();
            render_custom_statements.push(format!(
                "$$lunasMountComponent({}({}), {}, {});",
                custom_component_block.component_name,
                custom_component_block.args.to_object(variable_names),
                parent_idx,
                ref_idx
            ));
            ref_node_ids.push(format!(
                "{}-component",
                custom_component_block.custom_component_block_id
            ));
        }
    }
    render_custom_statements
}

// TODO: Review usage and make private if possible

/// Returns a binary number that is the result of ORing all the numbers in the argument.
/// ```
/// let numbers = vec![0b0001, 0b0010, 0b0100];
/// let result = get_combined_binary_number(numbers);
/// assert_eq!(result, 0b0111);
/// ```
pub fn get_combined_binary_number(numbers: Vec<u32>) -> u32 {
    let mut result = 0;
    for (_, &value) in numbers.iter().enumerate() {
        result |= value;
    }
    result
}

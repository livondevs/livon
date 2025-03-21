use crate::{
    generate_js::{
        create_event_listener, create_fragments, gen_create_anchor_statements,
        gen_ref_getter_from_needed_ids, gen_render_custom_component_statements,
        get_combined_binary_number,
    },
    orig_html_struct::structs::NodeContent,
    structs::{
        ctx::ContextCategories,
        transform_info::{
            ActionAndTarget, CustomComponentBlockInfo, IfBlockInfo, RefMap, TextNodeRendererGroup,
            VariableNameAndAssignedNumber,
        },
        transform_targets::NodeAndReactiveInfo,
    },
    transformers::html_utils::create_lunas_internal_component_statement,
};

use super::utils::create_indent;

// TODO: Many of the following functions are similar to top-level component creation functions, such as creating refs and rendering if statements. Consider refactoring them into a single function.
pub fn gen_render_if_blk_func(
    if_block_info: &Vec<IfBlockInfo>,
    needed_ids: &Vec<RefMap>,
    actions_and_targets: &Vec<ActionAndTarget>,
    text_node_renderer: &TextNodeRendererGroup,
    custom_component_blocks_info: &Vec<CustomComponentBlockInfo>,
    variable_names: &Vec<String>,
    dep_vars_assigned_numbers: &Vec<VariableNameAndAssignedNumber>,
    elm_and_var_relation: &Vec<NodeAndReactiveInfo>,
    ref_node_ids: &mut Vec<String>,
    ctx_categories: &ContextCategories,
    current_for_ctx: Option<&String>,
    under_for: bool,
) -> Option<String> {
    let mut render_if = vec![];

    for if_block in if_block_info.iter() {
        if !if_block.check_latest_for_ctx(ctx_categories, &current_for_ctx) {
            continue;
        }
        let initial_ref_node_ids_len = ref_node_ids.len();
        let if_blk_elm_loc = match under_for {
            true => format!("[{}, index]", ref_node_ids.len()),
            false => ref_node_ids.len().to_string(),
        };
        let create_internal_element_statement = match &if_block.node.content {
            NodeContent::Element(elm) => {
                create_lunas_internal_component_statement(elm, "$$createLunasElement")
            }
            _ => panic!(),
        };

        let mut post_render_statement: Vec<String> = Vec::new();

        let ref_getter_str = gen_ref_getter_from_needed_ids(
            needed_ids,
            &Some(&if_block.ctx_under_if),
            ref_node_ids,
            ctx_categories,
        );
        if let Some(ref_getter) = ref_getter_str {
            post_render_statement.push(ref_getter);
        }

        let if_blk_name = match under_for {
            true => format!("`{}-${{index}}`", if_block.target_if_blk_id),
            false => format!("\"{}\"", if_block.target_if_blk_id),
        };

        let ev_listener_code = create_event_listener(
            actions_and_targets,
            &if_block.ctx_under_if,
            &ref_node_ids,
            under_for,
        );
        if let Some(ev_listener_code) = ev_listener_code {
            post_render_statement.push(ev_listener_code);
        }

        let gen_anchor = gen_create_anchor_statements(
            &text_node_renderer,
            &if_block.ctx_under_if,
            ref_node_ids,
            under_for,
        );
        if let Some(gen_anchor) = gen_anchor {
            post_render_statement.push(gen_anchor);
        }

        let render_child_component = gen_render_custom_component_statements(
            &custom_component_blocks_info,
            &if_block.ctx_under_if,
            &variable_names,
            ref_node_ids,
            under_for,
        );
        if !render_child_component.is_empty() {
            post_render_statement.extend(render_child_component);
        }

        let parent_if_blk_id_idx_num = ref_node_ids
            .iter()
            .position(|x| x == &if_block.parent_id)
            .unwrap()
            .to_string();
        let parent_if_blk_id_idx = match under_for {
            // TODO: 重要 nestに対応せよ。
            true => format!("[{}, index]", parent_if_blk_id_idx_num),
            false => parent_if_blk_id_idx_num.to_string(),
        };
        let idx_of_anchor_of_if_blk = match if_block.distance_to_next_elm > 1 {
            true => Some(
                ref_node_ids
                    .iter()
                    .position(|x| x == &format!("{}-anchor", if_block.target_if_blk_id))
                    .unwrap()
                    .to_string(),
            ),
            false => match &if_block.target_anchor_id {
                Some(target_anchor_id) => Some(
                    ref_node_ids
                        .iter()
                        .position(|x| x == target_anchor_id)
                        .unwrap()
                        .to_string(),
                ),
                None => None,
            },
        };

        let if_on_create = match post_render_statement.is_empty() {
            true => "() => {}".to_string(),
            false => format!(
                r#"function() {{
{}
}}"#,
                create_indent(post_render_statement.join("\n").as_str()),
            ),
        };

        let anchor_idx = match idx_of_anchor_of_if_blk {
            Some(idx) => match under_for {
                true => format!(r#", [{}, index]"#, idx),
                false => format!(r#", {}"#, idx),
            },
            None => "".to_string(),
        };

        // array to js array string
        let ctxjs_array = {
            let mut ctx_over_if = if_block.ctx_over_if.clone();
            if under_for {
                let latest_for_ctx_idx = if_block.get_latest_for_ctx_idx(ctx_categories);
                if let Some(latest_for_ctx_idx) = latest_for_ctx_idx {
                    ctx_over_if = ctx_over_if
                        .iter()
                        // Get elements after latest_for_ctx_idx (excluding latest_for_ctx_idx itself)
                        .skip(latest_for_ctx_idx + 1)
                        .map(|x| x.to_string())
                        .collect();
                }
            }
            format!(
                r#"[{}]"#,
                ctx_over_if
                    .iter()
                    .map(|x| format!("\"{}\"", x))
                    .collect::<Vec<String>>()
                    .join(",")
            )
        };

        let parent_for_array = {
            let for_context_before_current_for = if under_for {
                let latest_for_ctx_idx = if_block.get_latest_for_ctx_idx(ctx_categories);
                if let Some(latest_for_ctx_idx) = latest_for_ctx_idx {
                    if_block
                        .ctx_over_if
                        .iter()
                        .take(latest_for_ctx_idx + 1)
                        .filter(|x| ctx_categories.for_ctx.iter().any(|f| f == *x))
                        .map(|x| x.to_string())
                        .collect()
                } else {
                    vec![]
                }
            } else {
                vec![]
            };
            format!(
                r#"[{}]"#,
                for_context_before_current_for
                    .iter()
                    .map(|x| format!("\"{}\"", x))
                    .collect::<Vec<String>>()
                    .join(",")
            )
        };

        let ref_node_ids_len_increase = ref_node_ids.len() - initial_ref_node_ids_len;
        let dep_number = dep_vars_assigned_numbers
            .iter()
            .filter(|v| {
                if_block
                    .condition_dep_vars
                    .iter()
                    .map(|d| *d == v.name)
                    .collect::<Vec<bool>>()
                    .contains(&true)
            })
            .map(|v| v.assignment)
            .collect::<Vec<u32>>();

        let fragments = create_fragments(
            &elm_and_var_relation,
            &dep_vars_assigned_numbers,
            &ref_node_ids,
            &if_block.ctx_under_if,
        );

        let if_fragments = if let Some(fragments) = fragments {
            format!(
                r#",
[
{}
]"#,
                create_indent(fragments.as_str())
            )
        } else {
            "".to_string()
        };

        let create_if_func_inside = format!(
            r#"{},
() => ({}),
() => ({}),
{},
{},
{},
{},
[{}, {}],
[{}{}]{}"#,
            if_blk_name,
            create_internal_element_statement,
            if_block.condition,
            if_on_create,
            ctxjs_array,
            parent_for_array,
            get_combined_binary_number(dep_number),
            if_blk_elm_loc,
            ref_node_ids_len_increase,
            parent_if_blk_id_idx,
            anchor_idx,
            if_fragments
        );

        let create_if_func = format!(
            r#"[
{}
]"#,
            create_indent(create_if_func_inside.as_str())
        );

        render_if.push(create_if_func);
    }

    if render_if.is_empty() {
        return None;
    }

    Some(format!(
        r#"$$lunasCreateIfBlock([
{}
]);"#,
        create_indent(render_if.join(",\n").as_str())
    ))
}

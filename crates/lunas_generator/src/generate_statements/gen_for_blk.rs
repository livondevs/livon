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
            ActionAndTarget, CustomComponentBlockInfo, ForBlockInfo, IfBlockInfo, RefMap,
            TextNodeRendererGroup, VariableNameAndAssignedNumber,
        },
        transform_targets::NodeAndReactiveInfo,
    },
    transformers::html_utils::create_lunas_internal_component_statement,
};

use super::{gen_if_blk::gen_render_if_blk_func, utils::create_indent};

// TODO: Many of the following functions are similar to top-level component creation functions, such as creating refs and rendering if statements. Consider refactoring them into a single function.
pub fn gen_render_for_blk_func(
    for_block_info: &Vec<ForBlockInfo>,
    ref_map: &Vec<RefMap>,
    actions_and_targets: &Vec<ActionAndTarget>,
    text_node_renderer: &TextNodeRendererGroup,
    custom_component_blocks_info: &Vec<CustomComponentBlockInfo>,
    variable_names: &Vec<String>,
    dep_vars_assigned_numbers: &Vec<VariableNameAndAssignedNumber>,
    elm_and_var_relation: &Vec<NodeAndReactiveInfo>,
    ref_node_ids: &mut Vec<String>,
    ctx_categories: &ContextCategories,
    if_blocks_info: &Vec<IfBlockInfo>,
) -> Option<String> {
    let mut render_for = vec![];

    for for_block in for_block_info.iter() {
        let initial_ref_node_ids_len = ref_node_ids.len();
        let create_internal_element_statement = match &for_block.node.content {
            NodeContent::Element(elm) => {
                create_lunas_internal_component_statement(elm, "$$createLunasElement")
            }
            _ => panic!(),
        };

        let mut post_render_statement: Vec<String> = Vec::new();

        let ref_getter_str = gen_ref_getter_from_needed_ids(
            ref_map,
            &Some(&for_block.ctx_under_for),
            ref_node_ids,
            ctx_categories,
        );
        if let Some(ref_getter) = ref_getter_str {
            post_render_statement.push(ref_getter);
        }

        let ev_listener_code = create_event_listener(
            actions_and_targets,
            &for_block.ctx_under_for,
            &ref_node_ids,
            true,
        );
        if let Some(ev_listener_code) = ev_listener_code {
            post_render_statement.push(ev_listener_code.clone());
        }

        let gen_anchor = gen_create_anchor_statements(
            &text_node_renderer,
            &for_block.ctx_under_for,
            ref_node_ids,
        );
        if let Some(gen_anchor) = gen_anchor {
            post_render_statement.push(gen_anchor);
        }

        let render_child_component = gen_render_custom_component_statements(
            &custom_component_blocks_info,
            &for_block.ctx_under_for,
            &variable_names,
            ref_node_ids,
        );
        if !render_child_component.is_empty() {
            post_render_statement.extend(render_child_component);
        }

        // ctx_under_forの最後の要素
        let last_ctx_under_for = for_block.ctx_under_for.last().unwrap();
        let if_blk_gen = gen_render_if_blk_func(
            &if_blocks_info,
            &ref_map,
            &actions_and_targets,
            &text_node_renderer,
            &custom_component_blocks_info,
            &variable_names,
            &dep_vars_assigned_numbers,
            &elm_and_var_relation,
            ref_node_ids,
            &ctx_categories,
            Some(last_ctx_under_for),
            true,
        );

        if let Some(if_blk_gen) = if_blk_gen {
            post_render_statement.push(if_blk_gen);
        }

        // forブロックの初期化処理（renderedNodeId用）を生成
        let for_on_create = if post_render_statement.is_empty() {
            "() => {}".to_string()
        } else {
            format!(
                r#"({}, {}) => {{
{}
}}"#,
                for_block.item_name,
                for_block.item_index,
                create_indent(post_render_statement.join("\n").as_str()),
            )
        };

        let ref_node_ids_len_increase = ref_node_ids.len() - initial_ref_node_ids_len;
        let dep_number = dep_vars_assigned_numbers
            .iter()
            .filter(|v| {
                for_block
                    .dep_vars
                    .iter()
                    .map(|d| *d == v.name)
                    .collect::<Vec<bool>>()
                    .contains(&true)
            })
            .map(|v| v.assignment)
            .collect::<Vec<u32>>();

        let _fragments = create_fragments(
            &elm_and_var_relation,
            &dep_vars_assigned_numbers,
            &ref_node_ids,
            &for_block.ctx_under_for,
        );
        // いったん fragments は使用しないので除外
        let parent_if_blk_id_idx = ref_node_ids
            .iter()
            .position(|x| x == &for_block.parent_id)
            .unwrap()
            .to_string();
        let idx_of_anchor_of_if_blk = match for_block.distance_to_next_elm > 1 {
            true => Some(
                ref_node_ids
                    .iter()
                    .position(|x| x == &format!("{}-anchor", for_block.target_for_blk_id))
                    .unwrap()
                    .to_string(),
            ),
            false => match &for_block.target_anchor_id {
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

        let anchor_idx = match idx_of_anchor_of_if_blk {
            Some(idx) => format!(r#", {}"#, idx),
            None => "".to_string(),
        };

        let create_for_func_inside = format!(
            r#""{}",
({}, {}) => {},
() => ({}),
{},
{},
[{}, {}],
[{}{}]"#,
            for_block.target_for_blk_id,
            for_block.item_name,
            for_block.item_index,
            create_internal_element_statement,
            for_block.item_collection,
            for_on_create,
            get_combined_binary_number(dep_number),
            initial_ref_node_ids_len,
            ref_node_ids_len_increase,
            parent_if_blk_id_idx,
            anchor_idx,
        );

        let create_for_func = format!(
            r#"[
{}
]"#,
            create_indent(create_for_func_inside.as_str())
        );

        render_for.push(create_for_func);
    }

    if render_for.is_empty() {
        return None;
    }

    Some(format!(
        r#"$$lunasCreateForBlock([
{}
]);"#,
        create_indent(render_for.join(",\n").as_str())
    ))
}

use crate::{
    generate_js::{
        create_event_listener, gen_create_anchor_statements, gen_ref_getter_from_needed_ids,
        gen_render_custom_component_statements,
    },
    orig_html_struct::structs::NodeContent,
    structs::transform_info::{
        ActionAndTarget, CustomComponentBlockInfo, IfBlockInfo, NeededIdName, TextNodeRendererGroup,
    },
    transformers::html_utils::create_lunas_internal_component_statement,
};

use super::utils::create_indent;

// TODO: Many of the following functions are similar to top-level component creation functions, such as creating refs and rendering if statements. Consider refactoring them into a single function.
pub fn gen_render_if_blk_func(
    if_block_info: &Vec<IfBlockInfo>,
    needed_ids: &Vec<NeededIdName>,
    actions_and_targets: &Vec<ActionAndTarget>,
    text_node_renderer: &TextNodeRendererGroup,
    custom_component_blocks_info: &Vec<CustomComponentBlockInfo>,
    variable_names: &Vec<String>,
) -> Vec<String> {
    let mut render_if = vec![];

    for if_block in if_block_info.iter() {
        // create element
        let create_internal_element_statement = match &if_block.node.content {
            NodeContent::Element(elm) => {
                create_lunas_internal_component_statement(elm, "$$createLunasElement")
            }
            _ => panic!(),
        };

        let mut rendering_statement = vec![];

        let ref_getter_str = gen_ref_getter_from_needed_ids(
            needed_ids,
            &Some(if_block),
            &Some(&if_block.ctx_under_if),
        );
        rendering_statement.push(ref_getter_str.as_str());

        let ev_listener_code = create_event_listener(actions_and_targets, &if_block.ctx_under_if);
        if ev_listener_code.len() != 0 {
            rendering_statement.extend(ev_listener_code.iter().map(|x| x.as_str()));
        }

        let gen_anchor = gen_create_anchor_statements(&text_node_renderer, &if_block.ctx_under_if);
        rendering_statement.extend(gen_anchor.iter().map(|x| x.as_str()));

        let render_child_component = gen_render_custom_component_statements(
            &custom_component_blocks_info,
            &if_block.ctx_under_if,
            &variable_names,
        );
        if render_child_component.len() != 0 {
            rendering_statement.extend(render_child_component.iter().map(|x| x.as_str()));
        }

        // if there are children if block under the if block, render them
        let children = if_block.find_children(&if_block_info);

        let child_block_rendering_exec = if children.len() != 0 {
            let mut child_block_rendering_exec = vec![];
            for child_if in children {
                child_block_rendering_exec.push(format!(
                    "\n({}) && $$lunasRenderIfBlock(\"{}\");",
                    child_if.condition, &child_if.if_blk_id
                ));
            }
            child_block_rendering_exec
        } else {
            vec![]
        };
        rendering_statement.extend(child_block_rendering_exec.iter().map(|x| x.as_str()));

        let name_of_parent_of_if_blk = format!("$$lunas{}Ref", if_block.parent_id);
        let name_of_anchor_of_if_blk = match if_block.distance_to_next_elm > 1 {
            true => format!("$$lunas{}Anchor", if_block.if_blk_id),
            false => match if_block.target_anchor_id {
                Some(_) => format!(
                    "$$lunas{}Ref",
                    if_block.target_anchor_id.as_ref().unwrap().clone()
                ),
                None => format!("null"),
            },
        };

        let if_on_create = match rendering_statement.len() == 0 {
            true => "() => {}".to_string(),
            false => format!(
                r#"function() {{
{}
}}"#,
                create_indent(rendering_statement.join("\n").as_str()),
            ),
        };

        let create_if_func_inside = format!(
            r#""{}",
()=>{},
()=>[{},{}],
{},"#,
            if_block.target_if_blk_id,
            create_internal_element_statement,
            name_of_parent_of_if_blk,
            name_of_anchor_of_if_blk,
            if_on_create,
        );

        let create_if_func = format!(
            r#"$$lunasCreateIfBlock(
{}
);"#,
            create_indent(create_if_func_inside.as_str())
        );

        render_if.push(create_if_func);
        if if_block.ctx_over_if.len() == 0 {
            render_if.push(format!(
                "({}) && $$lunasRenderIfBlock(\"{}\")",
                if_block.condition, &if_block.if_blk_id
            ));
        }
    }
    render_if
}

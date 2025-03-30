use crate::structs::transform_info::CustomComponentBlockInfo;

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
            let latest_ctx = match custom_component_block.ctx.last() {
                Some(ctx) => format!(r#""{}""#, ctx),
                None => "null".to_string(),
            };
            let indices = match under_for {
                true => format!("$$lunasForIndices"),
                false => "null".to_string(),
            };
            render_custom_statements.push(format!(
                "$$lunasInsertComponent({}({}), {}, {}, {}, {}, {});",
                custom_component_block.component_name,
                custom_component_block.args.to_object(variable_names),
                parent_idx,
                anchor,
                ref_idx,
                latest_ctx,
                indices
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

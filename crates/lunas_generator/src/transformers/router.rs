use crate::{consts::ROUTER_VIEW, structs::transform_info::CustomComponentBlockInfo};

pub fn generate_router_initialization_code(
    custom_component_blocks_info: &Vec<CustomComponentBlockInfo>,
    ref_node_ids: &Vec<String>,
) -> Result<String, String> {
    match custom_component_blocks_info
        .into_iter()
        .find(|cc| cc.component_name == ROUTER_VIEW)
    {
        Some(router_component) => Ok(if router_component.have_sibling_elm {
            match router_component.distance_to_next_elm > 1 {
                true => {
                    let parent_ref_idx = format!(
                        "$$lunasGetElm({})",
                        &ref_node_ids
                            .iter()
                            .position(|id| *id == router_component.parent_id)
                            .unwrap()
                            .to_string()
                    );
                    let x = format!(
                        "$$lunasGetElm({})",
                        ref_node_ids
                            .iter()
                            .position(|x| {
                                x == &format!(
                                    "{}-anchor",
                                    router_component.custom_component_block_id
                                )
                            })
                            .unwrap()
                            .to_string()
                    );
                    format!(
                        "$$lunasRouter.initialize($$lunasGeneratedRoutes, {}, {}, true);",
                        parent_ref_idx, x
                    )
                }
                false => {
                    let parent_ref_idx = format!(
                        "$$lunasGetElm({})",
                        &ref_node_ids
                            .iter()
                            .position(|id| *id == router_component.parent_id)
                            .unwrap()
                            .to_string()
                    );
                    let anchor_ref_idx = match &router_component.target_anchor_id {
                        Some(anchor_id) => format!(
                            "$$lunasGetElm({})",
                            ref_node_ids
                                .iter()
                                .position(|id| id == anchor_id)
                                .unwrap()
                                .to_string()
                        ),
                        None => "null".to_string(),
                    };
                    format!(
                        "$$lunasRouter.initialize($$lunasGeneratedRoutes, {}, {}, true);",
                        parent_ref_idx, anchor_ref_idx
                    )
                }
            }
        } else {
            let parent_ref_idx = format!(
                "$$lunasGetElm({})",
                &ref_node_ids
                    .iter()
                    .position(|id| *id == router_component.parent_id)
                    .unwrap()
                    .to_string()
            );
            format!(
                "$$lunasRouter.initialize($$lunasGeneratedRoutes, {}, null, false);",
                parent_ref_idx,
            )
        }),
        None => Err("RouterView component not found".to_string()),
    }
}

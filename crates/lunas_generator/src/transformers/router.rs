use crate::{consts::ROUTER_VIEW, structs::transform_info::CustomComponentBlockInfo};

pub fn generate_router_initialization_code(
    custom_component_blocks_info: &Vec<CustomComponentBlockInfo>,
) -> Result<String, String> {
    match custom_component_blocks_info
        .into_iter()
        .find(|cc| cc.component_name == ROUTER_VIEW)
    {
        Some(router_component) => Ok(if router_component.have_sibling_elm {
            match router_component.distance_to_next_elm > 1 {
                true => {
                    format!(
                        "$$lunasRouter.initialize($$lunasGeneratedRoutes, $$lunas{}Ref, $$lunas{}Anchor, true);",
                        router_component.parent_id, router_component.custom_component_block_id
                    )
                }
                false => {
                    let anchor_ref_name = match &router_component.target_anchor_id {
                        Some(anchor_id) => format!("$$lunas{}Ref", anchor_id),
                        None => "null".to_string(),
                    };
                    format!(
                        "$$lunasRouter.initialize($$lunasGeneratedRoutes, $$lunas{}Ref, {}, true);",
                        router_component.parent_id, anchor_ref_name
                    )
                }
            }
        } else {
            format!(
                "$$lunasRouter.initialize($$lunasGeneratedRoutes, $$lunas{}Ref, null, false);",
                router_component.parent_id,
            )
        }),
        None => Err("RouterView component not found".to_string()),
    }
}

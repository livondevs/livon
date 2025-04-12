use crate::structs::transform_info::ActionAndTarget;

use super::utils::create_indent;

pub fn generate_create_event_listener(
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
            true => format!("[{}, ...$$livonForIndices]", reference_node_idx),
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
        r#"$$livonAddEvListener([
{}
]);"#,
        formatted_result
    ))
}

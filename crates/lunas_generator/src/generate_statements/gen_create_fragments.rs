use crate::{
    generate_js::get_combined_binary_number,
    structs::{
        transform_info::VariableNameAndAssignedNumber, transform_targets::NodeAndReactiveInfo,
    },
};

use super::utils::create_indent;

pub fn gen_create_fragments(
    elm_and_variable_relations: &Vec<NodeAndReactiveInfo>,
    variable_name_and_assigned_numbers: &Vec<VariableNameAndAssignedNumber>,
    ref_node_ids: &Vec<String>,
    current_ctx: &Vec<String>,
    under_for: bool,
    fragment_func_args: &Option<Vec<String>>,
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
                        .collect::<Vec<u128>>();

                    let target_node_idx = {
                        let target_node_idx = ref_node_ids
                            .iter()
                            .position(|id| id == &elm_and_attr_relation.elm_id)
                            .unwrap();

                        match under_for {
                            true => format!("[{}, ...$$lunasForIndices]", target_node_idx),
                            false => target_node_idx.to_string(),
                        }
                    };

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
                    .collect::<Vec<u128>>();

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

    if let Some(args_vec) = fragment_func_args {
        let args = args_vec.iter().cloned().collect::<Vec<String>>().join(", ");
        Some(format!(
            r#"({}) => [
{}
]"#,
            args,
            create_indent(fragments.join(",\n").as_str())
        ))
    } else {
        Some(format!(
            r#"[
{}
]"#,
            create_indent(fragments.join(",\n").as_str())
        ))
    }
}

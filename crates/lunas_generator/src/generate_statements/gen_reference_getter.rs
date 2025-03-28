use crate::structs::transform_info::{IdBasedElementAccess, RefMap};

use super::utils::gen_binary_map_from_bool;

pub fn gen_reference_getter(
    ref_maps: &Vec<RefMap>,
    ctx: &Option<&Vec<String>>,
    ref_node_ids: &mut Vec<String>,
    is_under_for: bool,
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
    let offset = if !is_under_for {
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

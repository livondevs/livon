use nanoid::nanoid;

use crate::{
    consts::ROUTER_COMPONENTS,
    orig_html_struct::{
        html_manipulation::{
            HtmlManipulation, HtmlManipulator, RemoveChildForCustomComponent,
            RemoveChildForIfStatement, RemoveChildForRepeatStatement, RemoveChildTextNode,
            SetIdToParentForChildReactiveText,
        },
        structs::{Element, Node, NodeContent},
    },
    structs::{
        ctx::ContextCategories,
        transform_info::{
            ActionAndTarget, ComponentArgs, CustomComponentBlockInfo, EventBindingStatement,
            EventTarget, ForBlockInfo, IdBasedElementAccess, IfBlockInfo,
            ManualRendererForTextNode, NodeCreationMethod, RefMap,
        },
        transform_targets::{
            ElmAndReactiveAttributeRelation, ElmAndVariableContentRelation, NodeAndReactiveInfo,
            ReactiveAttr, TextAndVariableContentRelation,
        },
    },
};

use super::utils::{append_v_to_vars_in_html, UUID_GENERATOR};

// TODO:この関数の責務が多すぎるので、可能な限り分離させる
// TODO:dep_vars の使い方を再考する
// TODO: 引数が大きすぎるので、共通の目的を持った引数はstructとしてグループ化する
pub fn check_html_elms(
    varibale_names: &Vec<String>,
    component_names: &Vec<String>,
    node: &mut Node,
    // TODO: needed_idsからリネーム
    needed_ids: &mut Vec<RefMap>,
    elm_and_var_relation: &mut Vec<NodeAndReactiveInfo>,
    actions_and_targets: &mut Vec<ActionAndTarget>,
    parent_uuid: Option<&String>,
    html_manipulators: &mut Vec<HtmlManipulator>,
    if_blocks_info: &mut Vec<IfBlockInfo>,
    for_blocks_info: &mut Vec<ForBlockInfo>,
    custom_component_blocks_info: &mut Vec<CustomComponentBlockInfo>,
    txt_node_renderer: &mut Vec<ManualRendererForTextNode>,
    ctx_cats: &mut ContextCategories,
    if_blk_ctx: &Vec<String>,
    element_location: &Vec<usize>,
    count_of_siblings: usize,
    txt_node_to_be_deleted: bool,
) -> Result<(), String> {
    let node_id = node.uuid.clone();
    match &mut node.content {
        NodeContent::Element(element) => {
            let mut ctx_array = if_blk_ctx.clone();
            if !component_names.contains(&element.tag_name) {
                for (key, action_value) in &element.attributes.clone() {
                    // if attrs.name starts with "@"
                    if key.starts_with("@") {
                        let action_name = &key[1..];
                        set_id_for_needed_elm(
                            element,
                            needed_ids,
                            &node_id,
                            &ctx_array,
                            element_location,
                            false,
                        );
                        if let Some(value) = &&action_value {
                            actions_and_targets.push(ActionAndTarget {
                                action_name: action_name.to_string(),
                                action: EventTarget::new(value.to_string(), varibale_names),
                                target: node_id.clone(),
                                ctx: ctx_array.clone(),
                            })
                        }
                        element.attributes.remove(key);
                    } else if key == ":if" {
                        // TODO: Add error message for unwrap below
                        let condition = action_value.clone().unwrap();
                        let ctx_under_if = {
                            let mut ctx = ctx_array.clone();
                            ctx.push(node.uuid.clone());
                            ctx
                        };
                        ctx_cats.if_ctx.push(node.uuid.clone());
                        html_manipulators.push(HtmlManipulator {
                            target_uuid: parent_uuid.unwrap().clone(),
                            manipulations: HtmlManipulation::RemoveChildForIfStatement(
                                RemoveChildForIfStatement {
                                    child_uuid: node.uuid.clone(),
                                    condition: condition.clone(),
                                    block_id: node_id.clone(),
                                    ctx_over_if: ctx_array.clone(),
                                    ctx_under_if,
                                    elm_loc: element_location.clone(),
                                },
                            ),
                        });
                        element.attributes.remove(key);
                        element
                            .attributes
                            .insert("$$$conditional$$$".to_string(), None);
                        ctx_array.push(node.uuid.clone());
                    } else if key == ":for" {
                        // TODO: Add error message for unwrap below
                        let condition = action_value.clone().unwrap();
                        // TODO: (P1) implement parser for 'for of'
                        // Pattern1 :for="item of items"
                        // Pattern2 :for="(item) of items"
                        // Pattern3 :for="(item, index) of items"
                        let parts: Vec<&str> = condition.split(" of ").collect();
                        if parts.len() != 2 {
                            return Err(format!(
                                "Invalid format for :for directive. Expected 'item of items', got '{}'",
                                condition
                            ));
                        }
                        let item_part = parts[0].trim();
                        let items_collection = parts[1].trim();

                        let (item_name, index_name) = if item_part.starts_with('(')
                            && item_part.ends_with(')')
                        {
                            let inner_parts: Vec<&str> = item_part[1..item_part.len() - 1]
                                .split(',')
                                .map(|s| s.trim())
                                .collect();
                            match inner_parts.len() {
                                1 => (inner_parts[0], Some("index")),
                                2 => (inner_parts[0], Some(inner_parts[1])),
                                _ => return Err(format!(
                                    "Invalid format for :for directive. Expected '(item, index) of items', got '{}'",
                                    condition
                                )),
                            }
                        } else {
                            (item_part, Some("index"))
                        };

                        // Validate item_name is a valid variable name
                        if item_name.is_empty() {
                            return Err(format!(
                                "Item variable name cannot be empty in :for directive"
                            ));
                        }

                        // Check if the items_collection exists in the available variables
                        if !varibale_names.contains(&items_collection.to_string())
                            && !items_collection.contains(".")
                        {
                            // Allow dot notation for accessing properties, but warn about the base variable
                            return Err(format!(
                                "Collection '{}' in :for directive is not defined",
                                items_collection
                            ));
                        }

                        let ctx_under_if = {
                            let mut ctx = ctx_array.clone();
                            ctx.push(node.uuid.clone());
                            ctx
                        };
                        ctx_cats.for_ctx.push(node.uuid.clone());

                        html_manipulators.push(HtmlManipulator {
                            target_uuid: parent_uuid.unwrap().clone(),
                            manipulations: HtmlManipulation::RemoveChildForForStatement(
                                RemoveChildForRepeatStatement {
                                    child_uuid: node.uuid.clone(),
                                    item_name: item_name.to_string(),
                                    item_index: index_name.unwrap().to_string(),
                                    item_collection: items_collection.to_string(),
                                    block_id: node_id.clone(),
                                    ctx_over_for: ctx_array.clone(),
                                    ctx_under_for: ctx_under_if,
                                    elm_loc: element_location.clone(),
                                },
                            ),
                        });
                        element.attributes.remove(key);
                        element
                            .attributes
                            .insert("$$$conditional$$$".to_string(), None);
                        ctx_array.push(node.uuid.clone());
                    } else if key.starts_with("::") {
                        let binding_attr = &key[2..];
                        set_id_for_needed_elm(
                            element,
                            needed_ids,
                            &node_id,
                            &ctx_array,
                            element_location,
                            false,
                        );
                        if let Some(value) = &&action_value {
                            actions_and_targets.push(ActionAndTarget {
                                action_name: "input".to_string(),
                                action: EventTarget::EventBindingStatement(EventBindingStatement {
                                    statement: format!(
                                        "{}.v = event.target.{}",
                                        &value, &binding_attr
                                    ),
                                    arg: "e".to_string(),
                                }),
                                target: node_id.clone(),
                                ctx: ctx_array.clone(),
                            });
                            elm_and_var_relation.push(
                                NodeAndReactiveInfo::ElmAndReactiveAttributeRelation(
                                    ElmAndReactiveAttributeRelation {
                                        elm_id: node_id.clone(),
                                        reactive_attr: vec![ReactiveAttr {
                                            attribute_key: binding_attr.to_string(),
                                            content_of_attr: format!("{}.v", value),
                                            variable_names: vec![value.clone()],
                                        }],
                                        ctx: ctx_array.clone(),
                                        elm_loc: element_location.clone(),
                                    },
                                ),
                            );
                        }
                        element.attributes.remove(key);
                    } else if key.starts_with(":") {
                        // TODO: reconsider about this constraint
                        if key == ":innerHtml" {
                            Err(format!(":innerHtml is not supported"))?;
                        } else if key == ":textContent" {
                            Err(format!(":textContent is not supported"))?;
                        }
                        let id: String = set_id_for_needed_elm(
                            element,
                            needed_ids,
                            &node_id,
                            &ctx_array,
                            element_location,
                            false,
                        );
                        let raw_attr_name = &key[1..];
                        let raw_attr_value = action_value.clone();

                        let reactive_attr_info =
                            find_reactive_attr_from_id(&id, elm_and_var_relation);

                        // if elm_and_var_relation includes elm_id

                        let reactive_attr_info = match reactive_attr_info {
                            Some(rel) => rel,
                            None => {
                                let rel2 = ElmAndReactiveAttributeRelation {
                                    elm_id: node_id.clone(),
                                    reactive_attr: vec![],
                                    ctx: ctx_array.clone(),
                                    elm_loc: element_location.clone(),
                                };
                                elm_and_var_relation.push(
                                    NodeAndReactiveInfo::ElmAndReactiveAttributeRelation(rel2),
                                );
                                find_reactive_attr_from_id(&node_id, elm_and_var_relation).unwrap()
                            }
                        };

                        // Check if the value is null
                        // TODO:要素のIndexを返すようにする
                        if raw_attr_value.is_none() {
                            Err(format!("value of attribute :{} is null", raw_attr_name))?;
                        }

                        let mut raw_attr_value = raw_attr_value.unwrap();

                        let (raw_attr_value, used_vars) =
                            append_v_to_vars_in_html(&mut raw_attr_value, varibale_names);

                        element.attributes.remove(key);
                        element.attributes.insert(
                            raw_attr_name.to_string(),
                            Some(format!("${{{}}}", raw_attr_value)),
                        );

                        let reactive_attr = ReactiveAttr {
                            attribute_key: raw_attr_name.to_string(),
                            content_of_attr: raw_attr_value,
                            variable_names: used_vars,
                        };

                        reactive_attr_info.reactive_attr.push(reactive_attr);
                    }
                }

                // When the tag_name corresponds to the component_names
            } else {
                html_manipulators.push(HtmlManipulator {
                    // TODO: add error message for unwrap below
                    target_uuid: parent_uuid.unwrap().clone(),
                    manipulations: HtmlManipulation::RemoveChildForCustomComponent(
                        RemoveChildForCustomComponent {
                            component_name: element.tag_name.clone(),
                            attributes: element.attributes_without_meta(),
                            child_uuid: node.uuid.clone(),
                            ctx: ctx_array.clone(),
                            elm_loc: element_location.clone(),
                        },
                    ),
                });
            }

            let count_of_siblings = element.children.len();

            let element_children = element.children.clone();
            for (index, child_node) in element.children.iter_mut().enumerate() {
                let mut new_element_location = element_location.clone();
                new_element_location.push(index);

                let txt_node_to_be_deleted = if index != 0 {
                    match &element_children.get(index - 1).unwrap().content {
                        NodeContent::Element(next_element) => {
                            next_element
                                .attributes_without_meta()
                                .iter()
                                .any(|f| f.0.starts_with(":if"))
                                || component_names.contains(&next_element.tag_name)
                        }
                        _ => true,
                    }
                } else {
                    false
                };

                check_html_elms(
                    varibale_names,
                    component_names,
                    child_node,
                    needed_ids,
                    elm_and_var_relation,
                    actions_and_targets,
                    Some(&node.uuid),
                    html_manipulators,
                    if_blocks_info,
                    for_blocks_info,
                    custom_component_blocks_info,
                    txt_node_renderer,
                    ctx_cats,
                    &ctx_array,
                    &new_element_location,
                    count_of_siblings,
                    txt_node_to_be_deleted,
                )?;
            }

            // TODO: 下の処理を関数にまとめる

            html_manipulators.sort_by(|a, b| {
                fn manip_to_ctx(manip: &HtmlManipulator) -> Vec<usize> {
                    match &manip.manipulations {
                        HtmlManipulation::RemoveChildForIfStatement(a) => a.elm_loc.clone(),
                        HtmlManipulation::RemoveChildForCustomComponent(b) => b.elm_loc.clone(),
                        HtmlManipulation::SetIdForReactiveContent(c) => c.elm_loc.clone(),
                        HtmlManipulation::RemoveChildTextNode(d) => d.elm_loc.clone(),
                        HtmlManipulation::RemoveChildForForStatement(e) => e.elm_loc.clone(),
                    }
                }
                let aloc = manip_to_ctx(a);
                let bloc = manip_to_ctx(b);
                aloc.cmp(&bloc)
            });

            // TODO: When html_manipulators is consumed, it should be removed
            for manip in html_manipulators {
                if manip.target_uuid == node.uuid {
                    match &manip.manipulations {
                        HtmlManipulation::RemoveChildForIfStatement(remove_statement) => {
                            set_id_for_needed_elm(
                                element,
                                needed_ids,
                                &node_id,
                                &remove_statement.ctx_over_if,
                                element_location,
                                false,
                            );
                            let (mut deleted_node, _, distance, idx_of_ref) =
                                element.remove_child(&remove_statement.child_uuid, component_names);

                            let _deleted_elm = match &mut deleted_node.content {
                                NodeContent::Element(elm) => elm,
                                _ => panic!("not element"),
                            };

                            create_space_for_ref_map(
                                needed_ids,
                                &deleted_node.uuid,
                                &remove_statement.ctx_under_if,
                                &remove_statement.elm_loc,
                            );

                            // TODO:remove_childにまとめる
                            let target_anchor_id = if let Some(idx_of_ref) = idx_of_ref {
                                let node_id =
                                    &element.children[idx_of_ref as usize - 1].uuid.clone();
                                Some(set_id_for_needed_elm(
                                    match &mut element.children[idx_of_ref as usize - 1].content {
                                        NodeContent::Element(elm) => elm,
                                        _ => panic!("not element"),
                                    },
                                    needed_ids,
                                    node_id,
                                    // TODO: ifブロックの親のctxを指定しているが、その旨が明示的ではないので、明示的にする
                                    &if_blk_ctx,
                                    {
                                        let mut new_element_location = element_location.clone();
                                        new_element_location.push(idx_of_ref as usize);
                                        &new_element_location.clone()
                                    },
                                    false,
                                ));
                                Some(node_id.clone())
                            } else {
                                None
                            };
                            let ref_text_node_id = match distance != 1 {
                                true => Some(nanoid!()),
                                false => None,
                            };
                            let (cond, dep_vars) = append_v_to_vars_in_html(
                                remove_statement.condition.as_str(),
                                &varibale_names,
                            );
                            if_blocks_info.push(IfBlockInfo {
                                parent_id: node_id.clone(),
                                target_if_blk_id: remove_statement.child_uuid.clone(),
                                distance_to_next_elm: distance,
                                target_anchor_id,
                                node: deleted_node,
                                ref_text_node_id,
                                condition: cond,
                                condition_dep_vars: dep_vars,
                                ctx_under_if: remove_statement.ctx_under_if.clone(),
                                ctx_over_if: remove_statement.ctx_over_if.clone(),
                                if_blk_id: remove_statement.block_id.clone(),
                                element_location: remove_statement.elm_loc.clone(),
                            });
                        }
                        HtmlManipulation::RemoveChildForForStatement(remove_statement) => {
                            set_id_for_needed_elm(
                                element,
                                needed_ids,
                                &node_id,
                                &remove_statement.ctx_over_for,
                                element_location,
                                false,
                            );
                            let (mut deleted_node, _, distance, idx_of_ref) =
                                element.remove_child(&remove_statement.child_uuid, component_names);

                            let _deleted_elm = match &mut deleted_node.content {
                                NodeContent::Element(elm) => elm,
                                _ => panic!("not element"),
                            };

                            create_space_for_ref_map(
                                needed_ids,
                                &deleted_node.uuid,
                                &remove_statement.ctx_under_for,
                                &remove_statement.elm_loc,
                            );

                            // TODO:remove_childにまとめる
                            let target_anchor_id = if let Some(idx_of_ref) = idx_of_ref {
                                let node_id =
                                    &element.children[idx_of_ref as usize - 1].uuid.clone();
                                Some(set_id_for_needed_elm(
                                    match &mut element.children[idx_of_ref as usize - 1].content {
                                        NodeContent::Element(elm) => elm,
                                        _ => panic!("not element"),
                                    },
                                    needed_ids,
                                    node_id,
                                    // TODO: ifブロックの親のctxを指定しているが、その旨が明示的ではないので、明示的にする
                                    &if_blk_ctx,
                                    {
                                        let mut new_element_location = element_location.clone();
                                        new_element_location.push(idx_of_ref as usize);
                                        &new_element_location.clone()
                                    },
                                    false,
                                ));
                                Some(node_id.clone())
                            } else {
                                None
                            };
                            let ref_text_node_id = match distance != 1 {
                                true => Some(nanoid!()),
                                false => None,
                            };
                            let (item_collection, dep_vars) = append_v_to_vars_in_html(
                                &remove_statement.item_collection.as_str(),
                                &varibale_names,
                            );
                            for_blocks_info.push(ForBlockInfo {
                                parent_id: node_id.clone(),
                                target_for_blk_id: remove_statement.child_uuid.clone(),
                                distance_to_next_elm: distance,
                                target_anchor_id,
                                node: deleted_node,
                                ref_text_node_id,
                                item_name: remove_statement.item_name.clone(),
                                item_index: remove_statement.item_index.clone(),
                                item_collection,
                                ctx_over_for: remove_statement.ctx_over_for.clone(),
                                ctx_under_for: remove_statement.ctx_under_for.clone(),
                                for_blk_id: remove_statement.block_id.clone(),
                                element_location: remove_statement.elm_loc.clone(),
                                dep_vars,
                            });
                        }
                        HtmlManipulation::RemoveChildForCustomComponent(remove_statement) => {
                            set_id_for_needed_elm(
                                element,
                                needed_ids,
                                &node_id,
                                &remove_statement.ctx,
                                element_location,
                                false,
                            );
                            let (_, _, distance, idx_of_ref) =
                                element.remove_child(&remove_statement.child_uuid, component_names);

                            // TODO:remove_childにまとめる
                            let target_anchor_id = if let Some(idx_of_ref) = idx_of_ref {
                                let node_id =
                                    &element.children[idx_of_ref as usize - 1].uuid.clone();
                                Some(set_id_for_needed_elm(
                                    match &mut element.children[idx_of_ref as usize - 1].content {
                                        NodeContent::Element(elm) => elm,
                                        _ => panic!("not element"),
                                    },
                                    needed_ids,
                                    node_id,
                                    // TODO: 親のctxを指定しているが、その旨が明示的ではないので、明示的にする
                                    &if_blk_ctx,
                                    {
                                        let mut new_element_location = element_location.clone();
                                        new_element_location.push(idx_of_ref as usize);
                                        &new_element_location.clone()
                                    },
                                    false,
                                ));
                                Some(node_id.clone())
                            } else {
                                None
                            };

                            custom_component_blocks_info.push(CustomComponentBlockInfo {
                                parent_id: node_id.clone(),
                                distance_to_next_elm: distance,
                                have_sibling_elm: count_of_siblings > 1,
                                target_anchor_id,
                                component_name: remove_statement.component_name.clone(),
                                args: ComponentArgs::new(&remove_statement.attributes),
                                ctx: remove_statement.ctx.clone(),
                                custom_component_block_id: UUID_GENERATOR.lock().unwrap().gen(),
                                element_location: remove_statement.elm_loc.clone(),
                                is_routing_component: ROUTER_COMPONENTS
                                    .into_iter()
                                    .any(|x| x == remove_statement.component_name),
                            });
                        }
                        HtmlManipulation::SetIdForReactiveContent(set_id) => {
                            set_id_for_needed_elm(
                                element,
                                needed_ids,
                                &node_id,
                                &set_id.ctx,
                                element_location,
                                false,
                            );
                            elm_and_var_relation.push(NodeAndReactiveInfo::ElmAndVariableRelation(
                                ElmAndVariableContentRelation {
                                    elm_id: node_id.clone(),
                                    dep_vars: set_id.depenent_vars.clone(),
                                    content_of_element: set_id.text.clone(),
                                    ctx: set_id.ctx.clone(),
                                    elm_loc: set_id.elm_loc.clone(),
                                },
                            ));
                        }
                        HtmlManipulation::RemoveChildTextNode(remove_text_node) => {
                            set_id_for_needed_elm(
                                element,
                                needed_ids,
                                &node_id,
                                &remove_text_node.ctx,
                                element_location,
                                false,
                            );

                            let (_, _, _, idx_of_ref) =
                                element.remove_child(&remove_text_node.child_uuid, component_names);
                            // TODO:remove_childにまとめる
                            let target_anchor_id = if let Some(idx_of_ref) = idx_of_ref {
                                let node_id =
                                    &element.children[idx_of_ref as usize - 1].uuid.clone();
                                Some(set_id_for_needed_elm(
                                    match &mut element.children[idx_of_ref as usize - 1].content {
                                        NodeContent::Element(elm) => elm,
                                        _ => panic!("not element"),
                                    },
                                    needed_ids,
                                    node_id,
                                    // TODO: ifブロックの親のctxを指定しているが、その旨が明示的ではないので、明示的にする
                                    &if_blk_ctx,
                                    {
                                        let mut new_element_location = element_location.clone();
                                        new_element_location.push(idx_of_ref as usize);
                                        &new_element_location.clone()
                                    },
                                    false,
                                ));
                                Some(node_id.clone())
                            } else {
                                None
                            };
                            txt_node_renderer.push(ManualRendererForTextNode {
                                parent_id: node_id.clone(),
                                text_node_id: remove_text_node.child_uuid.clone(),
                                content: remove_text_node.content.clone(),
                                ctx: remove_text_node.ctx.clone(),
                                element_location: remove_text_node.elm_loc.clone(),
                                target_anchor_id: target_anchor_id.clone(),
                            });

                            elm_and_var_relation.push(
                                NodeAndReactiveInfo::TextAndVariableContentRelation(
                                    TextAndVariableContentRelation {
                                        text_node_id: remove_text_node.child_uuid.clone(),
                                        dep_vars: remove_text_node.depenent_vars.clone(),
                                        content_of_element: remove_text_node.content.clone(),
                                        ctx: remove_text_node.ctx.clone(),
                                        elm_loc: remove_text_node.elm_loc.clone(),
                                    },
                                ),
                            );
                        }
                    }
                }
            }

            Ok(())
        }
        NodeContent::TextNode(text) => {
            let (dep_vars, _) = replace_text_with_reactive_value(text, varibale_names);
            if dep_vars.len() > 0 && count_of_siblings <= 1 {
                html_manipulators.push(HtmlManipulator {
                    target_uuid: parent_uuid.unwrap().clone(),
                    manipulations: HtmlManipulation::SetIdForReactiveContent(
                        SetIdToParentForChildReactiveText {
                            text: text.clone(),
                            depenent_vars: dep_vars,
                            ctx: if_blk_ctx.clone(),
                            elm_loc: element_location.clone(),
                        },
                    ),
                });
            } else if dep_vars.len() > 0 && count_of_siblings > 1 || txt_node_to_be_deleted {
                html_manipulators.push(HtmlManipulator {
                    target_uuid: parent_uuid.unwrap().clone(),
                    manipulations: HtmlManipulation::RemoveChildTextNode(RemoveChildTextNode {
                        depenent_vars: dep_vars,
                        ctx: if_blk_ctx.clone(),
                        elm_loc: element_location.clone(),
                        child_uuid: node_id,
                        content: text.clone(),
                    }),
                });
            }
            Ok(())
        }
        crate::orig_html_struct::structs::NodeContent::Comment(_) => Ok(()),
    }
}

fn set_id_for_needed_elm(
    element: &mut Element,
    ref_maps: &mut Vec<RefMap>,
    node_id: &String,
    ctx: &Vec<String>,
    elm_loc: &Vec<usize>,
    is_array: bool,
) -> String {
    // If a reference to the element already exists in refMap as NodeCreationMethod, return only the element UUID without setting an ID
    let node_creation_methods = ref_maps.iter().filter_map(|x| match x {
        RefMap::NodeCreationMethod(inner) => Some(inner),
        _ => None,
    });
    for method in node_creation_methods {
        if method.node_id == *node_id {
            return method.node_id.clone();
        }
    }

    let id_based_methods = ref_maps
        .iter()
        .filter_map(|x| match x {
            RefMap::IdBasedElementAccess(inner) => Some(inner),
            _ => None,
        })
        .collect::<Vec<_>>();

    if let Some(Some(id)) = element.attributes.get("id") {
        let id = if id_based_methods.iter().any(|x| x.id_name == id.clone()) {
            id.clone()
        } else {
            ref_maps.push(RefMap::IdBasedElementAccess(IdBasedElementAccess {
                id_name: id.clone(),
                to_delete: false,
                node_id: node_id.clone(),
                ctx: ctx.clone(),
                elm_loc: elm_loc.clone(),
                is_array,
            }));
            id.clone()
        };
        id
    } else {
        let new_id = UUID_GENERATOR.lock().unwrap().gen();
        element
            .attributes
            .insert("id".to_string(), Some(new_id.clone()));
        ref_maps.push(RefMap::IdBasedElementAccess(IdBasedElementAccess {
            id_name: new_id.clone(),
            to_delete: true,
            node_id: node_id.clone(),
            ctx: ctx.clone(),
            elm_loc: elm_loc.clone(),
            is_array,
        }));
        new_id
    }
}

fn create_space_for_ref_map(
    ref_maps: &mut Vec<RefMap>,
    id: &String,
    ctx: &Vec<String>,
    elm_loc: &Vec<usize>,
) {
    ref_maps.push(RefMap::NodeCreationMethod(NodeCreationMethod {
        node_id: id.clone(),
        ctx: ctx.clone(),
        elm_loc: elm_loc.clone(),
    }));
}

// FIXME:カッコが複数でも、escapeTextは各バインディングに1つだけでいい
// 具体例:
// 現在:${$$lunasEscapeHtml(count.v+count.v)} count ${$$lunasEscapeHtml(count)} ${$$lunasEscapeHtml( count + count )}
// 将来的:${$$lunasEscapeHtml(`${count.v+count.v} count ${count} ${ count + count }`)}

// カッコが1つだけの場合、その部分のみをエスケープする
// Give: <div>    ${count} </div>
// Want: <div>    ${$$lunasEscapeHtml(count)} </div>
// TODO: count_of_bindingsの返却をやめる
fn replace_text_with_reactive_value(
    code: &mut String,
    variables: &Vec<String>,
) -> (Vec<String>, u32) {
    let mut count_of_bindings = 0;

    let start_tag = "${";
    let end_tag = "}";
    let mut new_code = String::new();
    let mut depending_vars = vec![];
    let mut last_end = 0;

    while let Some(start) = code[last_end..].find(start_tag) {
        count_of_bindings += 1;
        let start = start + last_end;
        if let Some(end) = code[start..].find(end_tag) {
            let end = end + start;
            let pre_bracket = &code[last_end..start];
            let in_bracket = &code[start + 2..end];
            let _post_bracket = &code[end + 1..];

            new_code.push_str(pre_bracket);
            new_code.push_str(start_tag);
            let (output, dep_vars) = append_v_to_vars_in_html(in_bracket, variables);
            new_code.push_str(&escape_html(&output));
            new_code.push_str(end_tag);

            last_end = end + 1;

            depending_vars.extend(dep_vars);
        }
    }

    new_code.push_str(&code[last_end..]);
    *code = new_code;
    (depending_vars, count_of_bindings)
}

pub fn create_lunas_internal_component_statement(
    elm: &Element,
    generation_func_name: &str,
) -> String {
    let mut code = String::new();
    code.push_str(format!("{}(`", generation_func_name).as_str());
    for child in &elm.children {
        code.push_str(&child.to_string());
    }
    code.push_str("`, \"");
    code.push_str(&elm.tag_name);
    code.push_str("\"");
    let attrs = elm.attributes_without_meta();
    if attrs.len() > 0 {
        code.push_str(", {");
        for (key, value) in attrs.iter() {
            let js_value = match value {
                Some(value) => format!("`{}`", value),
                None => "null".to_string(),
            };
            code.push_str(&format!("\"{}\": {},", key, js_value));
        }
        code.push('}');
    }
    code.push_str(")");
    code
}

// TODO: テストを別ファイルに移動する
#[cfg(test)]
mod tests {
    use super::replace_text_with_reactive_value;

    #[test]
    fn exploration() {
        let code = "$$lunasEscapeHtml(count2.v+count.v)";
        let mut code = code.to_string();
        replace_text_with_reactive_value(
            &mut code,
            &vec!["count".to_string(), "count2".to_string()],
        );
        assert_eq!(code, "$$lunasEscapeHtml(count2.v+count.v)");
    }

    #[test]
    fn exploration2() {
        let code = "$$lunasEscapeHtml( count2.v + count.v )";
        let mut code = code.to_string();
        replace_text_with_reactive_value(
            &mut code,
            &vec!["count".to_string(), "count2".to_string()],
        );
        assert_eq!(code, "$$lunasEscapeHtml( count2.v + count.v )");
    }

    #[test]
    fn exploration3() {
        let code = "${interval==null?'start':'clear'}";
        let mut code = code.to_string();
        replace_text_with_reactive_value(&mut code, &vec!["interval".to_string()]);
        assert_eq!(
            code,
            "${$$lunasEscapeHtml(interval.v == null ? 'start' : 'clear')}"
        );
    }
}

fn escape_html(s: &str) -> String {
    format!("$$lunasEscapeHtml({})", s)
}

fn find_reactive_attr_from_id<'a>(
    id: &str,
    reactive_attrs: &'a mut Vec<NodeAndReactiveInfo>,
) -> Option<&'a mut ElmAndReactiveAttributeRelation> {
    reactive_attrs
        .iter_mut()
        .filter_map(|elm_and_var_relation| {
            if let NodeAndReactiveInfo::ElmAndReactiveAttributeRelation(elm_and_var_relation) =
                elm_and_var_relation
            {
                Some(elm_and_var_relation)
            } else {
                None
            }
        })
        .find(|x| x.elm_id == id)
}

use crate::structs::transform_info::TextNodeRendererGroup;

use super::utils::create_indent;

pub fn gen_create_anchor_statements(
    text_node_renderer: &TextNodeRendererGroup,
    ctx_condition: &Vec<String>,
    ref_node_ids: &mut Vec<String>,
    under_for: bool,
) -> Option<String> {
    let ref_node_ids_count_before_creating_anchors = ref_node_ids.len();
    let mut create_anchor_statements = vec![];
    let mut iter = text_node_renderer.renderers.iter().peekable();
    let mut amount_of_next_elm = 1;
    while let Some(render) = iter.next() {
        match render {
            crate::structs::transform_info::TextNodeRenderer::ManualRenderer(txt_renderer) => {
                if &txt_renderer.ctx != ctx_condition {
                    continue;
                }
                let anchor_idx = match &txt_renderer.target_anchor_id {
                    Some(anchor_id) => {
                        let reference_node_idx = ref_node_ids.iter().position(|id| id == anchor_id);
                        match reference_node_idx {
                            Some(idx) => match under_for {
                                true => format!("[{}, ...$$livonForIndices]", idx),
                                false => idx.to_string(),
                            },
                            None => "null".to_string(),
                        }
                    }
                    None => "null".to_string(),
                };
                let parent_node_idx = {
                    let parent_node_idx = ref_node_ids
                        .iter()
                        .position(|id| id == &txt_renderer.parent_id)
                        .unwrap()
                        .to_string();
                    match under_for {
                        true => format!("[{}, ...$$livonForIndices]", parent_node_idx),
                        false => parent_node_idx,
                    }
                };
                let create_anchor_statement = format!(
                    "[1, {}, {}, `{}`],",
                    &parent_node_idx,
                    &anchor_idx,
                    &txt_renderer.content.trim(),
                );
                ref_node_ids.push(txt_renderer.text_node_id.clone());
                create_anchor_statements.push(create_anchor_statement);
            }
            _ => {
                let next_render = iter.peek();
                let (distance_to_next_elm, ctx, target_anchor_id, block_id, parent_id) =
                    render.get_empty_text_node_info();
                if distance_to_next_elm <= 1 {
                    continue;
                }
                if &ctx != ctx_condition {
                    continue;
                }
                if let Some(next_renderer) = next_render {
                    if render.is_next_elm_the_same_anchor(next_renderer) {
                        ref_node_ids.push(format!("{}-anchor", block_id));
                        amount_of_next_elm += 1;
                        continue;
                    }
                }

                let anchor_node_idx = match &target_anchor_id {
                    Some(anchor_id) => {
                        let reference_node_idx =
                            ref_node_ids.iter().position(|id| id == anchor_id).unwrap();
                        let reference_node_idx = reference_node_idx.to_string();
                        match under_for {
                            true => format!("[{}, ...$$livonForIndices]", reference_node_idx),
                            false => reference_node_idx,
                        }
                    }
                    None => "null".to_string(),
                };
                let parent_node_idx = {
                    let parent_node_idx = ref_node_ids
                        .iter()
                        .position(|id| id == &parent_id)
                        .unwrap()
                        .to_string();
                    match under_for {
                        true => format!("[{}, ...$$livonForIndices]", parent_node_idx),
                        false => parent_node_idx,
                    }
                };

                ref_node_ids
                    .iter()
                    .position(|id| id == &parent_id)
                    .unwrap()
                    .to_string();
                let create_anchor_statement = format!(
                    "[{}, {}, {}],",
                    amount_of_next_elm, parent_node_idx, anchor_node_idx
                );
                create_anchor_statements.push(create_anchor_statement);
                ref_node_ids.push(format!("{}-anchor", block_id));
                amount_of_next_elm = 1;
            }
        }
    }

    let anchor_offset = match under_for {
        true => {
            format!(
                ", [{}, ...$$livonForIndices]",
                ref_node_ids_count_before_creating_anchors.to_string()
            )
        }
        false => match ref_node_ids_count_before_creating_anchors == 0 {
            true => "".to_string(),
            false => format!(
                ", {}",
                ref_node_ids_count_before_creating_anchors.to_string()
            ),
        },
    };

    if ref_node_ids_count_before_creating_anchors == 0 {
        "".to_string()
    } else {
        format!(
            ", {}",
            ref_node_ids_count_before_creating_anchors
                .to_string()
                .as_str()
        )
    };

    if create_anchor_statements.is_empty() {
        return None;
    }

    Some(
        format!(
            r#"$$livonInsertTextNodes([
{}
]{});"#,
            create_indent(create_anchor_statements.join("\n").as_str()),
            anchor_offset
        )
        .to_string(),
    )
}

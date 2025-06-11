use crate::{
    structs::{
        js_analyze::JsFunctionDeps,
        transform_info::{AddStringToPosition, TransformInfo},
    },
    transformers::utils_swc::transform_ts_to_js,
};
use serde_json::{to_value, Value};

/// Applies a sequence of TransformInfo operations to the given script.
/// Resolves overlapping and ensures valid UTF-8 boundaries by adjusting non-boundary indices.

/// Applies transforms using cumulative offset correction to maintain byte-accurate positions.
pub fn add_or_remove_strings_to_script(
    mut transforms: Vec<TransformInfo>,
    script: &str,
) -> (String, String) {
    // Priority: Replace(0), Remove(1), Add(2), Move(3)
    fn priority(tr: &TransformInfo) -> u8 {
        match tr {
            TransformInfo::ReplaceText(_) => 0,
            TransformInfo::RemoveStatement(_) => 1,
            TransformInfo::AddStringToPosition(_) => 2,
            TransformInfo::MoveToTheEnd(_) => 3,
        }
    }
    // Sort by sort_order (AddStringToPosition), then by type priority, then by original byte position
    transforms.sort_by(|a, b| {
        // 1) sort_order for AddStringToPosition (others treated as 0)
        let sa = if let TransformInfo::AddStringToPosition(ad) = a {
            ad.sort_order as usize
        } else {
            0
        };
        let sb = if let TransformInfo::AddStringToPosition(ad) = b {
            ad.sort_order as usize
        } else {
            0
        };
        sa.cmp(&sb)
            // 2) type priority
            .then_with(|| {
                let pa = priority(a);
                let pb = priority(b);
                pa.cmp(&pb)
            })
            // 3) original byte position
            .then_with(|| {
                let posa = match a {
                    TransformInfo::ReplaceText(rt) => rt.start_position as usize,
                    TransformInfo::RemoveStatement(rs) => rs.start_position as usize,
                    TransformInfo::AddStringToPosition(ad) => ad.position as usize,
                    TransformInfo::MoveToTheEnd(me) => me.start_position as usize,
                };
                let posb = match b {
                    TransformInfo::ReplaceText(rt) => rt.start_position as usize,
                    TransformInfo::RemoveStatement(rs) => rs.start_position as usize,
                    TransformInfo::AddStringToPosition(ad) => ad.position as usize,
                    TransformInfo::MoveToTheEnd(me) => me.start_position as usize,
                };
                posa.cmp(&posb)
            })
    });

    let mut result = script.to_string();
    let mut end_str = "".to_string();
    // Track cumulative byte offset change: Vec of (original_pos, delta_bytes)
    let mut deltas: Vec<(usize, isize)> = Vec::new();

    // Compute adjusted position given original pos
    let adjust = |orig: usize, deltas: &[(usize, isize)]| -> usize {
        let total: isize = deltas
            .iter()
            .filter(|(p, _)| *p <= orig)
            .map(|(_, d)| *d)
            .sum();
        ((orig as isize) + total) as usize
    };

    for tr in transforms {
        match tr {
            TransformInfo::ReplaceText(rt) => {
                let orig_start = rt.start_position as usize;
                let orig_end = rt.end_position as usize;
                let start = adjust(orig_start, &deltas);
                let end = adjust(orig_end, &deltas);
                if start <= end && end <= result.len() {
                    let new_len = rt.string.len();
                    let old_len = end - start;
                    result.replace_range(start..end, &rt.string);
                    deltas.push((orig_end, new_len as isize - old_len as isize));
                }
            }
            TransformInfo::RemoveStatement(rs) => {
                let orig_start = rs.start_position as usize;
                let orig_end = rs.end_position as usize;
                let start = adjust(orig_start, &deltas);
                let end = adjust(orig_end, &deltas);
                if start <= end && end <= result.len() {
                    let old_len = end - start;
                    result.replace_range(start..end, "");
                    deltas.push((orig_end, -(old_len as isize)));
                }
            }
            TransformInfo::AddStringToPosition(ad) => {
                let orig = ad.position as usize;
                let pos = adjust(orig, &deltas);
                if pos <= result.len() {
                    let add_len = ad.string.len();
                    result.insert_str(pos, &ad.string);
                    deltas.push((orig, add_len as isize));
                }
            }
            TransformInfo::MoveToTheEnd(me) => {
                let orig_start = me.start_position as usize;
                let orig_end = me.end_position as usize;
                let start = adjust(orig_start, &deltas);
                let end = adjust(orig_end, &deltas);
                if start <= end && end <= result.len() {
                    let slice = result[start..end].to_string();
                    result.replace_range(start..end, "");
                    end_str.push_str("\n");
                    end_str.push_str(&slice);
                    let len = slice.len();
                    // remove length at start, then add at end
                    deltas.push((orig_end, -(len as isize)));
                    // end position is original end: now appended after end of file, delta not needed for future
                }
            }
        }
    }
    (result, end_str)
}

use super::{
    js_utils::search_json,
    utils_swc::{parse_expr_with_swc, parse_module_with_swc},
};

pub fn append_v_to_vars_in_html(
    input_ts: &str,
    variables: &Vec<String>,
    variable_names_to_add_value_accessor: &Vec<String>,
    func_deps: &Vec<JsFunctionDeps>,
    is_expr: bool,
) -> Result<(String, Vec<String>), String> {
    // 1) Transpile TS to JS
    let js = transform_ts_to_js(input_ts).map_err(|e| e.to_string())?;

    // 2) Parse JS code into a JSON AST
    let parsed_json = if is_expr {
        let expr = parse_expr_with_swc(&js).map_err(|e| e.to_string())?;
        to_value(expr).unwrap()
    } else {
        let module = parse_module_with_swc(&js).map_err(|e| e.to_string())?;
        to_value(module).unwrap()
    };

    // 3) Prepare buffers for search_json output
    let mut positions = Vec::new();
    let mut imports = Vec::new(); // unused here
    let mut depending_vars = Vec::new();
    let mut depending_funcs = Vec::new();

    // 4) Invoke search_json to collect positions and dependent identifiers
    search_json(
        &parsed_json,
        js.as_str(),
        &variables,
        &variable_names_to_add_value_accessor,
        &vec![],
        false,
        &mut positions,
        &mut imports,
        &mut depending_vars,
        &mut depending_funcs,
    );

    // 5) Apply transformations to the original input
    let (modified_string, _) = add_or_remove_strings_to_script(positions, &js);

    // 6) Gather vars from function dependencies that were actually invoked
    let func_dep_vars = func_deps
        .iter()
        .filter_map(|func| {
            if depending_funcs.contains(&func.name) {
                Some(func.depending_vars.clone())
            } else {
                None
            }
        })
        .flatten()
        .collect::<Vec<String>>();

    // 7) Merge and deduplicate all depending variable names
    let all_depending_values = depending_vars
        .into_iter()
        .chain(func_dep_vars.into_iter())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<String>>();

    Ok((modified_string, all_depending_values))
}

pub fn convert_non_reactive_to_obj(input: &str, variables: &Vec<String>) -> Result<String, String> {
    let parsed = parse_module_with_swc(&input.to_string()).map_err(|e| e.to_string())?;
    let parsed_json = serde_json::to_value(&parsed).unwrap();
    let positions = find_non_reactives(&parsed_json, &variables);
    let (modified_string, _) = add_or_remove_strings_to_script(positions, &input.to_string());
    Ok(modified_string)
}

pub fn find_non_reactives(json: &Value, variables: &Vec<String>) -> Vec<TransformInfo> {
    let mut positions = vec![];

    // When obj.type is ExpressionStatement
    // AND obj.expression.type is Identifier
    // AND obj.expression.value is in variables (= obj.expression.value is not reactive variable)
    // mark them as non-reactive and make them object

    if let Value::Object(obj) = json {
        if obj.contains_key("type") && obj["type"] == Value::String("ExpressionStatement".into()) {
            let expression = obj.get("expression").unwrap();
            let expression_type = expression.get("type").unwrap();
            let is_expression_type_identifier =
                *expression_type == Value::String("Identifier".into());
            let identifier_value_is_in_variables = if is_expression_type_identifier {
                let value = expression.get("value").unwrap();
                variables.iter().any(|e| e == value.as_str().unwrap())
            } else {
                false
            };

            if !(is_expression_type_identifier && identifier_value_is_in_variables) {
                let span = expression.get("span").unwrap();
                let end = span.get("end").unwrap();
                let start = span.get("start").unwrap();
                positions.push(TransformInfo::AddStringToPosition(AddStringToPosition {
                    position: (end.as_u64().unwrap() - 1) as u32,
                    string: ")".to_string(),
                    sort_order: 1,
                }));
                positions.push(TransformInfo::AddStringToPosition(AddStringToPosition {
                    position: (start.as_u64().unwrap() - 1) as u32,
                    string: "$$lunasCreateNonReactive(".to_string(),
                    sort_order: 1,
                }));

                return positions;
            }
        }
        for (_key, value) in obj {
            let result_positions = find_non_reactives(value, variables);
            positions.extend(result_positions);
        }
    } else if let Value::Array(arr) = json {
        for value in arr {
            let result_positions = find_non_reactives(value, variables);
            positions.extend(result_positions);
        }
    }
    positions
}

use crate::structs::{
    js_analyze::JsFunctionDeps,
    js_utils::JsSearchParent,
    transform_info::{AddStringToPosition, TransformInfo},
};
use rand::{rngs::StdRng, SeedableRng};
use serde_json::Value;
use std::{env, sync::Mutex};

// TODO: 綺麗な実装にする
pub fn add_or_remove_strings_to_script(
    position_and_strs: Vec<TransformInfo>,
    script: &String,
) -> String {
    let mut transformers = position_and_strs.clone();
    transformers.sort_by(|a, b| {
        let a = match a {
            TransformInfo::AddStringToPosition(a) => a.sort_order,
            TransformInfo::RemoveStatement(_) => 0,
            TransformInfo::ReplaceText(_) => 0,
        };
        let b = match b {
            TransformInfo::AddStringToPosition(b) => b.sort_order,
            TransformInfo::RemoveStatement(_) => 0,
            TransformInfo::ReplaceText(_) => 0,
        };
        a.cmp(&b)
    });
    transformers.sort_by(|a, b| {
        let a = match a {
            TransformInfo::AddStringToPosition(a) => a.position,
            TransformInfo::RemoveStatement(a) => a.start_position,
            TransformInfo::ReplaceText(a) => a.start_position,
        };
        let b = match b {
            TransformInfo::AddStringToPosition(b) => b.position,
            TransformInfo::RemoveStatement(b) => b.start_position,
            TransformInfo::ReplaceText(b) => b.start_position,
        };
        a.cmp(&b)
    });
    let mut result = String::new();
    let mut last_position = 0;
    for transform in transformers {
        match transform {
            TransformInfo::AddStringToPosition(add) => {
                result.push_str(&script[last_position..add.position as usize]);
                result.push_str(&add.string);
                last_position = add.position as usize;
            }
            TransformInfo::RemoveStatement(remove) => {
                result.push_str(&script[last_position..remove.start_position as usize]);
                last_position = remove.end_position as usize;
            }
            TransformInfo::ReplaceText(replace) => {
                result.push_str(&script[last_position..replace.start_position as usize]);
                result.push_str(&replace.string);
                last_position = replace.end_position as usize;
            }
        }
    }
    result.push_str(&script[last_position..]);
    return result;
}

use rand::seq::SliceRandom;

use super::{js_utils::search_json, utils_swc::parse_with_swc};

lazy_static! {
    pub static ref UUID_GENERATOR: Mutex<UuidGenerator> = Mutex::new(UuidGenerator::new());
}

pub struct UuidGenerator {
    seed: u8,
}

impl UuidGenerator {
    fn new() -> UuidGenerator {
        UuidGenerator { seed: 0 }
    }

    pub fn gen(&mut self) -> String {
        if is_testgen() {
            let seed = [self.seed; 32]; // ここに適当なシード値を設定します。
            let mut rng: StdRng = SeedableRng::from_seed(seed);

            let alphabet: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_$";
            let size = 21;

            let id: String = (0..size)
                .map(|_| {
                    let random_char = alphabet.choose(&mut rng).unwrap();
                    *random_char as char
                })
                .collect();
            self.seed = self.seed + 1;
            id
        } else {
            let alphabet: [char; 53] = [
                'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
                'q', 'r', 's', 't', 'v', 'u', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F',
                'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'V', 'U',
                'W', 'X', 'Y', 'Z', '$',
            ];
            nanoid::nanoid!(10, &alphabet)
        }
    }
}

// FIXME: test env is no longer needed
fn is_testgen() -> bool {
    match env::var("LIVON_TEST") {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn append_v_to_vars_in_html(
    input: &str,
    variables: &Vec<String>,
    func_deps: &Vec<JsFunctionDeps>,
) -> (String, Vec<String>) {
    let parsed = parse_with_swc(&input.to_string());

    let parsed_json = serde_json::to_value(&parsed).unwrap();

    let (positions, _, depending_vars, depending_funcs) = search_json(
        &parsed_json,
        &input.to_string(),
        &variables,
        None,
        JsSearchParent::ParentIsArray,
    );

    let modified_string = add_or_remove_strings_to_script(positions, &input.to_string());

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

    let all_depending_values = depending_vars
        .iter()
        .chain(func_dep_vars.iter())
        .cloned()
        .collect::<std::collections::HashSet<String>>()
        .into_iter()
        .collect::<Vec<String>>();

    (modified_string, all_depending_values)
}

pub fn convert_non_reactive_to_obj(input: &str, variables: &Vec<String>) -> String {
    let parsed = parse_with_swc(&input.to_string());
    let parsed_json = serde_json::to_value(&parsed).unwrap();
    let positions = find_non_reactives(&parsed_json, &variables);
    let modified_string = add_or_remove_strings_to_script(positions, &input.to_string());
    modified_string
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
                    string: "$$livonCreateNonReactive(".to_string(),
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

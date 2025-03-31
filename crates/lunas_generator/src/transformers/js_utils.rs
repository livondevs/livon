use std::vec;

use lunas_parser::DetailedBlock;
use serde_json::Value;

use crate::{
    ast_analyzer::function_analyzer::analyze_ast,
    structs::{
        js_analyze::{JsFunctionDeps, Tidy},
        js_utils::JsSearchParent,
        transform_info::{
            AddStringToPosition, RemoveStatement, ReplaceText, TransformInfo,
            VariableNameAndAssignedNumber,
        },
    },
};

use super::utils::add_or_remove_strings_to_script;

pub fn analyze_js(
    blocks: &DetailedBlock,
    initial_num: u32,
    variables: &mut Vec<VariableNameAndAssignedNumber>,
) -> (Vec<String>, Vec<String>, String, Vec<JsFunctionDeps>) {
    if let Some(js_block) = &blocks.detailed_language_blocks.js {
        let mut positions = vec![];
        let mut imports = vec![];
        // find all variable declarations
        let str_positions = find_variable_declarations(&js_block.ast, initial_num, variables);
        // add all variable declarations to positions to add custom variable declaration function
        positions.extend(str_positions);
        let variable_names = variables.iter().map(|v| v.name.clone()).collect();
        let (position_result, import_result, _, _) = search_json(
            &js_block.ast,
            &js_block.raw,
            &variable_names,
            Some(&imports),
            JsSearchParent::NoneValue,
        );

        let mut functions_and_deps = analyze_ast(&js_block.ast, &variable_names);
        functions_and_deps.tidy();

        positions.extend(position_result);
        imports.extend(import_result);
        let output = add_or_remove_strings_to_script(positions, &js_block.raw);
        (variable_names, imports, output, functions_and_deps)
    } else {
        let variable_names = variables
            .iter()
            .map(|v| v.name.clone())
            .collect::<Vec<String>>();
        (variable_names, vec![], "".to_string(), vec![])
    }
}

// Finds all variable declarations in a javascript file and returns a vector of VariableNameAndAssignedNumber structs
fn find_variable_declarations(
    json: &Value,
    initial_num: u32,
    variables: &mut Vec<VariableNameAndAssignedNumber>,
) -> vec::Vec<TransformInfo> {
    if let Some(Value::Array(body)) = json.get("body") {
        let mut str_positions = vec![];
        let mut num_generator = power_of_two_generator(initial_num);
        for body_item in body {
            if Some(&Value::String("VariableDeclaration".to_string())) == body_item.get("type") {
                if let Some(Value::Array(declarations)) = body_item.get("declarations") {
                    for declaration in declarations {
                        let name = if let Some(Value::Object(id)) = declaration.get("id") {
                            if let Some(Value::String(name)) = id.get("value") {
                                Some(name.to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                        // get span
                        let start_and_end =
                            if let Some(Value::Object(init)) = declaration.get("init") {
                                if let Some(Value::Object(span)) = init.get("span") {
                                    if let Some(Value::Number(end)) = span.get("end") {
                                        if let Some(Value::Number(start)) = span.get("start") {
                                            Some((start, end))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            };
                        if let Some(name) = name {
                            if let Some((start, end)) = start_and_end {
                                let variable_num = num_generator();
                                variables.push(VariableNameAndAssignedNumber {
                                    name,
                                    assignment: variable_num,
                                });
                                str_positions.push(TransformInfo::AddStringToPosition(
                                    AddStringToPosition {
                                        position: (start.as_u64().unwrap() - 1) as u32,
                                        string: "$$lunasReactive(".to_string(),
                                    },
                                ));
                                str_positions.push(TransformInfo::AddStringToPosition(
                                    AddStringToPosition {
                                        position: (end.as_u64().unwrap() - 1) as u32,
                                        string: format!(")"),
                                    },
                                ));
                            }
                        }
                    }
                }
            }
        }
        str_positions
    } else {
        vec![]
    }
}

fn power_of_two_generator(init: u32) -> impl FnMut() -> u32 {
    let mut count = init;
    move || -> u32 {
        let result = 2u32.pow(count);
        count += 1;
        result
    }
}

// TODO: (P5) Use mutable references for the arguments instead of returning them
pub fn search_json(
    json: &serde_json::Value,
    raw_js: &String,
    variables: &Vec<String>,
    // FIXME: imports are unused
    imports: Option<&Vec<String>>,
    parent: JsSearchParent,
) -> (Vec<TransformInfo>, Vec<String>, Vec<String>, Vec<String>) {
    use serde_json::Value;

    if let Value::Object(obj) = json {
        if obj.contains_key("type") && obj["type"] == Value::String("Identifier".into()) {
            if (parent.clone().is_some()
                && parent.clone().unwrap().get("type").as_ref()
                    != Some(&&Value::String("VariableDeclarator".to_string())))
                || parent.clone().eq(&JsSearchParent::ParentIsArray)
            {
                if let Some(Value::String(variable_name)) = obj.get("value") {
                    if variables.iter().any(|e| e == variable_name) {
                        if let Some(Value::Object(span)) = obj.get("span") {
                            if let Some(Value::Number(end)) = span.get("end") {
                                return (
                                    vec![TransformInfo::AddStringToPosition(AddStringToPosition {
                                        position: (end.as_u64().unwrap() - 1) as u32,
                                        string: ".v".to_string(),
                                    })],
                                    vec![],
                                    vec![variable_name.clone()],
                                    vec![], // 関数名はここでは対象外
                                );
                            }
                        }
                    }
                }
            }
            return (vec![], vec![], vec![], vec![]);
        } else if obj.contains_key("type")
            && obj["type"] == Value::String("ImportDeclaration".into())
        {
            let trim_end = obj["span"]["end"].as_u64().unwrap() as u32;
            let mut remove_end = trim_end;
            if raw_js.chars().nth(trim_end as usize).unwrap() == '\n' {
                remove_end += 1;
            }

            return (
                vec![TransformInfo::RemoveStatement(RemoveStatement {
                    start_position: obj["span"]["start"].as_u64().unwrap() as u32 - 1,
                    end_position: remove_end,
                })],
                vec![raw_js
                    .chars()
                    .skip(obj["span"]["start"].as_u64().unwrap() as usize - 1)
                    .take(trim_end as usize - obj["span"]["start"].as_u64().unwrap() as usize)
                    .collect()],
                vec![],
                vec![],
            );
        } else if obj.contains_key("type")
            && obj["type"] == Value::String("MemberExpression".into())
        {
            if let Some(object) = obj.get("object") {
                if let Some(property) = obj.get("property") {
                    let is_target_property_router = if let Value::Object(property) = property {
                        if let Some(Value::String(property_value)) = property.get("value") {
                            property_value == "router"
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    let is_target_property_after_mount = if let Value::Object(property) = property {
                        if let Some(Value::String(property_value)) = property.get("value") {
                            property_value == "afterMount"
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    let is_target_property_after_unmount = if let Value::Object(property) = property
                    {
                        if let Some(Value::String(property_value)) = property.get("value") {
                            property_value == "afterUnmount"
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    let is_target_object = if let Value::Object(object) = object {
                        if let Some(Value::String(object_value)) = object.get("value") {
                            object_value == "Lunas"
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if is_target_object && is_target_property_router {
                        if let Some(Value::Object(span)) = obj.get("span") {
                            let start = span["start"].as_u64().unwrap() as u32;
                            let end = span["end"].as_u64().unwrap() as u32;
                            return (
                                vec![TransformInfo::ReplaceText(ReplaceText {
                                    start_position: start - 1,
                                    end_position: end - 1,
                                    string: "$$lunasRouter".to_string(),
                                })],
                                vec![],
                                vec![],
                                vec![],
                            );
                        }
                    } else if is_target_object && is_target_property_after_mount {
                        if let Some(Value::Object(span)) = obj.get("span") {
                            let start = span["start"].as_u64().unwrap() as u32;
                            let end = span["end"].as_u64().unwrap() as u32;
                            return (
                                vec![TransformInfo::ReplaceText(ReplaceText {
                                    start_position: start - 1,
                                    end_position: end - 1,
                                    string: "$$lunasAfterMount".to_string(),
                                })],
                                vec![],
                                vec![],
                            );
                        }
                    } else if is_target_object && is_target_property_after_unmount {
                        if let Some(Value::Object(span)) = obj.get("span") {
                            let start = span["start"].as_u64().unwrap() as u32;
                            let end = span["end"].as_u64().unwrap() as u32;
                            return (
                                vec![TransformInfo::ReplaceText(ReplaceText {
                                    start_position: start - 1,
                                    end_position: end - 1,
                                    string: "$$lunasAfterUnmount".to_string(),
                                })],
                                vec![],
                                vec![],
                            );
                        }
                    }
                }
            }
        } else if obj.contains_key("type") && obj["type"] == Value::String("CallExpression".into())
        {
            let mut functions_found = vec![];
            if let Some(callee) = obj.get("callee") {
                if let Value::Object(callee_obj) = callee {
                    if callee_obj.get("type") == Some(&Value::String("Identifier".to_string())) {
                        if let Some(Value::String(func_name)) = callee_obj.get("value") {
                            functions_found.push(func_name.clone());
                        }
                    }
                }
            }
            let mut trans_tmp = vec![];
            let mut import_tmp = vec![];
            let mut dep_vars_tmp = vec![];
            let mut funcs_tmp = functions_found;
            for (_key, value) in obj {
                let (trans_res, import_res, dep_vars, funcs) = search_json(
                    value,
                    raw_js,
                    variables,
                    imports,
                    JsSearchParent::MapValue(&obj),
                );
                trans_tmp.extend(trans_res);
                import_tmp.extend(import_res);
                dep_vars_tmp.extend(dep_vars);
                funcs_tmp.extend(funcs);
            }
            return (trans_tmp, import_tmp, dep_vars_tmp, funcs_tmp);
        }

        let mut trans_tmp = vec![];
        let mut import_tmp = vec![];
        let mut dep_vars_tmp = vec![];
        let mut funcs_tmp = vec![];
        for (_key, value) in obj {
            let (trans_res, import_res, dep_vars, funcs) = search_json(
                value,
                raw_js,
                variables,
                imports,
                JsSearchParent::MapValue(&obj),
            );
            trans_tmp.extend(trans_res);
            import_tmp.extend(import_res);
            dep_vars_tmp.extend(dep_vars);
            funcs_tmp.extend(funcs);
        }
        return (trans_tmp, import_tmp, dep_vars_tmp, funcs_tmp);
    } else if let Value::Array(arr) = json {
        let mut trans_tmp = vec![];
        let mut import_tmp = vec![];
        let mut dep_vars_tmp = vec![];
        let mut funcs_tmp = vec![];
        for child_value in arr {
            let (trans_res, import_res, dep_vars, funcs) = search_json(
                child_value,
                raw_js,
                variables,
                imports,
                JsSearchParent::ParentIsArray,
            );
            trans_tmp.extend(trans_res);
            import_tmp.extend(import_res);
            dep_vars_tmp.extend(dep_vars);
            funcs_tmp.extend(funcs);
        }
        return (trans_tmp, import_tmp, dep_vars_tmp, funcs_tmp);
    }
    return (vec![], vec![], vec![], vec![]);
}

use std::vec;

use lunas_parser::DetailedBlock;
use num_bigint::BigUint;
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
) -> (Vec<String>, String, Vec<JsFunctionDeps>, Vec<String>) {
    if let Some(js_block) = &blocks.detailed_language_blocks.js {
        let mut positions = vec![];
        let mut imports = vec![];
        // find all variable declarations
        let (str_positions, mut num_gen) =
            find_variable_declarations(&js_block.ast, initial_num, variables, false);
        // add all variable declarations to positions to add custom variable declaration function
        positions.extend(str_positions);
        let lun_imports = find_luns_imports(&js_block.ast);
        for lun_import in &lun_imports {
            variables.push(VariableNameAndAssignedNumber {
                name: lun_import.clone(),
                assignment: num_gen.as_mut().unwrap()(),
            })
        }
        let variable_names = variables.iter().map(|v| v.name.clone()).collect();
        let (position_result, import_result, _, _) = search_json(
            &js_block.ast,
            &js_block.raw,
            &variable_names,
            Some(&imports),
            JsSearchParent::NoneValue,
            true,
        );

        let mut functions_and_deps = analyze_ast(&js_block.ast, &variable_names);
        functions_and_deps.tidy();

        positions.extend(position_result);
        imports.extend(import_result);
        let output = add_or_remove_strings_to_script(positions, &js_block.raw);
        (imports, output, functions_and_deps, lun_imports)
    } else {
        (vec![], "".to_string(), vec![], vec![])
    }
}

pub fn load_lunas_script_variables(variables: &Vec<String>) -> String {
    format!("$$lunasSetImportVars([{}])", variables.join(", "))
}

// Finds all variable declarations in a JavaScript AST (including export declarations)
// and returns a tuple containing a vector of TransformInfo and an optional number generator function.
pub fn find_variable_declarations(
    json: &Value,
    initial_num: u32,
    variables: &mut Vec<VariableNameAndAssignedNumber>,
    lunas_script: bool,
) -> (Vec<TransformInfo>, Option<impl FnMut() -> BigUint>) {
    if let Some(Value::Array(body)) = json.get("body") {
        let mut str_positions = Vec::new();
        let mut num_generator = power_of_two_generator(initial_num);

        for body_item in body {
            // Determine if the item is a VariableDeclaration or an ExportDeclaration containing a VariableDeclaration
            let maybe_decl = match body_item.get("type") {
                // Direct variable declaration
                Some(Value::String(t)) if t == "VariableDeclaration" => Some(body_item),
                // Export declaration wrapping a variable declaration
                Some(Value::String(t)) if t == "ExportDeclaration" => body_item.get("declaration"),
                _ => None,
            };

            if let Some(Value::Object(decl_obj)) = maybe_decl {
                if let Some(Value::Array(declarations)) = decl_obj.get("declarations") {
                    for declaration in declarations {
                        // Extract the variable name
                        let name = declaration
                            .get("id")
                            .and_then(|id| id.get("value"))
                            .and_then(Value::as_str)
                            .map(String::from);

                        // Extract start and end positions from the initialization span
                        let start_end = declaration
                            .get("init")
                            .and_then(|init| init.get("span"))
                            .and_then(Value::as_object)
                            .and_then(|span| {
                                span.get("start")
                                    .and_then(Value::as_u64)
                                    .zip(span.get("end").and_then(Value::as_u64))
                            })
                            .map(|(s, e)| (s as u32, e as u32));

                        if let (Some(name), Some((start, end))) = (name, start_end) {
                            // Generate a unique number for this variable
                            let variable_num = num_generator();
                            variables.push(VariableNameAndAssignedNumber {
                                name: name.clone(),
                                assignment: variable_num.clone(),
                            });

                            // Prepare the reactive or non-reactive wrapper strings
                            let open_wrapper = if lunas_script {
                                "$$lunasCreateNonReactive(".to_string()
                            } else {
                                "$$lunasReactive(".to_string()
                            };

                            // Insert wrapper before the initialization start
                            str_positions.push(TransformInfo::AddStringToPosition(
                                AddStringToPosition {
                                    position: start.saturating_sub(1),
                                    string: open_wrapper,
                                    sort_order: 1,
                                },
                            ));
                            // Insert closing parenthesis after the initialization end
                            str_positions.push(TransformInfo::AddStringToPosition(
                                AddStringToPosition {
                                    position: end.saturating_sub(1),
                                    string: ")".to_string(),
                                    sort_order: 1,
                                },
                            ));
                        }
                    }
                }
            }
        }

        (str_positions, Some(num_generator))
    } else {
        (Vec::new(), None)
    }
}

/// Finds all imports whose source ends with ".luns" and returns their local names.
pub fn find_luns_imports(json: &Value) -> Vec<String> {
    let mut imports = Vec::new();

    // Get the top-level "body" array
    if let Some(Value::Array(body)) = json.get("body") {
        for item in body {
            // Check if this item is an ImportDeclaration
            if item.get("type").and_then(Value::as_str) == Some("ImportDeclaration") {
                // Extract the module specifier string
                if let Some(source_str) = item
                    .get("source")
                    .and_then(|src| src.get("value"))
                    .and_then(Value::as_str)
                {
                    // Filter for those ending with ".lun.ts"
                    if source_str.ends_with(".lun.ts") {
                        // Iterate over all specifiers
                        if let Some(specs) = item.get("specifiers").and_then(Value::as_array) {
                            for spec in specs {
                                // For each ImportSpecifier, take the local name
                                if let Some(local_name) = spec
                                    .get("local")
                                    .and_then(|l| l.get("value"))
                                    .and_then(Value::as_str)
                                {
                                    imports.push(local_name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    imports
}

fn power_of_two_generator(init: u32) -> impl FnMut() -> BigUint {
    let mut count = init;
    move || -> BigUint {
        let result = BigUint::from(2u32).pow(count as u32);
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
    delete_imports: bool,
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
                                        sort_order: 0,
                                    })],
                                    vec![],
                                    vec![variable_name.clone()],
                                    vec![], // Function names are not targeted here
                                );
                            }
                        }
                    }
                }
            }
            return (vec![], vec![], vec![], vec![]);
        } else if delete_imports == true
            && obj.contains_key("type")
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
                    delete_imports,
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
                delete_imports,
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
                delete_imports,
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

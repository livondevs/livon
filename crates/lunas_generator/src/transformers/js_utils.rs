use std::vec;

use lunas_parser::DetailedBlock;
use num_bigint::BigUint;
use serde_json::Value;

use crate::{
    ast_analyzer::function_analyzer::analyze_ast,
    structs::{
        js_analyze::{JsFunctionDeps, Tidy},
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
        // 1) Prepare buffers for transforms, imports, dependency variables, and functions
        let mut positions: Vec<TransformInfo> = Vec::new();
        let mut imports: Vec<String> = Vec::new();
        let mut dep_vars: Vec<String> = Vec::new();
        let mut funcs: Vec<String> = Vec::new();

        // 2) Find all variable declarations
        let (str_positions, mut num_gen) =
            find_variable_declarations(&js_block.ast, initial_num, variables, false);
        positions.extend(str_positions);

        // 3) Collect Lunas-specific imports and assign numbers
        let lun_imports = find_luns_imports(&js_block.ast);
        for lun_import in &lun_imports {
            variables.push(VariableNameAndAssignedNumber {
                name: lun_import.clone(),
                assignment: num_gen.as_mut().unwrap()(),
            });
        }

        // 4) Build a list of variable names for search
        let variable_names: Vec<String> = variables.iter().map(|v| v.name.clone()).collect();

        // 5) Invoke search_json with mutable buffers for all outputs
        search_json(
            &js_block.ast,
            &js_block.raw,
            &variable_names,
            &vec![],
            true,
            &mut positions,
            &mut imports,
            &mut dep_vars,
            &mut funcs,
        );

        // 6) Analyze function dependencies
        let mut functions_and_deps = analyze_ast(&js_block.ast, &variable_names);
        functions_and_deps.tidy();

        // 7) Apply transformations to the script
        let output = add_or_remove_strings_to_script(positions.clone(), &js_block.raw);

        (imports, output, functions_and_deps, lun_imports)
    } else {
        // Return empties if no JS block
        (vec![], String::new(), vec![], vec![])
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

pub fn search_json(
    json: &Value,
    raw_js: &str,
    variables: &[String],
    parents: &Vec<&Value>,
    delete_imports: bool,
    transforms: &mut Vec<TransformInfo>,
    imports_out: &mut Vec<String>,
    dep_vars_out: &mut Vec<String>,
    funcs_out: &mut Vec<String>,
) {
    let parent = parents.last().clone();
    let next_parents = parents
        .clone()
        .into_iter()
        .chain(std::iter::once(json))
        .collect::<Vec<&Value>>();
    if let Value::Object(obj) = json {
        // Identifier case
        if obj.get("type") == Some(&Value::String("Identifier".into())) {
            // Skip adding .v if inside the first argument array of a Lunas.watch call.
            let mut skip_watch_arg = false;
            for (i, ancestor) in parents.iter().enumerate() {
                // 2) Find a CallExpression whose callee is `Lunas.watch`
                if ancestor.get("type").and_then(Value::as_str) == Some("CallExpression") {
                    if let Some(callee) = ancestor.get("callee").and_then(Value::as_object) {
                        let is_watch = callee.get("type").and_then(Value::as_str)
                            == Some("MemberExpression")
                            && callee
                                .get("object")
                                .and_then(|o| o.get("value"))
                                .and_then(Value::as_str)
                                == Some("Lunas")
                            && callee
                                .get("property")
                                .and_then(|p| p.get("value"))
                                .and_then(Value::as_str)
                                == Some("watch");
                        if is_watch {
                            // 3) Check if an ArrayExpression appears directly under that CallExpression
                            let mut saw_array = false;
                            for child in parents.iter().skip(i + 1) {
                                if child.get("type").and_then(Value::as_str)
                                    == Some("ArrayExpression")
                                {
                                    saw_array = true;
                                    break;
                                }
                            }
                            if saw_array {
                                skip_watch_arg = true;
                            }
                            break;
                        }
                    }
                }
            }
            if skip_watch_arg {
                // We are inside Lunas.watch’s first argument array — do not append `.v`
                return;
            }
            let parent_is_obj = match parent {
                Some(Value::Object(_)) => true,
                _ => false,
            };
            let skip = parent_is_obj
                && parent.clone().unwrap().get("type").as_ref()
                    != Some(&&Value::String("VariableDeclarator".into()));
            let parent_is_array = matches!(parent, Some(Value::Array(_)));
            if (skip || parent_is_array)
                && obj
                    .get("value")
                    .and_then(Value::as_str)
                    .map_or(false, |v| variables.contains(&v.to_string()))
            {
                if let Some(span) = obj.get("span").and_then(Value::as_object) {
                    if let Some(end) = span.get("end").and_then(Value::as_u64) {
                        transforms.push(TransformInfo::AddStringToPosition(AddStringToPosition {
                            position: (end - 1) as u32,
                            string: ".v".into(),
                            sort_order: 0,
                        }));
                        dep_vars_out.push(obj["value"].as_str().unwrap().to_string());
                    }
                }
            }
            return;
        }

        // ImportDeclaration removal
        if delete_imports && obj.get("type") == Some(&Value::String("ImportDeclaration".into())) {
            let start = obj["span"]["start"].as_u64().unwrap() as u32 - 1;
            let mut end = obj["span"]["end"].as_u64().unwrap() as u32;
            if raw_js.chars().nth(end as usize) == Some('\n') {
                end += 1;
            }
            transforms.push(TransformInfo::RemoveStatement(RemoveStatement {
                start_position: start,
                end_position: end,
            }));
            let snippet: String = raw_js
                .chars()
                .skip(start as usize)
                .take((end - start) as usize)
                .collect();
            imports_out.push(snippet);
            return;
        }

        // MemberExpression replacements
        if obj.get("type") == Some(&Value::String("MemberExpression".into())) {
            let mut replace_target = None;
            if let (Some(object), Some(property)) = (obj.get("object"), obj.get("property")) {
                if let (Some(obj_val), Some(prop_val)) = (
                    object.get("value").and_then(Value::as_str),
                    property.get("value").and_then(Value::as_str),
                ) {
                    if obj_val == "Lunas" {
                        replace_target = match prop_val {
                            "router" => Some("$$lunasRouter"),
                            "afterMount" => Some("$$lunasAfterMount"),
                            "afterUnmount" => Some("$$lunasAfterUnmount"),
                            "watch" => Some("$$lunasWatch"),
                            _ => None,
                        };
                    }
                }
            }
            if let Some(new_text) = replace_target {
                if let Some(span) = obj.get("span").and_then(Value::as_object) {
                    let start = span["start"].as_u64().unwrap() as u32 - 1;
                    let end = span["end"].as_u64().unwrap() as u32 - 1;
                    transforms.push(TransformInfo::ReplaceText(ReplaceText {
                        start_position: start,
                        end_position: end,
                        string: new_text.into(),
                    }));
                }
                return;
            }
            // Recurse into all fields if not replaced.
            for (key, value) in obj {
                if key == "property" {
                    continue;
                }
                if obj.get("type").and_then(Value::as_str) == Some("KeyValueProperty")
                    && key == "key"
                {
                    continue;
                }
                search_json(
                    value,
                    raw_js,
                    variables,
                    &next_parents,
                    delete_imports,
                    transforms,
                    imports_out,
                    dep_vars_out,
                    funcs_out,
                );
            }
            return;
        }

        // CallExpression: recurse into callee + args
        if obj.get("type") == Some(&Value::String("CallExpression".into())) {
            // collect function name
            if let Some(callee) = obj.get("callee").and_then(Value::as_object) {
                if callee.get("type") == Some(&Value::String("Identifier".into())) {
                    if let Some(name) = callee.get("value").and_then(Value::as_str) {
                        funcs_out.push(name.to_string());
                    }
                }
            }
            // Recurse into all fields, but skip property keys in a KeyValueProperty.
            for (key, value) in obj {
                if obj.get("type").and_then(Value::as_str) == Some("KeyValueProperty")
                    && key == "key"
                {
                    continue;
                }
                search_json(
                    value,
                    raw_js,
                    variables,
                    &next_parents,
                    delete_imports,
                    transforms,
                    imports_out,
                    dep_vars_out,
                    funcs_out,
                );
            }
            return;
        }

        // Generic object: recurse into all fields except property keys
        for (key, value) in obj {
            if obj.get("type").and_then(Value::as_str) == Some("KeyValueProperty") && key == "key" {
                continue;
            }
            search_json(
                value,
                raw_js,
                variables,
                &next_parents,
                delete_imports,
                transforms,
                imports_out,
                dep_vars_out,
                funcs_out,
            );
        }
    }
    // Array: recurse each element
    else if let Value::Array(arr) = json {
        for value in arr {
            search_json(
                value,
                raw_js,
                variables,
                &next_parents,
                delete_imports,
                transforms,
                imports_out,
                dep_vars_out,
                funcs_out,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        js_utils::search_json,
        utils_swc::{parse_expr_with_swc, parse_module_with_swc},
    };
    use super::*;
    use serde_json::to_value;

    // Struct to hold the input parameters for the test.
    struct TestInput {
        raw_js: String,
        variables: Vec<String>,
        is_module: bool,
    }

    // Struct to hold the expected output for the test.
    struct TestExpected {
        output_js: String,
    }

    macro_rules! generate_for_test {
        ($name:ident, $input:expr, $expected:expr) => {
            #[test]
            fn $name() {
                let TestInput {
                    raw_js,
                    variables,
                    is_module,
                } = $input;
                let TestExpected { output_js } = $expected;
                let mut transforms = Vec::new();
                let mut imports = Vec::new();
                let mut dep_vars = Vec::new();
                let mut funcs = Vec::new();

                let parsed_json = if is_module {
                    to_value(parse_module_with_swc(&raw_js)).unwrap()
                } else {
                    to_value(parse_expr_with_swc(&raw_js)).unwrap()
                };
                // println!("AST: {:?}", parsed_json);
                search_json(
                    &parsed_json,
                    raw_js.as_str(),
                    variables.as_slice(),
                    &vec![],
                    false,
                    &mut transforms,
                    &mut imports,
                    &mut dep_vars,
                    &mut funcs,
                );

                let output = add_or_remove_strings_to_script(transforms.clone(), &raw_js);

                assert_eq!(output, output_js);
            }
        };
    }

    generate_for_test!(
        test_nested_object_identifier,
        TestInput {
            raw_js: "const currentBet = 0;\nconst obj = { inner: { currentBet: currentBet } };"
                .to_string(),
            variables: vec!["currentBet".to_string()],
            is_module: true
        },
        TestExpected {
            output_js:
                "const currentBet = 0;\nconst obj = { inner: { currentBet: currentBet.v } };"
                    .to_string()
        }
    );

    generate_for_test!(
        test_function_call_argument,
        TestInput {
            raw_js: "const currentBet = 0;\nfunction useBet() { return currentBet; }".to_string(),
            variables: vec!["currentBet".to_string()],
            is_module: true
        },
        TestExpected {
            output_js: "const currentBet = 0;\nfunction useBet() { return currentBet.v; }"
                .to_string()
        }
    );

    generate_for_test!(
        test_function_parameter_no_transform,
        TestInput {
            raw_js: "const currentBet = 0;\nfunction test(currentBet) { return currentBet; }"
                .to_string(),
            variables: vec!["currentBet".to_string()],
            is_module: true
        },
        TestExpected {
            output_js: "const currentBet = 0;\nfunction test(currentBet) { return currentBet; }"
                .to_string()
        }
    );

    generate_for_test!(
        test_function_object_property,
        TestInput {
            raw_js:
                "const obj = { property: \"hello\", obj: \"hello\" };\nfunction test() { return { a: obj.property, obj: obj.obj } }"
                    .to_string(),
            variables: vec!["obj".to_string()],
            is_module: true
        },
        TestExpected {
            output_js:
                "const obj = { property: \"hello\", obj: \"hello\" };\nfunction test() { return { a: obj.v.property, obj: obj.v.obj } }"
                    .to_string()
        }
    );

    generate_for_test!(
        test_dont_add_v_to_watch_func_first_arg,
        TestInput {
            raw_js: "Lunas.watch([count], () => { console.log(count) });".to_string(),
            variables: vec!["count".to_string()],
            is_module: true
        },
        TestExpected {
            output_js: "$$lunasWatch([count], () => { console.log(count.v) });".to_string()
        }
    );
    generate_for_test!(
        test_add_v_simple,
        TestInput {
            raw_js: "count".to_string(),
            variables: vec!["count".to_string()],
            is_module: false
        },
        TestExpected {
            output_js: "count.v".to_string()
        }
    );
}

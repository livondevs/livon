use serde_json::Value;
use std::collections::HashSet;

use crate::structs::js_analyze::JsFunctionDeps;

pub fn analyze_ast(ast: &Value, external_vars: &Vec<String>) -> Vec<JsFunctionDeps> {
    let mut top_level_func_names = HashSet::new();
    if let Some(body) = ast.get("body").and_then(|b| b.as_array()) {
        for node in body {
            if node.get("type").and_then(|t| t.as_str()) == Some("FunctionDeclaration") {
                if let Some(identifier) = node.get("identifier") {
                    if let Some(name) = identifier.get("value").and_then(|v| v.as_str()) {
                        top_level_func_names.insert(name.to_string());
                    }
                }
            }
        }
    }

    let mut results = Vec::new();

    if let Some(body) = ast.get("body").and_then(|b| b.as_array()) {
        for node in body {
            if node.get("type").and_then(|t| t.as_str()) == Some("FunctionDeclaration") {
                if let Some(identifier) = node.get("identifier") {
                    if let Some(func_name) = identifier.get("value").and_then(|v| v.as_str()) {
                        let mut depending_vars = HashSet::new();
                        let mut depending_funcs = HashSet::new();
                        traverse(
                            node.get("body").unwrap_or(&Value::Null),
                            external_vars,
                            &top_level_func_names,
                            &mut depending_vars,
                            &mut depending_funcs,
                        );
                        results.push(JsFunctionDeps {
                            name: func_name.to_string(),
                            depending_vars: depending_vars.into_iter().collect(),
                            depending_funcs: depending_funcs.into_iter().collect(),
                        });
                    }
                }
            }
        }
    }

    results
}

fn traverse(
    node: &Value,
    external_vars: &Vec<String>,
    top_level_func_names: &HashSet<String>,
    depending_vars: &mut HashSet<String>,
    depending_funcs: &mut HashSet<String>,
) {
    if node.is_null() {
        return;
    }
    if let Some(array) = node.as_array() {
        for elem in array {
            traverse(
                elem,
                external_vars,
                top_level_func_names,
                depending_vars,
                depending_funcs,
            );
        }
        return;
    }
    if let Some(obj) = node.as_object() {
        if let Some(ty) = obj.get("type").and_then(|v| v.as_str()) {
            if ty == "FunctionDeclaration" {
                return;
            }
            if ty == "Identifier" {
                if let Some(value) = obj.get("value").and_then(|v| v.as_str()) {
                    if external_vars.contains(&value.to_string()) {
                        depending_vars.insert(value.to_string());
                    }
                }
            }
            if ty == "CallExpression" {
                if let Some(callee) = obj.get("callee") {
                    if let Some(callee_obj) = callee.as_object() {
                        if let Some(callee_ty) = callee_obj.get("type").and_then(|v| v.as_str()) {
                            if callee_ty == "Identifier" {
                                if let Some(name) = callee_obj.get("value").and_then(|v| v.as_str())
                                {
                                    if top_level_func_names.contains(&name.to_string()) {
                                        depending_funcs.insert(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        for (_, v) in obj {
            traverse(
                v,
                external_vars,
                top_level_func_names,
                depending_vars,
                depending_funcs,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // This test verifies that analyze_ast correctly extracts the dependencies:
    // - For function "foo", it should detect external variable "x" and the function call to top-level "bar".
    // - For function "bar", it should detect external variable "y" only.
    #[test]
    fn test_analyze_ast() {
        // AST for testing
        let ast = json!({
          "body": [
            {
              "type": "FunctionDeclaration",
              "identifier": { "value": "foo" },
              "body": {
                "type": "BlockStatement",
                "body": [
                  { "type": "Identifier", "value": "x" },
                  {
                    "type": "CallExpression",
                    "callee": { "type": "Identifier", "value": "bar" }
                  }
                ]
              }
            },
            {
              "type": "FunctionDeclaration",
              "identifier": { "value": "bar" },
              "body": {
                "type": "BlockStatement",
                "body": [
                  { "type": "Identifier", "value": "y" }
                ]
              }
            }
          ]
        });

        // Definition of external variable list
        let external_vars = vec!["x".to_string(), "y".to_string(), "z".to_string()];

        // Call analyze_ast to analyze dependencies
        let mut deps = analyze_ast(&ast, &external_vars);

        // Since the order of results is not guaranteed, sort by function name for predictable comparison.
        deps.sort_by(|a, b| a.name.cmp(&b.name));

        // Expected result for function "bar"
        let expected_bar = JsFunctionDeps {
            name: "bar".to_string(),
            depending_vars: ["y".to_string()].into_iter().collect(),
            depending_funcs: HashSet::new(),
        };

        // Expected result for function "foo"
        let expected_foo = JsFunctionDeps {
            name: "foo".to_string(),
            depending_vars: ["x".to_string()].into_iter().collect(),
            depending_funcs: ["bar".to_string()].into_iter().collect(),
        };

        // Create expected result vector and sort by function name.
        let mut expected = vec![expected_bar, expected_foo];
        expected.sort_by(|a, b| a.name.cmp(&b.name));

        // Assert that the output matches the expected dependencies.
        assert_eq!(deps, expected);
    }
}

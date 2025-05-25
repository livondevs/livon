use serde_json::Value;
use std::collections::HashSet;

use crate::structs::js_analyze::JsFunctionDeps;

pub fn analyze_ast(ast: &Value, external_vars: &Vec<String>) -> Vec<JsFunctionDeps> {
    let mut top_level_func_names = HashSet::new();
    if let Some(body) = ast.get("body").and_then(|b| b.as_array()) {
        for node in body {
            // Collect FunctionDeclaration names.
            if node.get("type").and_then(|t| t.as_str()) == Some("FunctionDeclaration") {
                if let Some(identifier) = node.get("identifier") {
                    if let Some(name) = identifier.get("value").and_then(|v| v.as_str()) {
                        top_level_func_names.insert(name.to_string());
                    }
                }
            }
            // Collect arrow functions defined by VariableDeclaration.
            if node.get("type").and_then(|t| t.as_str()) == Some("VariableDeclaration") {
                if let Some(declarations) = node.get("declarations").and_then(|d| d.as_array()) {
                    for declarator in declarations {
                        if let Some(init) = declarator.get("init") {
                            if init.get("type").and_then(|t| t.as_str())
                                == Some("ArrowFunctionExpression")
                            {
                                if let Some(id) = declarator.get("id") {
                                    if let Some(name) = id.get("value").and_then(|v| v.as_str()) {
                                        top_level_func_names.insert(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let mut results = Vec::new();

    if let Some(body) = ast.get("body").and_then(|b| b.as_array()) {
        for node in body {
            // Process FunctionDeclaration.
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
            // Process ArrowFunctionExpression in VariableDeclaration.
            if node.get("type").and_then(|t| t.as_str()) == Some("VariableDeclaration") {
                if let Some(declarations) = node.get("declarations").and_then(|d| d.as_array()) {
                    for declarator in declarations {
                        if let Some(init) = declarator.get("init") {
                            if init.get("type").and_then(|t| t.as_str())
                                == Some("ArrowFunctionExpression")
                            {
                                if let Some(id) = declarator.get("id") {
                                    if let Some(func_name) =
                                        id.get("value").and_then(|v| v.as_str())
                                    {
                                        let mut depending_vars = HashSet::new();
                                        let mut depending_funcs = HashSet::new();
                                        traverse(
                                            init.get("body").unwrap_or(&Value::Null),
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
            // Do not traverse into nested function bodies.
            if ty == "FunctionDeclaration" || ty == "ArrowFunctionExpression" {
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
    use std::collections::HashSet;

    #[test]
    fn test_analyze_ast_with_function_declarations() {
        // AST for testing FunctionDeclaration nodes.
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

        let external_vars = vec!["x".to_string(), "y".to_string(), "z".to_string()];
        let mut deps = analyze_ast(&ast, &external_vars);
        deps.sort_by(|a, b| a.name.cmp(&b.name));

        let expected_bar = JsFunctionDeps {
            name: "bar".to_string(),
            depending_vars: ["y".to_string()].into_iter().collect(),
            depending_funcs: HashSet::new(),
        };

        let expected_foo = JsFunctionDeps {
            name: "foo".to_string(),
            depending_vars: ["x".to_string()].into_iter().collect(),
            depending_funcs: ["bar".to_string()].into_iter().collect(),
        };

        let mut expected = vec![expected_bar, expected_foo];
        expected.sort_by(|a, b| a.name.cmp(&b.name));

        assert_eq!(deps, expected);
    }

    #[test]
    fn test_analyze_ast_with_arrow_function() {
        // AST including an arrow function defined inside a VariableDeclaration.
        let ast = json!({
          "body": [
            {
              "type": "VariableDeclaration",
              "span": { "start": 0, "end": 30 },
              "ctxt": 0,
              "kind": "const",
              "declare": false,
              "declarations": [
                {
                  "type": "VariableDeclarator",
                  "span": { "start": 6, "end": 30 },
                  "id": {
                    "type": "Identifier",
                    "span": { "start": 6, "end": 11 },
                    "ctxt": 2,
                    "value": "item2",
                    "optional": false,
                    "typeAnnotation": null
                  },
                  "init": {
                    "type": "ArrowFunctionExpression",
                    "span": { "start": 14, "end": 30 },
                    "ctxt": 0,
                    "params": [],
                    "body": {
                      "type": "BlockStatement",
                      "span": { "start": 18, "end": 30 },
                      "ctxt": 3,
                      "stmts": [
                        {
                          "type": "ReturnStatement",
                          "span": { "start": 22, "end": 28 },
                          "argument": { "type": "Identifier", "value": "x" }
                        }
                      ]
                    },
                    "async": false,
                    "generator": false,
                    "typeParameters": null,
                    "returnType": null
                  },
                  "definite": false
                }
              ]
            }
          ]
        });

        let external_vars = vec!["x".to_string()];
        let deps = analyze_ast(&ast, &external_vars);

        let expected = JsFunctionDeps {
            name: "item2".to_string(),
            depending_vars: ["x".to_string()].into_iter().collect(),
            depending_funcs: HashSet::new(),
        };

        assert_eq!(deps, vec![expected]);
    }
}

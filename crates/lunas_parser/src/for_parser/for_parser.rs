//! Parser for `for..of` and `for..in` statements, converting them to a unified representation.
//!
//! This module uses SWC to parse the input and directly traverses the AST
//! to extract loop components. It uses `SourceMap::span_to_snippet`
//! to get the exact source substrings from AST node spans.

use std::error::Error;
use std::result::Result;

use swc_common::{sync::Lrc, FileName, SourceMap, SourceMapper, Span, Spanned}; // Added Spanned
use swc_ecma_ast::{
    CallExpr, Callee, Expr, ForHead, ForInStmt, ForOfStmt, MemberProp, ModuleItem, Stmt,
};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

/// Kind of `for` statement: `for..of` or `for..in`.
#[derive(Debug, PartialEq, Clone)]
pub enum ForKind {
    Of,
    In,
}

/// Parsed representation of a `for` statement.
#[derive(Debug, PartialEq, Clone)]
pub struct ParsedFor {
    pub kind: ForKind,
    pub iterable: String,
    pub pattern: Vec<String>,
    /// Raw pattern string as appeared in the input (e.g., "[x]" or "item"),
    /// without 'let', 'const', or 'var'.
    pub raw: String,
}

impl ParsedFor {
    /// Parse strings like `"const [idx, val] of data.entries()"` or `"let key in mapObj"`.
    pub fn parse(input: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let src = input.trim();
        let wrapped = format!("for({}){{}}", src);

        let cm: Lrc<SourceMap> = Default::default();
        let fm = cm.new_source_file(FileName::Custom("for_stmt.js".into()).into(), wrapped);
        let lexer = Lexer::new(
            Syntax::Es(Default::default()),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );
        let mut parser = Parser::new_from(lexer);
        let module = parser.parse_module().map_err(|e| {
            Box::<dyn Error + Send + Sync>::from(format!(
                "SWC parse error for input '{}': {:?}",
                src, e
            ))
        })?;

        let module_item = module.body.into_iter().next().ok_or_else(|| {
            Box::<dyn Error + Send + Sync>::from(format!("Empty input, no statements: '{}'", src))
        })?;

        let actual_stmt = match module_item {
            ModuleItem::Stmt(s) => s,
            _ => {
                return Err(Box::<dyn Error + Send + Sync>::from(format!(
                    "Input did not yield a statement: '{}'",
                    src
                )))
            }
        };

        println!("actual_stmt: {:?}", actual_stmt);

        let (kind, left_for_head, right_expr) = match actual_stmt {
            Stmt::ForOf(ForOfStmt { left, right, .. }) => (ForKind::Of, left, right),
            Stmt::ForIn(ForInStmt { left, right, .. }) => (ForKind::In, left, right),
            _ => {
                // Catch all other statement types
                return Err(Box::<dyn Error + Send + Sync>::from(format!(
                    "Not a for..of or for..in statement (found other statement type for input): '{}'", src
                )));
            }
        };

        let pattern_span: Span = match left_for_head {
            ForHead::VarDecl(var_decl) => {
                if var_decl.decls.is_empty() {
                    return Err(From::from(
                        "Variable declaration in for loop head has no declarators",
                    ));
                }
                var_decl.decls[0].name.span() // Pat (in VarDeclarator) implements Spanned
            }
            ForHead::Pat(pat) => pat.span(),
            ForHead::UsingDecl(_) => {
                return Err(Box::<dyn Error + Send + Sync>::from(format!(
                    "Not a for..of or for..in statement (found UsingDecl): '{}'",
                    src
                )));
            }
        };

        let raw = cm
            .span_to_snippet(pattern_span)
            .map_err(|e| {
                Box::<dyn Error + Send + Sync>::from(format!(
                    "Failed to get snippet for pattern (span {:?}): {:?}",
                    pattern_span, e
                ))
            })?
            .trim()
            .to_string();

        let mut iterable_span = right_expr.span(); // Box<Expr> implements Spanned

        if let Expr::Call(CallExpr {
            callee, // callee is Callee (not &Callee due to destructuring)
            args,
            ..
        }) = &*right_expr
        // Dereference Box<Expr> to get &Expr
        {
            if let Callee::Expr(callee_expr_val) = callee {
                // callee_expr_val is Box<Expr>
                if let Expr::Member(member_expr) = &**callee_expr_val {
                    // Deref Box<Expr> to &Expr, then match
                    if let MemberProp::Ident(ident_prop) = &member_expr.prop {
                        // prop is MemberProp
                        if ident_prop.sym.as_ref() == "entries" {
                            // obj is Box<Expr>
                            if let Expr::Ident(obj_ident) = &*member_expr.obj {
                                // Deref Box<Expr> to &Expr
                                if obj_ident.sym.as_ref() == "Object" {
                                    if let Some(first_arg) = args.get(0) {
                                        if first_arg.spread.is_none() {
                                            iterable_span = first_arg.expr.span();
                                            // first_arg.expr is Box<Expr>
                                        }
                                    }
                                } else {
                                    iterable_span = member_expr.obj.span(); // obj is Box<Expr>
                                }
                            } else {
                                iterable_span = member_expr.obj.span(); // obj is Box<Expr>
                            }
                        }
                    }
                }
                // This else-if branch was for future use and can be removed if not immediately needed
                // to simplify. If kept, ensure callee_ident is properly used or marked as unused.
                // } else if let Callee::Expr(callee_expr_val) = callee {
                //     if let Expr::Ident(_callee_ident) = &**callee_expr_val {
                //         // _callee_ident is &Ident
                //     }
            }
        }

        let iterable = cm
            .span_to_snippet(iterable_span)
            .map_err(|e| {
                Box::<dyn Error + Send + Sync>::from(format!(
                    "Failed to get snippet for iterable (span {:?}): {:?}",
                    iterable_span, e
                ))
            })?
            .trim()
            .to_string();

        let inner_pattern_str = raw
            .trim_start_matches(&['[', '{'][..])
            .trim_end_matches(&[']', '}'][..])
            .trim();

        let pattern = if inner_pattern_str.is_empty() {
            if raw == "[]" || raw == "{}" {
                Vec::new()
            } else if !raw.starts_with(&['[', '{'][..])
                && !raw.ends_with(&[']', '}'][..])
                && !raw.is_empty()
            {
                vec![raw.clone()]
            } else {
                Vec::new()
            }
        } else {
            inner_pattern_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };

        Ok(ParsedFor {
            kind,
            iterable,
            pattern,
            raw,
        }
        .for_in_to_for_of())
    }

    pub fn clone_with_new_iterable(&self, new_iterable: &str) -> Self {
        ParsedFor {
            kind: self.kind.clone(),
            iterable: new_iterable.to_string(),
            pattern: self.pattern.clone(),
            raw: self.raw.clone(),
        }
    }

    pub fn for_in_to_for_of(&self) -> Self {
        if self.kind == ForKind::In {
            ParsedFor {
                kind: ForKind::Of,
                iterable: format!("Object.keys({})", self.iterable),
                pattern: self.pattern.clone(),
                raw: self.raw.clone(),
            }
        } else {
            self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ForKind, ParsedFor};

    macro_rules! generate_for_tests {
        ($($name:ident: $input:expr => $expected:expr),+ $(,)?) => {
            $(#[test]
            fn $name() {
                let pf = ParsedFor::parse($input).unwrap_or_else(|e| panic!("Parse failed for input '{}':\nError: {:?}", $input, e));
                let exp: ParsedFor = $expected;
                assert_eq!(pf.kind,     exp.kind, "Input: '{}'", $input);
                assert_eq!(pf.iterable, exp.iterable, "Input: '{}'", $input);
                assert_eq!(pf.raw,      exp.raw, "Input: '{}'", $input);
                assert_eq!(pf.pattern,  exp.pattern, "Input: '{}'", $input);
            })+
        };
    }

    macro_rules! generate_for_error_tests {
        ($($name:ident: $input:expr),+ $(,)?) => {
            $(#[test]
            #[should_panic]
            fn $name() {
                match ParsedFor::parse($input) {
                    Ok(parsed_for) => {
                        panic!("Expected error for input '{}', but got Ok({:?})", $input, parsed_for);
                    }
                    Err(_) => { /* Expected error, test will pass due to should_panic */ }
                }
            })+
        };
    }

    generate_for_tests! {
        object_entries_1: "const [index, value] of Object.entries(data)" => ParsedFor { kind: ForKind::Of, iterable: "Object.entries(data)".into(), pattern: vec!["index".into(), "value".into()], raw: "[index, value]".into() },
        object_entries_2: "var { k , v } of Object.entries( myMap )" => ParsedFor { kind: ForKind::Of, iterable: "Object.entries( myMap )".into(), pattern: vec!["k".into(), "v".into()], raw: "{ k , v }".into() },
        method_entries_1: "const [idx, val] of myData.entries()" => ParsedFor { kind: ForKind::Of, iterable: "myData.entries()".into(), pattern: vec!["idx".into(), "val".into()], raw: "[idx, val]".into() },
        method_entries_2: "let [ k, v ] of another_obj.get_items().entries()" => ParsedFor { kind: ForKind::Of, iterable: "another_obj.get_items().entries()".into(), pattern: vec!["k".into(), "v".into()], raw: "[ k, v ]".into() },
        method_entries_3: "[i, b] of bools.entries()" => ParsedFor { kind: ForKind::Of, iterable: "bools.entries()".into(), pattern: vec!["i".into(), "b".into()], raw: "[i, b]".into() },
        plain_of_1: "let value of dataArr" => ParsedFor { kind: ForKind::Of, iterable: "dataArr".into(), pattern: vec!["value".into()], raw: "value".into() },
        plain_of_2: "item of getItems()" => ParsedFor { kind: ForKind::Of, iterable: "getItems()".into(), pattern: vec!["item".into()], raw: "item".into() },
        plain_of_3: "val of obj.prop" => ParsedFor { kind: ForKind::Of, iterable: "obj.prop".into(), pattern: vec!["val".into()], raw: "val".into() },
        in_without_value_1: "var key in mapObj" => ParsedFor { kind: ForKind::In, iterable: "mapObj".into(), pattern: vec!["key".into()], raw: "key".into() },
        in_without_value_2: "index in obj.getIndices()" => ParsedFor { kind: ForKind::In, iterable: "obj.getIndices()".into(), pattern: vec!["index".into()], raw: "index".into() },
        whitespace_variations_1: " [ i , v ] of Object.entries(  sampleData ) " => ParsedFor { kind: ForKind::Of, iterable: "sampleData".into(), pattern: vec!["i".into(), "v".into()], raw: "[ i , v ]".into() },
        whitespace_variations_2: "const\t[ index , value ]\rof\t myArr.entries( \n ) " => ParsedFor { kind: ForKind::Of, iterable: "myArr".into(), pattern: vec!["index".into(), "value".into()], raw: "[ index , value ]".into() },
        whitespace_variations_3: " let \t item \n of \t data " => ParsedFor { kind: ForKind::Of, iterable: "data".into(), pattern: vec!["item".into()], raw: "item".into() },
        whitespace_variations_4: " key\tin\tobject " => ParsedFor { kind: ForKind::In, iterable: "object".into(), pattern: vec!["key".into()], raw: "key".into() },
        function_calling_rhs_1: "item of filteredItems()" => ParsedFor { kind: ForKind::Of, iterable: "filteredItems()".into(), pattern: vec!["item".into()], raw: "item".into() },
        edge_case_1: "let [ k, v ] of (getObj()).items.entries()" => ParsedFor { kind: ForKind::Of, iterable: "(getObj()).items".into(), pattern: vec!["k".into(), "v".into()], raw: "[ k, v ]".into() },
        edge_case_2: "const [idx, val] of Object.entries(await getData().then(r => r.json()))" => ParsedFor { kind: ForKind::Of, iterable: "await getData().then(r => r.json())".into(), pattern: vec!["idx".into(), "val".into()], raw: "[idx, val]".into() },
        edge_case_3: "[i6] of [...Array(counts[5]).keys()]" => ParsedFor { kind: ForKind::Of, iterable: "[...Array(counts[5]).keys()]".into(), pattern: vec!["i6".into()], raw: "[i6]".into() },
        edge_case_4: "i of [...Array(bools.length).keys()]" => ParsedFor { kind: ForKind::Of, iterable: "[...Array(bools.length).keys()]".into(), pattern: vec!["i".into()], raw: "i".into() },
        valid_js_no_decl_array: "[a,b,c] of d" => ParsedFor { kind: ForKind::Of, iterable: "d".into(), pattern: vec!["a".into(), "b".into(), "c".into()], raw: "[a,b,c]".into()},
        valid_js_no_decl_object: "const {i, v} of nonEntries()" => ParsedFor { kind: ForKind::Of, iterable: "nonEntries()".into(), pattern: vec!["i".into(), "v".into()], raw: "{i, v}".into()},
        // Test for "const [a,] of d" which is valid JS
        valid_trailing_comma_pattern: "const [a,] of d" => ParsedFor { kind: ForKind::Of, iterable: "d".into(), pattern: vec!["a".into()], raw: "[a,]".into() },
        // Test for "const [i, v] in data.entries()" which is valid JS
        valid_for_in_destructuring: "const [i, v] in data.entries()" => ParsedFor { kind: ForKind::In, iterable: "data.entries()".into(), pattern: vec!["i".into(), "v".into()], raw: "[i, v]".into() },
    }

    generate_for_error_tests! {
        invalid_1: "for foo bar",
        invalid_2: "let [a] of",
        invalid_syntax_for_loop_extra_tokens: "let a in obj extra",
        invalid_4: "let [a,b c] of data",
        // invalid_5 was moved to generate_for_tests as valid_trailing_comma_pattern
        invalid_6: "x of y z",
        invalid_7: "in obj",
        invalid_8: "let x y z of arr",
        invalid_9: "", // Empty input error
        // invalid_10 was moved to generate_for_tests as valid_for_in_destructuring
        invalid_13: "val of obj.",
        invalid_14: "val of obj.()",
    }

    // Remove specific test functions test_invalid_5_specific and test_invalid_10_specific
    // as their logic is now incorporated into generate_for_tests! or confirmed as error cases.
}

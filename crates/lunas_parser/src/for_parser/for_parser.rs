use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "for_parser/for_parser.pest"]
struct ForParser;

#[derive(Debug, PartialEq)]
pub struct ParsedFor {
    pub iter_array: String,
    pub item_index: Option<String>,
    pub item_value: Option<String>,
}

/// Parses a JS `for ... of/in ...` header and returns its components.
/// Returns Err(String) on any parse or semantic error.
pub fn parse_for_statement(input: &str) -> Result<ParsedFor, String> {
    let mut pairs = ForParser::parse(Rule::for_stmt, input).map_err(|e| e.to_string())?;
    let stmt = pairs.next().ok_or("Empty parse result")?;

    let mut index = None;
    let mut value = None;
    let mut pending_ident = None;
    let mut operator = None;
    let mut rhs = None;

    for pair in stmt.into_inner() {
        match pair.as_rule() {
            Rule::declaration => {}
            Rule::pattern => {
                let inner = pair.into_inner().next().unwrap();
                match inner.as_rule() {
                    Rule::array_destructure | Rule::object_destructure => {
                        let mut idents = inner.into_inner();
                        index = Some(idents.next().unwrap().as_str().to_string());
                        value = Some(idents.next().unwrap().as_str().to_string());
                    }
                    Rule::identifier => pending_ident = Some(inner.as_str().to_string()),
                    _ => unreachable!(),
                }
            }
            Rule::operator => operator = Some(pair.as_str()),
            Rule::rhs => rhs = Some(pair.as_str().trim().to_string()),
            _ => unreachable!(),
        }
    }

    // Semantic validations
    let op = operator.ok_or("Missing operator")?;
    if op == "in" && (value.is_some()) {
        return Err("Destructuring pattern not allowed with 'in' operator".into());
    }

    // Set pending identifier based on operator
    if let Some(name) = pending_ident {
        match op {
            "in" => index = Some(name),
            "of" => value = Some(name),
            _ => unreachable!(),
        }
    }

    // rhsが `Object.entries(...)` や `.entries()` で終わる場合、内部を抽出
    let iter_array = if let Some(rhs_str) = rhs {
        if let Some(array_name) = rhs_str
            .strip_prefix("Object.entries(")
            .and_then(|s| s.strip_suffix(')'))
        {
            array_name.trim().to_string()
        } else if rhs_str.ends_with(".entries()") {
            rhs_str.strip_suffix(".entries()").unwrap().to_string()
        } else {
            rhs_str
        }
    } else {
        return Err("Missing RHS".into());
    };

    Ok(ParsedFor {
        iter_array,
        item_index: index,
        item_value: value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! generate_for_tests {
        ($($name:ident: $input:expr => $expected:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    match parse_for_statement($input) {
                        Ok(actual) => assert_eq!(actual, $expected, "Test failed for input: {}", $input),
                        Err(e) => panic!("Expected Ok({:?}), got Err('{}') for input: {}", $expected, e, $input),
                    }
                }
            )*
        };
    }

    macro_rules! generate_for_error_tests {
        ($($name:ident: $input:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    assert!(parse_for_statement($input).is_err(), "Expected Err for input: {}", $input);
                }
            )*
        };
    }

    use crate::ParsedFor;

    generate_for_tests! {
        object_entries_1: "const [index, value] of Object.entries(data)" => ParsedFor { iter_array: "data".into(), item_index: Some("index".into()), item_value: Some("value".into()) },
        object_entries_2: "var { k , v } of Object.entries( myMap )" => ParsedFor { iter_array: "myMap".into(), item_index: Some("k".into()), item_value: Some("v".into()) },
        method_entries_1: "const [idx, val] of myData.entries()" => ParsedFor { iter_array: "myData".into(), item_index: Some("idx".into()), item_value: Some("val".into()) },
        method_entries_2: "let [ k, v ] of another_obj.get_items().entries()" => ParsedFor { iter_array: "another_obj.get_items()".into(), item_index: Some("k".into()), item_value: Some("v".into()) },
        plain_of_1: "let value of dataArr" => ParsedFor { iter_array: "dataArr".into(), item_index: None, item_value: Some("value".into()) },
        plain_of_2: "item of getItems()" => ParsedFor { iter_array: "getItems()".into(), item_index: None, item_value: Some("item".into()) },
        plain_of_3: "val of obj.prop" => ParsedFor { iter_array: "obj.prop".into(), item_index: None, item_value: Some("val".into()) },
        in_without_value_1: "var key in mapObj" => ParsedFor { iter_array: "mapObj".into(), item_index: Some("key".into()), item_value: None },
        in_without_value_2: "index in obj.getIndices()" => ParsedFor { iter_array: "obj.getIndices()".into(), item_index: Some("index".into()), item_value: None },
        whitespace_variations_1: " [ i , v ] of Object.entries(  sampleData ) " => ParsedFor { iter_array: "sampleData".into(), item_index: Some("i".into()), item_value: Some("v".into()) },
        whitespace_variations_2: "const\t[ index , value ]\rof\t myArr.entries( \n ) " => ParsedFor { iter_array: "myArr".into(), item_index: Some("index".into()), item_value: Some("value".into()) },
        whitespace_variations_3: " let \t item \n of \t data " => ParsedFor { iter_array: "data".into(), item_index: None, item_value: Some("item".into()) },
        whitespace_variations_4: " key\tin\tobject " => ParsedFor { iter_array: "object".into(), item_index: Some("key".into()), item_value: None },
        function_calling_rhs_1: "item of filteredItems()" => ParsedFor { iter_array: "filteredItems()".into(), item_index: None, item_value: Some("item".into()) },
        edge_case_1: "let [ k, v ] of (getObj()).items.entries()" => ParsedFor { iter_array: "(getObj()).items".into(), item_index: Some("k".into()), item_value: Some("v".into()) },
        edge_case_2: "const [idx, val] of Object.entries(await getData().then(r => r.json()))" => ParsedFor { iter_array: "await getData().then(r => r.json())".into(), item_index: Some("idx".into()), item_value: Some("val".into()) },
        edge_case_3: "[i6] of [...Array(counts[5]).keys()]" => ParsedFor { iter_array: "[...Array(counts[5]).keys()]".into(), item_index: Some("i6".into()), item_value: None },
        edge_case_4: "i of [...Array(bools.length).keys()]" => ParsedFor { iter_array: "[...Array(bools.length).keys()]".into(), item_index: Some("i".into()), item_value: None },
    }

    generate_for_error_tests! {
        invalid_1: "for foo bar",
        invalid_2: "let [a] of",
        invalid_3: "let a in obj extra",
        invalid_4: "let [a,b c] of data",
        invalid_5: "const [a,] of d",
        invalid_6: "x of y z",
        invalid_7: "in obj",
        invalid_8: "let x y z of arr",
        invalid_9: "",
        invalid_10: "const [i, v] in data.entries()",
        invalid_11: "const {i, v} of nonEntries()",
        invalid_12: "[a,b,c] of d",
        invalid_13: "val of obj.",
        invalid_14: "val of obj.()",
    }
}

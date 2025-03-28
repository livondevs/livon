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

    let mut array = None;
    let mut index = None;
    let mut value = None;
    let mut pending_ident = None;
    let mut op = None;

    let stmt = pairs
        .next()
        .ok_or_else(|| "Empty parse result".to_string())?;
    for pair in stmt.into_inner() {
        match pair.as_rule() {
            Rule::pattern => {
                let mut inner = pair.into_inner();
                if let Some(first) = inner.next() {
                    match first.as_rule() {
                        Rule::array_destructure | Rule::object_destructure => {
                            let mut parts = first.into_inner();
                            index = Some(parts.next().unwrap().as_str().to_string());
                            value = Some(parts.next().unwrap().as_str().to_string());
                        }
                        Rule::identifier => pending_ident = Some(first.as_str().to_string()),
                        _ => {}
                    }
                }
            }
            Rule::operator => op = Some(pair.as_str()),
            Rule::rhs => {
                for inner in pair.into_inner() {
                    match inner.as_rule() {
                        Rule::object_entries | Rule::method_entries => {
                            array = Some(inner.into_inner().next().unwrap().as_str().to_string());
                        }
                        Rule::plain => array = Some(inner.as_str().to_string()),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(name) = pending_ident {
        match op.unwrap_or("") {
            "in" => index = Some(name),
            _ => value = Some(name),
        }
    }

    Ok(ParsedFor {
        iter_array: array.ok_or_else(|| "Missing array operand".to_string())?,
        item_index: index,
        item_value: value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expect_ok(input: &str, expected: ParsedFor) {
        assert_eq!(parse_for_statement(input).unwrap(), expected);
    }

    fn expect_err(input: &str) {
        assert!(parse_for_statement(input).is_err());
    }

    #[test]
    fn test_object_entries() {
        expect_ok(
            "const [index, value] of Object.entries(data)",
            ParsedFor {
                iter_array: "data".into(),
                item_index: Some("index".into()),
                item_value: Some("value".into()),
            },
        );
    }

    #[test]
    fn test_method_entries() {
        expect_ok(
            "const [idx, val] of myData.entries()",
            ParsedFor {
                iter_array: "myData".into(),
                item_index: Some("idx".into()),
                item_value: Some("val".into()),
            },
        );
    }

    #[test]
    fn test_plain_of() {
        expect_ok(
            "let value of dataArr",
            ParsedFor {
                iter_array: "dataArr".into(),
                item_index: None,
                item_value: Some("value".into()),
            },
        );
    }

    #[test]
    fn test_in_without_value() {
        expect_ok(
            "var key in mapObj",
            ParsedFor {
                iter_array: "mapObj".into(),
                item_index: Some("key".into()),
                item_value: None,
            },
        );
    }

    #[test]
    fn test_whitespace_variations() {
        expect_ok(
            " [ i , v ] of Object.entries(  sampleData )",
            ParsedFor {
                iter_array: "sampleData".into(),
                item_index: Some("i".into()),
                item_value: Some("v".into()),
            },
        );
    }

    #[test]
    fn test_invalid_syntax() {
        expect_err("for foo bar");
    }
}

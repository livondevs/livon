use nom::{
    branch::permutation,
    bytes::complete::{is_not, tag, take_while1},
    character::complete::{alphanumeric0, alphanumeric1, space0},
    combinator::{all_consuming, opt},
    IResult,
};

use super::blocks::MetaData;

#[derive(Debug)]
pub enum DetailedMetaData {
    PropsInput(PropsInput),
    UseComponentStatement(UseComponentStatement),
    UseAutoRoutingStatement,
    UseRoutingStatement,
}

#[derive(Debug)]
pub struct PropsInput {
    pub variable_name: String,
    pub type_of_value: String,
    pub initial_value: Option<String>,
    pub is_nullable: bool,
}

#[derive(Debug)]
pub struct UseComponentStatement {
    pub component_name: String,
    pub component_path: String,
}

impl<'a> DetailedMetaData {
    pub fn from_simple_meta_data(simple_meta_data: MetaData) -> Result<Self, &'a str> {
        if simple_meta_data.kind == "input" {
            match parse_input_content(&simple_meta_data.content) {
                Ok((_, (variable_name, type_of_value, initial_value, is_nullable))) => {
                    Ok(Self::PropsInput(PropsInput {
                        variable_name: variable_name.to_string(),
                        type_of_value: type_of_value.to_string(),
                        initial_value: match initial_value {
                            Some(initial_value) => Some(initial_value.to_string()),
                            None => None,
                        },
                        is_nullable: is_nullable,
                    }))
                }
                Err(_) => Err("error parsing input content"),
            }
        } else if simple_meta_data.kind == "use" {
            parse_component_use_statement(&simple_meta_data.content)
                .map(|(_, (component_name, _, component_path))| {
                    Self::UseComponentStatement(UseComponentStatement {
                        component_name: component_name.to_string(),
                        component_path: component_path.to_string(),
                    })
                })
                .map_err(|_| "error parsing use statement")
        } else if simple_meta_data.kind == "useAutoRouting" {
            Ok(Self::UseAutoRoutingStatement)
        } else if simple_meta_data.kind == "useRouting" {
            Ok(Self::UseRoutingStatement)
        } else {
            Err("unknown kind of meta data")
        }
    }
}

fn parse_input_content(input: &str) -> IResult<&str, (&str, &str, Option<&str>, bool)> {
    let (input, (_, variable_name, _, _, _, type_name, optional_question_mark, content)) =
        permutation((
            space0,
            alphanumeric1,
            space0,
            tag(":"),
            space0,
            alphanumeric0,
            opt(tag("?")),
            opt(permutation((
                space0,
                tag("="),
                space0,
                all_consuming(is_not("\n")),
            ))),
        ))(input)?;

    let is_optional = optional_question_mark != None;
    let initial_value = match content {
        Some((_, _, _, value)) => Some(value),
        None => None,
    };

    Ok((
        &input,
        (variable_name, type_name, initial_value, is_optional),
    ))
}

use nom::{
    branch::alt,
    bytes::complete::take_while,
    character::complete::{char, multispace1},
    sequence::{preceded, tuple},
};

fn is_alphanumeric_underscore(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn parse_string(input: &str) -> IResult<&str, &str> {
    let (input, _) = alt((char('\''), char('\"')))(input)?;
    let (input, str_contents) = take_while(|c: char| c != '\'' && c != '\"')(input)?;
    let (input, _) = alt((char('\''), char('\"')))(input)?;
    Ok((input, str_contents))
}

fn parse_component_use_statement(input: &str) -> IResult<&str, (&str, &str, &str)> {
    all_consuming(tuple((
        take_while1(is_alphanumeric_underscore),
        preceded(multispace1, tag("from")),
        preceded(multispace1, parse_string),
    )))(input)
}

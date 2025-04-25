extern crate nom;

use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, not_line_ending},
    combinator::{map, recognize},
    multi::many0,
    IResult,
};

pub fn parse_language_name(input: &str) -> IResult<&str, String> {
    map(recognize(alphanumeric1), String::from)(input)
}

pub fn empty_lines(input: &str) -> IResult<&str, &str> {
    recognize(many0(tag("\n")))(input)
}

pub fn parse_content(input: &str) -> IResult<&str, String> {
    map(recognize(not_line_ending), String::from)(input)
}

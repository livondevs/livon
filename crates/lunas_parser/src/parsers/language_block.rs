use crate::parsers::utils::{empty_lines, parse_language_name};
use crate::structs::blocks::{LanguageBlock, ParsedItem};
extern crate nom;

use nom::bytes::complete::take_while;
use nom::character::complete::line_ending;
use nom::combinator::{not, opt, peek};
use nom::multi::many_till;

use nom::{
    branch::alt, bytes::complete::tag, character::complete::multispace0, sequence::tuple, IResult,
};

pub fn parse_language_block(input: &str) -> IResult<&str, ParsedItem> {
    let (input, _) = empty_lines(input)?;
    let (input, language_name) = parse_language_name(input)?;
    let (input, _) = tuple((multispace0, tag(":")))(input)?;
    let (input, _) = tag("\n")(input)?;

    let (input, content) = indented_content(input)?;

    Ok((
        input,
        ParsedItem::LanguageBlock(LanguageBlock {
            language_name,
            content: content.trim().to_string(),
        }),
    ))
}

fn is_space_or_tab(c: char) -> bool {
    c == ' ' || c == '\t'
}

fn is_not_line_ending(c: char) -> bool {
    c != '\n' && c != '\r'
}

fn indent(input: &str) -> IResult<&str, &str> {
    take_while(is_space_or_tab)(input)
}

fn indented_line<'a>(indentation: &'a str) -> impl Fn(&'a str) -> IResult<&'a str, &'a str> {
    move |input: &'a str| {
        let (input, ret) = opt(tag("\n"))(input)?;
        if ret != None {
            return Ok((input, ""));
        }
        let (input, _) = tag(indentation)(input)?;
        let (input, line) = take_while(is_not_line_ending)(input)?;
        let (input, _) = line_ending(input)?;
        Ok((input, line))
    }
}

fn take_empty_lines(input: &str) -> IResult<&str, &str> {
    take_while(|c| c == '\n' || c == '\r')(input)
}

fn content_of_first_line(input: &str) -> IResult<&str, &str> {
    let (input, content) = take_while(is_not_line_ending)(input)?;
    Ok((input, content))
}

fn indented_content(input: &str) -> IResult<&str, String> {
    let (input, _) = take_empty_lines(input)?;
    let (input, initial_indentation) = indent(input)?;
    let (input, content_of_first_line) = content_of_first_line(input)?;

    let (input, _) = line_ending(input)?;

    let (input, (mut lines, _)) = many_till(
        indented_line(initial_indentation),
        peek(not(alt((tag(initial_indentation), tag("\n"))))),
    )(input)?;

    lines.insert(0, content_of_first_line);

    let joined = lines.join("\n");

    Ok((input, joined))
}

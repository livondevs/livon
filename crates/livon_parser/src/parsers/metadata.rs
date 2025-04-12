use crate::parsers::utils::{empty_lines, parse_content, parse_language_name};
use crate::structs::blocks::{MetaData, ParsedItem};
use nom::branch::permutation;
use nom::character::complete::{alphanumeric1, space0};
use nom::combinator::opt;
use nom::multi::separated_list0;
use nom::{bytes::complete::tag, IResult};
use nom::{
    error::{ErrorKind, ParseError},
    AsChar, InputTakeAtPosition,
};
use std::collections::HashMap;

pub fn parse_meta_data<'a>(input: &str) -> IResult<&str, ParsedItem> {
    let (input, _) = empty_lines(input)?;

    let (input, _) = tag("@")(input)?;
    let (input, kind) = parse_language_name(input)?;
    let mut params = HashMap::new();
    let (input, tg) = opt(tag("("))(input)?;
    let input = if tg != None {
        let (input, result) = separated_list0(
            tag(","),
            permutation((
                space0,
                alphanumeric1,
                space0,
                tag(":"),
                space0,
                alphanumeric_or_quotes,
                space0,
            )),
        )(input)?;

        for (_, key, _, _, _, value, _) in result {
            params.insert(key.to_string(), value.to_string());
        }

        let (input, _) = tag(")")(input)?;
        Ok(input)
    } else {
        Ok(input)
    }?;

    let (input, _) = space0(input)?;
    let (input, content) = parse_content(input)?;

    Ok((
        input,
        ParsedItem::MetaData(MetaData {
            kind,
            params,
            content,
        }),
    ))
}

fn alphanumeric_or_quotes<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: InputTakeAtPosition,
    T::Item: AsChar + Clone,
    <T as InputTakeAtPosition>::Item: AsChar,
{
    input.split_at_position1_complete(
        |item| {
            let c = item.as_char();
            !(c.is_alphanumeric() || c == '\'' || c == '\"')
        },
        ErrorKind::AlphaNumeric,
    )
}

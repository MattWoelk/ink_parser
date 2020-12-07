// nom notes:
// many0/many1 - run a parser a bunch of times and return the results in a Vec
// alt - try all of the provided parsers (in a tuple) and return the result of the first one that works
//       alt((alpha1, digit1)) will grab digits and alphanumerics until it runs out
//       !!! so we don't need many1 in front of this !!!
// permutation - the same as alt, but each parser must be run at least once
//             - it returns the results in a tuple. It's a bit of a mystery.
//
// The ones that sound like they're doing multiples, but it's only for characters (which I don't like, so avoid these):
// take_until("chars") - this goes until it finds the tag "chars"
// is_a("12345") - grabs as many things that are 1 2 3 4 or 5 until it runs out
// tag("thing") - grabs a "thing" and throws it away. This is actually useful.
// take_till(|c| c == ':') - take until we match that one character. Not very useful.
// take_until("tag") - take until we match string of characters. Not very useful.
// take_while(|c| c != ':') - yeah, useless.
// TODO: delete this junk up here --^

use nom::branch::alt;
use nom::bytes::complete::{is_a, is_not, take_until, take_while, take_while1, take_while_m_n};
use nom::character::complete::alpha1;
use nom::combinator::{not, opt};
use nom::multi::{many0, many1};
use nom::sequence::{preceded, terminated, tuple};
use nom::IResult;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Knot {
    title: String,
    lines: Vec<String>,
}

impl Default for Knot {
    fn default() -> Self {
        todo!()
    }
}

/// whitespace that is not a newline character
fn nbsp_opt(input: &str) -> IResult<&str, &str> {
    take_while(|c| " \t".contains(c))(input)
    //is_a(" \t")(input)
}

/// newline characters
fn newline(input: &str) -> IResult<&str, &str> {
    preceded(nbsp_opt, take_while1(|c| "\n\r".contains(c)))(input)
}

fn equal_signs(input: &str) -> IResult<&str, &str> {
    preceded(nbsp_opt, take_while_m_n(2, 999, |c| c == '='))(input)
}

pub fn parse_knot_header(input: &str) -> IResult<&str, &str> {
    let (input, _) = terminated(&equal_signs, nbsp_opt)(input)?;

    let (input, title) = terminated(is_not(" \r\n\t=()/\\!@#$%^&*"), nbsp_opt)(input)?;

    let (input, _) = tuple((opt(equal_signs), newline))(input)?;

    Ok((input, title.trim_end()))
}

pub fn parse_text_line(input: &str) -> IResult<&str, &str> {
    let (input, text) = alt((take_until("->"), take_until("\n")))(input)?;
    Ok((input, text))
}

pub fn parse_knot(input: &str) -> IResult<&str, Knot> {
    let (input, title) = parse_knot_header(input)?;

    // TODO: parse as many lines as we can find, and deal with empty lines.
    // TODO: parse diverts (-> London) and throw those in here too, intelligently

    let (input, results) = many1(alt((parse_text_line, newline)))(input)?;
    //let (input, results) = many1(is_not("q"))(input)?;
    dbg!(results);
    let line = "";

    //let (input, line) = parse_text_line(input)?;
    //todo!();

    //let (input, line) = parse_text_line(input)?;

    Ok((
        input,
        Knot {
            title: title.into(),
            lines: vec![line.into()],
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_knot_header() {
        assert_eq!(
            parse_knot_header("== train_station\n"),
            Ok(("", "train_station"))
        );

        assert_eq!(
            parse_knot_header("== train_station ==\n"),
            Ok(("", "train_station"))
        );

        assert_eq!(
            parse_knot_header(" === train_station === \n"),
            Ok(("", "train_station"))
        );

        assert_eq!(
            parse_knot_header("===train_station===\n"),
            Ok(("", "train_station"))
        );

        assert!(parse_knot_header(" = train_station = \n").is_err()); // not enough equals signs
        assert!(parse_knot_header("== train station ==\n").is_err()); // no spaces allowed
        assert!(parse_knot_header("train station\n").is_err()); // no equals at the start
    }

    #[test]
    fn test_parse_text_line() {
        assert_eq!(
            parse_text_line("a line of text\n"),
            Ok(("\n", "a line of text"))
        );
        assert_eq!(
            parse_text_line("a line of text"),
            Ok(("", "a line of text"))
        );
        assert_eq!(
            parse_text_line("a line of text -> end"),
            Ok(("-> end", "a line of text "))
        );
    }

    #[test]
    fn test_knot() {
        assert_eq!(
            parse_knot("===train_station===\na line of text\n"),
            Ok((
                "",
                Knot {
                    title: "train_station".into(),
                    lines: vec!["a line of text".into()],
                }
            ))
        );
    }
}

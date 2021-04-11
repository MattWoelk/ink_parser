#![warn(rust_2018_idioms)]
#![allow(unused)]
use combine::error::StringStreamError;
use combine::parser::char::{char, digit, spaces, string};
use combine::parser::char::{crlf, newline};
use combine::parser::choice::{choice, or};
use combine::parser::range::{recognize, take_while1};
use combine::parser::repeat::{skip_until, take_until};
use combine::parser::sequence::skip;
use combine::{
    any, between, chainl1, look_ahead, none_of, not_followed_by, parser, satisfy, skip_count,
};
use combine::{
    attempt, eof, many, many1, optional, sep_by, sep_by1, skip_many1, token, ParseError, Parser,
    RangeStream, Stream,
};
use maplit::hashmap;
use pretty_assertions::{assert_eq, assert_ne};
use std::collections::HashMap;

// TODO: diverts should only parse one word, don't use "line()" for everything.

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Knot {
    title: String,
    lines: Vec<String>,
    choices: Vec<Choice>,
    divert: Option<Divert>,
}

impl Default for Knot {
    fn default() -> Self {
        Knot {
            title: "".to_string(),
            lines: vec![],
            choices: vec![],
            divert: None,
        }
    }
}

type KnotTitle = String;
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct Story {
    knots: HashMap<KnotTitle, Knot>,
}

fn line<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    spaces().with(
        not_followed_by(char('+').or(char('-'))).with(
            many1::<String, _, _>(satisfy(|c| c != '\n' && c != '\r'))
                .skip(optional(char('\n').or(char('\r').skip(char('\n'))))),
        ),
    )
}

fn lines<'a, Input>() -> impl Parser<Input, Output = Vec<String>>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(line().map(|s| s.into()))
}

type Divert = String;
type Choice = (String, Option<Vec<String>>, Option<Divert>);

// TODO: variables, conditionals, etc.
enum LineTypes {
    TEXT(String),
    DIVERT(Divert),
    CHOICE(Choice),
}

fn divert_line<'a, Input>() -> impl Parser<Input, Output = LineTypes>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    string("->")
        .and(many::<String, _, _>(char(' ')))
        .with(line())
        .map(|a| LineTypes::DIVERT(a))
}

fn choice_line<'a, Input>() -> impl Parser<Input, Output = LineTypes>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    char('+')
        .and(many::<String, _, _>(char(' ')))
        .with(line())
        .map(|a| LineTypes::CHOICE((a, None, None)))
}

fn knot_body_lines<'a, Input>() -> impl Parser<Input, Output = Vec<LineTypes>>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1::<Vec<LineTypes>, _, _>(choice((
        divert_line(),
        choice_line(),
        line().map(|x| LineTypes::TEXT(x)),
    )))
}

fn knot_title<'a, Input>() -> impl Parser<Input, Output = String>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    spaces().with(
        string("==")
            .skip(many::<String, _, _>(char('=')))
            .skip(spaces())
            .with(line()),
    )
}

fn knot<'a, Input>() -> impl Parser<Input, Output = Knot>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    knot_title()
        .and(knot_without_title())
        .map(|(knot_title, knot)| Knot {
            title: knot_title,
            lines: knot.lines,
            choices: vec![], // TODO
            divert: None,    // TODO
        })
}

fn knot_without_title<'a, Input>() -> impl Parser<Input, Output = Knot>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    spaces()
        .with(optional(lines()))
        .and(optional(many::<Vec<LineTypes>, _, _>(choice_line())))
        .and(optional(divert_line()))
        .map(|((a, b), c)| Knot {
            title: "INTRO".to_string(),
            lines: match a {
                Some(lines) => lines,
                None => vec![], // TODO
            },
            choices: vec![], // TODO
            divert: None,    // TODO
        })
}

fn story<'a, Input>() -> impl Parser<Input, Output = Story>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    knot_without_title().and(many(knot())).map(
        |(intro_knot, mut other_knots): (Knot, Vec<Knot>)| {
            let mut knots: Vec<Knot> = vec![];
            knots.push(intro_knot);
            knots.append(other_knots.as_mut());
            Story {
                knots: knots
                    .into_iter()
                    .map(|knot: Knot| (knot.title.clone(), knot))
                    .collect(),
            }
        },
    )
}

pub fn parse_story(text: &str) -> Story {
    story().parse(text).unwrap().0
}

#[derive(PartialEq, Debug)]
pub struct Date {
    pub year: i32,
    pub month: i32,
    pub day: i32,
}

fn two_digits<Input>() -> impl Parser<Input, Output = i32>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (digit(), digit()).map(|(x, y): (char, char)| {
        let x = x.to_digit(10).expect("digit");
        let y = y.to_digit(10).expect("digit");
        (x * 10 + y) as i32
    })
}

/// Parses a date
/// 2010-01-30
fn date<'a, Input>() -> impl Parser<Input, Output = Date>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        many::<String, _, _>(digit()),
        char('-'),
        two_digits(),
        char('-'),
        two_digits(),
    )
        .map(|(year, _, month, _, day)| {
            // Its ok to just unwrap since we only parsed digits
            Date {
                year: year.parse().unwrap(),
                month,
                day,
            }
        })
}

#[test]
fn test_line() {
    assert_eq!(
        line().parse("+ option"),
        Err(StringStreamError::UnexpectedParse)
    );
    assert_eq!(
        line().parse("-> divert"),
        Err(StringStreamError::UnexpectedParse)
    );
    assert_eq!(
        line().parse("no line endings"),
        Ok(("no line endings".to_string(), ""))
    );
    assert_eq!(
        line().parse("one line ending\n"),
        Ok(("one line ending".to_string(), ""))
    );
    assert_eq!(
        line().parse("both line endings\r\n"),
        Ok(("both line endings".to_string(), ""))
    );
    assert_eq!(
        line().parse("          line starting with spaces\r\n"),
        Ok(("line starting with spaces".to_string(), ""))
    );
    assert_eq!(
        line().parse("       \n \r\n   line starting with newlines and spaces\r\n"),
        Ok(("line starting with newlines and spaces".to_string(), ""))
    );
}

#[test]
fn test_story() {
    assert_eq!(
        story().parse(include_str!("../stories/two_knots.ink")),
        Ok((
            Story {
                knots: hashmap! {
                    "INTRO".into() => Knot {
                        title: "INTRO".into(),
                        lines: vec![],
                        choices: vec![],
                        divert: Some("paris".to_string()),
                    },
                    "paris".into() => Knot {
                        title: "paris".into(),
                        lines: vec![],
                        choices: vec![],
                        divert: Some("ending".to_string()),
                    },
                    "ending".into() => Knot {
                        title: "ending".into(),
                        lines: vec![],
                        choices: vec![],
                        divert: Some("END".to_string()),
                    },
                }
            },
            ""
        ))
    );

    assert_eq!(
        story().parse(include_str!("../stories/basic_story.ink")),
        Ok((Story::default(), ""))
    );
}

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
    EasyParser,
};
use combine::{
    attempt, eof, many, many1, optional, sep_by, sep_by1, skip_many1, token, ParseError, Parser,
    RangeStream, Stream,
};
use maplit::btreemap;
use pretty_assertions::{assert_eq, assert_ne};
use std::collections::BTreeMap;
use std::unreachable;

// TODO: diverts should only parse one word, don't use "line()" for everything.
// TODO: pass state along, so when parsing fails I can debug it.

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
    knots: BTreeMap<KnotTitle, Knot>,
}

fn line<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    not_followed_by(char('+').or(char('-')))
        .expected("wut?")
        .with(
            many1::<String, _, _>(satisfy(|c| c != '\n' && c != '\r'))
                .skip(optional(char('\n').or(char('\r').skip(char('\n'))))),
        )
}

fn lines<'a, Input>() -> impl Parser<Input, Output = Vec<String>>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    spaces().with(many1(line().skip(spaces()).map(|s| s.into())))
}

type Divert = String;
type Choice = (String, Option<Vec<String>>, Option<Divert>);

// TODO: variables, conditionals, etc.
// TODO: should the choice_lines (etc.) function return a piece of these, but have them be able to be cast to this main enum or something? What's the right way to do this?
#[derive(Debug, PartialEq, Eq, Clone)]
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

fn choice_lines<'a, Input>() -> impl Parser<Input, Output = LineTypes>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    char('+')
        .and(many::<String, _, _>(char(' ')))
        .with(line())
        .and(optional(lines()))
        .and(optional(divert_line()))
        .map(|((title, lines), divert)| {
            LineTypes::CHOICE((
                title,
                lines,
                divert.map(|divert| match divert {
                    LineTypes::DIVERT(text) => text,
                    _ => "".to_string(),
                }),
            ))
        })
}

fn knot_body_lines<'a, Input>() -> impl Parser<Input, Output = Vec<LineTypes>>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1::<Vec<LineTypes>, _, _>(choice((
        divert_line(),
        choice_lines(),
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
            choices: knot.choices,
            divert: knot.divert,
        })
}

fn knot_without_title<'a, Input>() -> impl Parser<Input, Output = Knot>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    spaces()
        .with(optional(lines()))
        .and(optional(many::<Vec<LineTypes>, _, _>(choice_lines())))
        .and(optional(divert_line()))
        .map(|((lines, choices), divert)| Knot {
            title: "INTRO".to_string(),
            lines: match lines {
                Some(lines) => lines,
                None => vec![],
            },
            choices: match choices {
                Some(choices) => choices
                    .into_iter()
                    .map(|x| match x {
                        LineTypes::CHOICE(c) => c,
                        _ => unreachable!(),
                    })
                    .collect(),
                None => vec![],
            },
            divert: divert.map(|x| match x {
                LineTypes::DIVERT(divert) => divert,
                _ => "".to_string(),
            }),
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
    story().easy_parse(text).unwrap().0
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
        line().easy_parse("no line endings"),
        Ok(("no line endings".to_string(), ""))
    );
    assert_eq!(
        line().easy_parse("one line ending\n"),
        Ok(("one line ending".to_string(), ""))
    );
    assert_eq!(
        line().easy_parse("both line endings\r\n"),
        Ok(("both line endings".to_string(), ""))
    );
    assert_eq!(
        line().easy_parse("          line starting with spaces\r\n"),
        Ok(("          line starting with spaces".to_string(), ""))
    );
    assert_eq!(
        line().easy_parse("       \n \r\n   line starting with newlines and spaces\r\n"),
        Ok((
            "       ".to_string(),
            " \r\n   line starting with newlines and spaces\r\n"
        ))
    );
}

#[test]
fn test_divert() {
    assert_eq!(
        divert_line().easy_parse("-> yeah"),
        Ok((LineTypes::DIVERT(("yeah".to_string())), ""))
    );

    //assert_eq!(
    //    divert_line().easy_parse("    -> yeah"),
    //    Ok((LineTypes::DIVERT(("yeah".to_string())), ""))
    //);
}

#[test]
fn test_choice() {
    assert_eq!(
        choice_lines().easy_parse("+ yeah"),
        Ok((LineTypes::CHOICE(("yeah".to_string(), None, None)), ""))
    );

    assert_eq!(
        choice_lines().easy_parse("+ yeah\n  one\ntwo\n     three"),
        Ok((
            LineTypes::CHOICE((
                "yeah".to_string(),
                Some(vec![
                    "one".to_string(),
                    "two".to_string(),
                    "three".to_string()
                ]),
                None
            )),
            ""
        ))
    );

    assert_eq!(
        choice_lines().easy_parse("+ yeah\n  one\ntwo\n-> paris"),
        Ok((
            LineTypes::CHOICE((
                "yeah".to_string(),
                Some(vec!["one".to_string(), "two".to_string(),]),
                Some("paris".to_string())
            )),
            ""
        ))
    );

    assert_eq!(
        lines().easy_parse("  one\ntwo\n   + paris"),
        Ok((vec!["one".to_string(), "two".to_string(),], "+ paris"))
    );

    assert_eq!(
        lines().easy_parse("  one\ntwo\n   -> paris"),
        Ok((vec!["one".to_string(), "two".to_string(),], "-> paris"))
    );

    assert_eq!(
        choice_lines().easy_parse("+ yeah\n  one\ntwo\n   -> paris"),
        Ok((
            LineTypes::CHOICE((
                "yeah".to_string(),
                Some(vec!["one".to_string(), "two".to_string(),]),
                Some("paris".to_string())
            )),
            ""
        ))
    );
}

#[test]
fn test_story() {
    assert_eq!(
        story().easy_parse(include_str!("../stories/two_knots.ink")),
        Ok((
            Story {
                knots: btreemap! {
                    "INTRO".to_string() => Knot {
                        title: "INTRO".to_string(),
                        lines: vec![
                            "to paris".to_string()
                        ],
                        choices: vec![],
                        divert: Some("paris".to_string()),
                    },
                    "paris".to_string() => Knot {
                        title: "paris".to_string(),
                        lines: vec![
                            "We are in paris.".to_string()
                        ],
                        choices: vec![],
                        divert: Some("ending".to_string()),
                    },
                    "ending".to_string() => Knot {
                        title: "ending".to_string(),
                        lines: vec![
                            "THE END now.".to_string()
                        ],
                        choices: vec![],
                        divert: Some("END".to_string()),
                    },
                }
            },
            ""
        ))
    );

    assert_eq!(
        story().easy_parse(include_str!("../stories/two_knots_with_choices.ink")),
        Ok((
            Story {
                knots: btreemap! {
                    "INTRO".to_string() => Knot {
                        title: "INTRO".to_string(),
                        lines: vec![
                            "to paris?".to_string()
                        ],
                        choices: vec![
                            ("yeah".to_string(), Some(vec!["yes, please".to_string()]), Some("paris".to_string())),
                            ("no".to_string(), Some(vec!["no, thank you".to_string()]), Some("ending".to_string())),
                        ],
                        divert: None,
                    },
                    "paris".to_string() => Knot {
                        title: "paris".to_string(),
                        lines: vec![
                            "We are in paris.".to_string()
                        ],
                        choices: vec![],
                        divert: Some("ending".to_string()),
                    },
                    "ending".to_string() => Knot {
                        title: "ending".to_string(),
                        lines: vec![
                            "THE END now.".to_string()
                        ],
                        choices: vec![],
                        divert: Some("END".to_string()),
                    },
                }
            },
            ""
        ))
    );
}

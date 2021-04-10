#![warn(rust_2018_idioms)]
#![allow(unused)]
use combine::parser::char::{char, digit, spaces, string};
use combine::parser::char::{crlf, newline};
use combine::parser::choice::{choice, or};
use combine::parser::range::{recognize, take_while1};
use combine::parser::repeat::{skip_until, take_until};
use combine::parser::sequence::skip;
use combine::{any, between, chainl1, look_ahead, none_of, parser, satisfy, skip_count};
use combine::{
    attempt, eof, many, many1, optional, sep_by, sep_by1, skip_many1, token, ParseError, Parser,
    RangeStream, Stream,
};
use pretty_assertions::{assert_eq, assert_ne};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Knot {
    title: String,
    lines: Vec<String>,
}

impl Default for Knot {
    fn default() -> Self {
        Knot {
            title: "".to_string(),
            lines: vec![],
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
    many1::<String, _, _>(satisfy(|c| c != '\n' && c != '\r'))
        .skip(optional(char('\n').or(char('\r').skip(char('\n')))))
}

fn lines<'a, Input>() -> impl Parser<Input, Output = Vec<String>>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(line().map(|s| s.into()))
}

fn knot<'a, Input>() -> impl Parser<Input, Output = Knot>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    string("==")
        .with(line())
        .and(lines())
        .map(|(knot_title, text_lines)| Knot {
            title: knot_title.into(),
            lines: text_lines,
        })
}

fn knot_without_title<'a, Input>() -> impl Parser<Input, Output = Knot>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    //take_until(or(attempt(string("==")), eof().map(|_| ""))).map(|c: String| {
    lines().map(|text_lines| Knot {
        title: "INTRO".into(),
        lines: text_lines,
    })
}

fn story<'a, Input>() -> impl Parser<Input, Output = Story>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    knot_without_title()
        .and(many::<Vec<Knot>, _, _>(knot()))
        .map(|(into_knot, mut other_knots)| {
            let mut knots: Vec<Knot> = vec![];
            knots.push(into_knot);
            knots.append(other_knots.as_mut());
            Story {
                knots: knots
                    .into_iter()
                    .map(|knot: Knot| (knot.title.clone(), knot))
                    .collect(),
            }
        })
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
}

#[test]
fn test_story() {
    dbg!(parse_story(include_str!("../stories/basic_story.ink")));
    todo!()
}

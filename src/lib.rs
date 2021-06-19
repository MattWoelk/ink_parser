#![warn(rust_2018_idioms)]
#![allow(unused)]

mod tests;

use combine::parser::char::{char, digit, spaces, string};
//use combine::parser::char::{crlf, newline};
//use combine::parser::choice::{choice, or};
use combine::parser::choice::choice;
//use combine::parser::range::{recognize, take_while1};
use combine::parser::repeat::{skip_until, take_until};
//use combine::parser::sequence::skip;
use combine::{
    any, between, chainl1, look_ahead, none_of, not_followed_by, parser, satisfy, skip_count,
    EasyParser,
};
use combine::{
    attempt, eof, many, many1, optional, sep_by, sep_by1, skip_many1, token, ParseError, Parser,
    RangeStream, Stream,
};
use maplit::btreemap;
use std::collections::BTreeMap;
use std::unreachable;

// TODO: get rid of line() and replace it with something more targetted for the different purposes
// TODO: can I just delete all whitespace at the start of each line at the start? Or will that mess up with my indexing errors...?

// TODO: get rid of trailing "==="s on knot titles
// TODO: diverts should only parse one word, don't use "line()" for everything.
// TODO: pass state along, so when parsing fails I can debug it.

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Knot {
    title: String,
    lines: Vec<String>,
    choices: Vec<Choice>, // TODO: Should this be a BTreeMap?
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
type Choice = (String, Option<Vec<String>>, Option<Divert>); // TODO: Should this middle one be a BTreeMap?

// TODO: variables, conditionals, etc.
#[derive(Debug, PartialEq, Eq, Clone)]
enum LineType {
    TEXT(String),
    DIVERT(Divert),
    CHOICE(Choice),
}

fn divert_line<'a, Input>() -> impl Parser<Input, Output = Divert>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    string("->")
        .and(many::<String, _, _>(char(' ')))
        .with(line())
}

fn choice_lines<'a, Input>() -> impl Parser<Input, Output = Choice>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    char('+')
        .and(many::<String, _, _>(char(' ')))
        .with(line())
        .skip(spaces())
        .and(optional(lines()))
        .and(optional(divert_line().map(LineType::DIVERT)))
        .map(|((title, lines), divert)| {
            (
                title,
                lines,
                divert.map(|divert| match divert {
                    LineType::DIVERT(text) => text,
                    _ => "".to_string(),
                }),
            )
        })
}

fn knot_body_lines<'a, Input>() -> impl Parser<Input, Output = Vec<LineType>>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1::<Vec<LineType>, _, _>(choice((
        divert_line().map(LineType::DIVERT),
        choice_lines().map(LineType::CHOICE),
        line().map(LineType::TEXT),
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
        .and(optional(many::<Vec<Choice>, _, _>(choice_lines())))
        .and(optional(divert_line()))
        .map(|((lines, choices), divert)| Knot {
            title: "INTRO".to_string(),
            lines: match lines {
                Some(lines) => lines,
                None => vec![],
            },
            choices: match choices {
                Some(choices) => choices,
                None => vec![],
            },
            divert: divert,
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

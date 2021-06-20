#![warn(rust_2018_idioms)]
//#![allow(unused)]

mod tests;

use combine::parser::char::{char, spaces, string};
use combine::parser::choice::choice;
use combine::{many, many1, optional, ParseError, Parser, RangeStream, Stream};
use combine::{not_followed_by, satisfy, EasyParser};
use std::collections::BTreeMap;

// TODO: deal with blank dialog lines, if needed.

// TODO: get rid of trailing "==="s on knot titles
// TODO: pass state along, so when parsing fails I can debug it.
// TODO: variables, conditionals, etc.

type KnotTitle = String;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Divert {
    knot_title: KnotTitle,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Choice {
    text: String,
    dialog_lines: Vec<String>,
    divert: Divert,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum KnotEnding {
    CHOICES(Vec<Choice>), // TODO: should this be a BTreeMap?
    DIVERT(Divert),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Knot {
    title: String,
    dialog_lines: Vec<String>,
    ending: KnotEnding,
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct Story {
    knots: BTreeMap<KnotTitle, Knot>,
}

impl From<&str> for Divert {
    fn from(s: &str) -> Self {
        Divert {
            knot_title: s.to_string(),
        }
    }
}

impl Default for Knot {
    fn default() -> Self {
        Knot {
            title: "".to_string(),
            dialog_lines: vec![],
            ending: KnotEnding::CHOICES(vec![]),
        }
    }
}

/// grabs the rest of the line, and consumes any trailing newline marker
fn rest_of_the_line<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    // TODO: can this be simplified? I just want to grab everything until we hit \n or \r
    many1::<String, _, _>(satisfy(|c| c != '\n' && c != '\r'))
        .skip(optional(char('\n').or(char('\r').skip(char('\n')))))
}

fn dialog_line<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    spaces().with(rest_of_the_line())
}

/// Must call spaces() before calling this,
/// because we can't make it optional() if spaces() consumes input
fn dialog_lines<'a, Input>() -> impl Parser<Input, Output = Vec<String>>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1::<Vec<String>, _, _>(
        //spaces().with( // TODO: I would love to have this here, but it consumes input, so we can't have dialog_lines() be optional()
        not_followed_by(string("->")) // TODO: I would love to put divert() right in here; not sure why I can't
            .skip(not_followed_by(string("+")))
            .with(dialog_line())
            .skip(spaces()),
    )
}

//fn lines<'a, Input>() -> impl Parser<Input, Output = Vec<String>>
//where
//    Input: RangeStream<Token = char, Range = &'a str>,
//    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
//{
//    spaces().with(many1(line().skip(spaces()).map(|s| s.into())))
//}

fn divert<'a, Input>() -> impl Parser<Input, Output = Divert>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    spaces().with(
        string("->")
            .skip(spaces())
            .with(rest_of_the_line())
            .map(|s| Divert { knot_title: s }),
    )
}

fn parse_choice<'a, Input>() -> impl Parser<Input, Output = Choice>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    char('+')
        .skip(spaces())
        .with(rest_of_the_line())
        .skip(spaces())
        .and(optional(dialog_lines()))
        .and(divert())
        .map(|((title, lines), divert)| Choice {
            text: title,
            dialog_lines: lines.unwrap_or_default(),
            divert,
        })
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
            .with(rest_of_the_line()),
    )
}

fn knot_end<'a, Input>() -> impl Parser<Input, Output = KnotEnding>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    spaces().with(choice((
        (many1::<Vec<Choice>, _, _>(parse_choice()).map(KnotEnding::CHOICES)),
        divert().map(KnotEnding::DIVERT),
    )))
}

fn knot_without_title<'a, Input>() -> impl Parser<Input, Output = Knot>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    spaces()
        .with(optional(dialog_lines()))
        .and(knot_end())
        .map(|(lines, ending)| Knot {
            title: "INTRO".to_string(),
            dialog_lines: lines.unwrap_or_default(),
            ending,
        })
}

fn knot<'a, Input>() -> impl Parser<Input, Output = Knot>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    spaces().with(
        knot_title()
            .and(knot_without_title())
            .map(|(knot_title, knot)| Knot {
                title: knot_title,
                dialog_lines: knot.dialog_lines,
                ending: knot.ending,
            }),
    )
}

fn story<'a, Input>() -> impl Parser<Input, Output = Story>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    knot_without_title().and(many(knot())).map(
        |(intro_knot, mut other_knots): (Knot, Vec<Knot>)| {
            let mut knots: Vec<Knot> = vec![intro_knot];
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

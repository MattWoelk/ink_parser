#![warn(rust_2018_idioms)]
#![allow(unused)]
use combine::parser;
use combine::parser::char::{char, digit, spaces, string};
use combine::parser::char::{crlf, newline};
use combine::parser::choice::or;
use combine::parser::range::take_while1;
use combine::parser::repeat::take_until;
use combine::{
    attempt, eof, many, many1, optional, sep_by, sep_by1, skip_many1, token, ParseError, Parser,
    RangeStream, Stream,
};
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

fn line<'a, Input>() -> impl Parser<Input, Output = String>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    //take_until(or(crlf(), eof().map(|_| '_')))
    //range("==") //.map(|s: &'a str| s.to_string())
    let tool = take_until(string("\n").or(string("\r\n")));
    let tool = tool
        .and(string("\r\n")) // TODO: why isn't this failing? it's not even trying to parse this...
        .0
        .and(token('\n'))
        .0;
    //tool
    sep_by1(tool, string("NEVER FIND THIS TEXT SRTIEFTNERSt")) // TODO: why is this nonsense required? Why does it make it work? Shouldn't I be able to return tool on its own???
}

fn lines<'a, Input>() -> impl Parser<Input, Output = Vec<String>>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    //let line = || {
    //    many1(digit())
    //        .skip(spaces())
    //        .map(|digits: String| digits.parse::<u32>().unwrap())
    //};

    many1(line::<'a>().map(|s| s.into()))
}

fn knot<'a, Input>() -> impl Parser<Input, Output = Knot>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    string("==")
        .with(line::<'a>())
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
    // TODO: parse the first knot, which has no title
    let lines = ();
    knot_without_title()
        .and(many::<Vec<_>, _, _>(knot()))
        .map(|(into_knot, other_knots)| {
            //dbg!(c);
            Story::default()
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
fn test_1() {
    //let mut result = story().parse(include_str!("../stories/basic_story.ink"));
    //dbg!(result);
    dbg!(line().parse("this is one line with no line endings"));
    dbg!(line().parse("this is one line with one line ending\n"));
    dbg!(line().parse("this is one line with line endings\r\n"));
    dbg!(line().parse(include_str!("../stories/basic_story.ink")));
    dbg!(lines().parse(include_str!("../stories/basic_story.ink")));
    dbg!(parse_story(include_str!("../stories/basic_story.ink")));

    //dbg!(date().parse("2015-08-02"));

    todo!()
}

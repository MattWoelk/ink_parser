#![warn(rust_2018_idioms)]
#![allow(unused)]
use combine::parser::char::newline;
use combine::parser::char::{char, digit};
use combine::parser::range::range;
use combine::parser::repeat::take_until;
use combine::{attempt, many, many1, ParseError, Parser, RangeStream, Stream};

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

fn knot<'a, Input>() -> impl Parser<Input, Output = Knot>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    take_until::<String, Input, _>(attempt(range("=="))).map(|c| {
        Knot::default() // {
                        //title: "?".to_string(),
                        //lines: lines().parse(&*c).unwrap().0,
                        //})
    })
}

fn line<'a, Input>() -> impl Parser<Input, Output = String>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    take_until(newline())
}

fn lines<'a, Input>() -> impl Parser<Input, Output = Vec<String>>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(line())
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
fn do_thing() {
    let mut result = knot().parse(include_str!("../stories/basic_story.ink"));
    dbg!(result);

    dbg!(date().parse("2015-08-02"));

    todo!()
}

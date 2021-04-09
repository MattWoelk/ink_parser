#![warn(rust_2018_idioms)]
//use combine::parser::range::take_while;
//use combine::parser::token::Token;
use combine::parser::char::{char, digit};
use combine::{many, ParseError, Parser, Stream};

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

//fn build_knot_parser<Input>() -> impl Parser<Input, Output = Knot>
//where
//    Input: Stream<Token = char>,
//    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
//{
//    let tool = take_while(|c: char| c.is_alphabetic());
//
//    tool.map(|c| Knot::default())
//}
//
//fn parse_knot(text: &str) -> Knot {
//    let mut parser = take_while(|c: char| c.is_alphabetic());
//    let result = dbg!(parser.parse(text));
//    let a: dyn Extend<_> = ();
//
//    Knot::default()
//}

fn knot<Input>() -> impl Parser<Input, Output = Knot>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    two_digits().map(|_| Knot::default())
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
    // Necessary due to rust-lang/rust#24159
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
fn date<Input>() -> impl Parser<Input, Output = Date>
where
    Input: Stream<Token = char>,
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

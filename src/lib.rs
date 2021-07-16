#![warn(rust_2018_idioms)]

mod tests;

use combine::parser::char::{char, spaces, string};
use combine::parser::choice::choice;
use combine::parser::repeat::take_until;
use combine::{
    attempt, eof, many, many1, optional, value, ParseError, Parser, RangeStream, Stream,
};
use combine::{not_followed_by, satisfy, EasyParser};
use std::collections::BTreeMap;

// TODO: get rid of comments, using a nice function that I can use everywhere
//       - maybe rest_of_the_line can absorb comments, and that's good enough?

// TODO: get rid of trailing "==="s on knot titles
// TODO: pass state along, so when parsing fails I can debug it.
// TODO: variables, conditionals, etc.

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct DialogLine {
    text: String,
    tags: Vec<String>,
}

type KnotTitle = String;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Divert {
    knot_title: KnotTitle,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Choice {
    text: String,
    dialog_lines: Vec<DialogLine>,
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
    dialog_lines: Vec<DialogLine>,
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

impl From<&str> for DialogLine {
    fn from(s: &str) -> Self {
        Self {
            text: s.to_string(),
            tags: vec![],
        }
    }
}

fn newline_character<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    char('\n').or(char('\r').skip(char('\n'))).map(|_| ())
}

/// grabs the rest of the line, and consumes any trailing newline marker
fn rest_of_the_line<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    // TODO: can this be simplified? I just want to grab everything until we hit \n or \r
    //       - maybe like we do in tag()?
    many1::<String, _, _>(satisfy(|c| c != '\n' && c != '\r')).skip(optional(newline_character()))
}

fn single_line_comment<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    attempt(string("//"))
        .with(many1::<String, _, _>(satisfy(|c| c != '\n' && c != '\r')))
        .with(value(()))
}

fn multi_line_comment<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    attempt(string("/*"))
        .with(take_until::<String, _, _>(string("*/")))
        .with(string("*/"))
        .with(value(()))
}

fn tag<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    attempt(string("#"))
        .with(take_until::<String, _, _>(choice((
            string("#"),
            string("\n"),
            string("\r\n"),
            eof().map(|_| ""),
        ))))
        .map(|x| x.trim().to_string())
}

// TODO: do this like I'm doing in dialog_lines(), then have dialog_lines() use this function
fn rest_of_the_line_ignoring_comments_with_tags<Input>() -> impl Parser<Input, Output = DialogLine>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    // TODO: this needs to ignore inline comments. :/
    //       maybe we call this a bunch of times, then flatten the resulting Strings?
    //       ... though having a newline within the multi_line_comment seems to break things ... which is unexpected ...
    many1::<String, _, _>(satisfy(|c| c != '\n' && c != '\r' && c != '/' && c != '#'))
        // TODO: this needs to be "//", not just a single slash, or that's going to cause prooooooblems...
        // TODO: can I use take_until, like we're doing in tag()?
        .and(optional(many1::<Vec<String>, _, _>(tag())))
        .skip(optional(single_line_comment()))
        .skip(optional(multi_line_comment()))
        .skip(optional(choice((string("\n"), string("\r\n")))))
        .map(|(s, tags)| DialogLine {
            text: s.trim_end().into(),
            tags: tags.unwrap_or_default(),
        })
}

// TODO: make this out of the good bits of dialog_lines(), or delete it entirely???
pub fn dialog_line<Input>() -> impl Parser<Input, Output = DialogLine>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    spaces().with(rest_of_the_line_ignoring_comments_with_tags())
}

/// Must call spaces() before calling this,
/// because we can't make it optional() if spaces() consumes input
/// TODO: fix that, so we can have spaces() at the start of this properly. (see below)
fn dialog_lines<'a, Input>() -> impl Parser<Input, Output = Vec<DialogLine>>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1::<Vec<DialogLine>, _, _>(
        //attempt(spaces()).with
        // TODO: I would love to have this here, but it consumes input,
        //                        so we can't have dialog_lines() be optional()
        not_followed_by(choice((string("->"), string("+"))))
            .skip(optional(single_line_comment()))
            .skip(optional(newline_character()))
            .skip(spaces())
            .skip(optional(
                multi_line_comment().skip(optional(newline_character())),
            )) // TODO: this section is the new stuff. Fix it.
            .skip(not_followed_by(choice((
                attempt(string("->")),
                attempt(string("+")),
            )))) // TODO: when this fails, we don't gracefully do anything...
            .with(many1::<String, _, _>(
                many1::<String, _, _>(
                    satisfy(|c| c != '\n' && c != '\r' && c != '/'), // TODO: do better than this, for comments
                ) // TODO: do we really need to find a character before a multi line comment? Hmm...
                .skip(optional(many1::<(), _, _>(multi_line_comment()))),
            ))
            .skip(optional(single_line_comment()))
            // TODO: I would love to put divert() right in here; not sure why I can't
            //.skip(not_followed_by(string("+"))) // TODO: does this do anything?
            //.skip(not_followed_by(string("->"))) // TODO: does this do anything?
            .skip(spaces())
            .map(|s| DialogLine {
                text: s.trim().to_string(),
                tags: vec![],
            }),
        // TODO: desired functionality:
        //       - parse N of these:
        //           - parse multi-line comment (0 or more)
        //           - parse text (1 or more)
        //       - parse single line comment (optional)
        //       - combine all the text into one string, and return that
    )
}

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
            .with(many1::<String, _, _>(satisfy(|c| {
                c != '\n' && c != '\r' && c != ' '
            })))
            .skip(optional(char(' ').and(string("==").skip(many::<
                String,
                _,
                _,
            >(
                char('=')
            )))))
            .skip(newline_character()),
    )
}

fn knot_end<'a, Input>() -> impl Parser<Input, Output = KnotEnding>
where
    Input: RangeStream<Token = char, Range = &'a str>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    spaces().with(choice((
        many1::<Vec<Choice>, _, _>(parse_choice()).map(KnotEnding::CHOICES),
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

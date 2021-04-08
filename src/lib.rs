use combine::parser::char::{letter, space};
use combine::{many1, sep_by, Parser};

#[test]
fn do_thing() {
    // Construct a parser that parses *many* (and at least *1) *letter*s
    let word = many1(letter());

    // Construct a parser that parses many *word*s where each word is *separated by* a (white)*space*
    let mut parser = sep_by(word, space())
        // Combine can collect into any type implementing `Default + Extend` so we need to assist rustc
        // by telling it that `sep_by` should collect into a `Vec` and `many1` should collect to a `String`
        .map(|mut words: Vec<String>| words.pop());
    let result = parser.parse("Pick up that word!");
    // `parse` returns `Result` where `Ok` contains a tuple of the parsers output and any remaining input.
    assert_eq!(result, Ok((Some("word".to_string()), "!")));
}

#[cfg(test)]
use crate::*;
#[cfg(test)]
use combine::error::StringStreamError;
use combine::stream::PointerOffset;
#[cfg(test)]
use pretty_assertions::{assert_eq, assert_ne};

// TODO: use this to both make errors easier to read, and easier to test
#[cfg(test)]
fn pointer_offset_to_row_col(pointer_offset: PointerOffset<str>, text: &str) -> (usize, usize) {
    let index = pointer_offset.translate_position(text);

    index_to_row_col(index, text)
}

#[cfg(test)]
/// row and column are 1-indexed
fn index_to_row_col(index: usize, text: &str) -> (usize, usize) {
    if text.len() == 0 {
        return (0, 0); // TODO: should this return an error instead? or return (1,1)?
    }

    //let text = &;
    let last_newline = text[..index + 1].rfind('\n').unwrap_or(0);
    let number_of_newlines = text[..index].chars().filter(|&c| c == '\n').count();

    dbg!(index);
    dbg!(last_newline);
    dbg!(number_of_newlines);

    let row = number_of_newlines + 1;
    let column = index - last_newline + 1 - if number_of_newlines > 0 { 1 } else { 0 };

    (row, column)
}

#[test]
fn text_index_to_row_col() {
    fn find_x(text: &str) -> (usize, usize) {
        index_to_row_col(text.find('X').unwrap(), text)
    }

    assert_eq!(find_x("X"), (1, 1));
    assert_eq!(find_x("     X      "), (1, 6));
    assert_eq!(find_x("     X\n     "), (1, 6));
    assert_eq!(find_x("     \nX     "), (2, 1));
    assert_eq!(find_x("     \n X    "), (2, 2));
    assert_eq!(find_x("\n     X      "), (2, 6));
    assert_eq!(find_x("\n\n     X      "), (3, 6));
    assert_eq!(find_x("\n    \nX      "), (3, 1));
    assert_eq!(find_x(" \n  \nX"), (3, 1));
}

#[test]
fn test_dialog_line() {
    assert_eq!(
        dialog_line().easy_parse("no line endings"),
        Ok(("no line endings".to_string(), ""))
    );
    assert_eq!(
        dialog_line().easy_parse("one line ending\n"),
        Ok(("one line ending".to_string(), ""))
    );
    assert_eq!(
        dialog_line().easy_parse("both line endings\r\n"),
        Ok(("both line endings".to_string(), ""))
    );
    assert_eq!(
        dialog_line().easy_parse("          line starting with spaces\r\n"),
        Ok(("          line starting with spaces".to_string(), ""))
    );
    assert_eq!(
        dialog_line().easy_parse("       \n \r\n   line starting with newlines and spaces\r\n"),
        Ok((
            "       ".to_string(),
            " \r\n   line starting with newlines and spaces\r\n"
        ))
    );
}

#[test]
fn test_divert() {
    assert_eq!(
        divert().easy_parse("-> yeah"),
        Ok((
            Divert {
                knot_title: "yeah".to_string()
            },
            ""
        ))
    );

    assert_eq!(
        divert().easy_parse("    -> yeah"),
        Ok((
            Divert {
                knot_title: "yeah".to_string()
            },
            ""
        ))
    );

    //let text = "===";
    //assert_eq!(
    //    pointer_offset_to_row_col(divert().easy_parse(text).unwrap_err().position, text),
    //    (1, 1)
    //);
}

#[test]
fn test_choice() {
    assert_eq!(
        parse_choice().easy_parse("+ yeah\n-> divert"),
        Ok((
            Choice {
                text: "yeah".to_string(),
                dialog_lines: vec![],
                divert: Divert {
                    knot_title: "divert".to_string()
                }
            },
            ""
        ))
    );

    assert_eq!(
        parse_choice().easy_parse("+ yeah\n  one\ntwo\n     three\n-> divert"),
        Ok((
            Choice {
                text: "yeah".to_string(),
                dialog_lines: vec!["one".to_string(), "two".to_string(), "three".to_string()],
                divert: Divert {
                    knot_title: "divert".to_string()
                }
            },
            ""
        ))
    );

    assert_eq!(
        parse_choice().easy_parse("+ yeah\n  one\ntwo\n-> paris"),
        Ok((
            Choice {
                text: "yeah".to_string(),
                dialog_lines: vec!["one".to_string(), "two".to_string()],
                divert: Divert {
                    knot_title: "paris".to_string()
                }
            },
            ""
        ))
    );

    //assert_eq!(
    //    lines().easy_parse("  one\ntwo\n   + paris"),
    //    Ok((vec!["one".to_string(), "two".to_string(),], "+ paris"))
    //);

    //assert_eq!(
    //    lines().easy_parse("  one\ntwo\n   -> paris"),
    //    Ok((vec!["one".to_string(), "two".to_string(),], "-> paris"))
    //);

    assert_eq!(
        parse_choice().easy_parse("+ yeah\n  one\ntwo\n   -> paris"),
        Ok((
            Choice {
                text: "yeah".to_string(),
                dialog_lines: vec!["one".to_string(), "two".to_string()],
                divert: Divert {
                    knot_title: "paris".to_string()
                }
            },
            ""
        ))
    );
}

//#[test]
//fn test_knot_title() {
//    assert_eq!(
//        knot_title().easy_parse(" === title1"),
//        Ok(("title1".to_string(), ""))
//    )
//}
//
//#[test]
//fn test_story() {
//    assert_eq!(
//        story().easy_parse(include_str!("../stories/two_knots.ink")),
//        Ok((
//            Story {
//                knots: btreemap! {
//                    "INTRO".to_string() => Knot {
//                        title: "INTRO".to_string(),
//                        lines: vec![
//                            "to paris".to_string()
//                        ],
//                        choices: vec![],
//                        divert: Some("paris".to_string()),
//                    },
//                    "paris".to_string() => Knot {
//                        title: "paris".to_string(),
//                        lines: vec![
//                            "We are in paris.".to_string()
//                        ],
//                        choices: vec![],
//                        divert: Some("ending".to_string()),
//                    },
//                    "ending".to_string() => Knot {
//                        title: "ending".to_string(),
//                        lines: vec![
//                            "THE END now.".to_string()
//                        ],
//                        choices: vec![],
//                        divert: Some("END".to_string()),
//                    },
//                }
//            },
//            ""
//        ))
//    );
//
//    assert_eq!(
//        story().easy_parse(include_str!("../stories/two_knots_with_choices.ink")),
//        Ok((
//            Story {
//                knots: btreemap! {
//                    "INTRO".to_string() => Knot {
//                        title: "INTRO".to_string(),
//                        lines: vec![
//                            "to paris?".to_string()
//                        ],
//                        choices: vec![
//                            ("yeah".to_string(), Some(vec!["yes, please".to_string()]), Some("paris".to_string())),
//                            ("no".to_string(), Some(vec!["no, thank you".to_string()]), Some("ending".to_string())),
//                        ],
//                        divert: None,
//                    },
//                    "paris".to_string() => Knot {
//                        title: "paris".to_string(),
//                        lines: vec![
//                            "We are in paris.".to_string()
//                        ],
//                        choices: vec![],
//                        divert: Some("ending".to_string()),
//                    },
//                    "ending".to_string() => Knot {
//                        title: "ending".to_string(),
//                        lines: vec![
//                            "THE END now.".to_string()
//                        ],
//                        choices: vec![],
//                        divert: Some("END".to_string()),
//                    },
//                }
//            },
//            ""
//        ))
//    );
//
//    assert_eq!(
//        story().easy_parse(include_str!("../stories/basic_story.ink")),
//        Ok((
//            Story {
//                knots: btreemap! {
//                    "INTRO".to_string() => Knot {
//                        title: "INTRO".to_string(),
//                        lines: vec![
//                            "Want to go to paris?".to_string(),
//                            "PLEASE!?".to_string(),
//                            "will you?????????".to_string(),
//                        ],
//                        choices: vec![
//                            (
//                                "yeah!".to_string(),
//                                None,
//                                Some(
//                                    "paris".to_string(),
//                                ),
//                            ),
//                            (
//                                "\"Around the world, Monsieur?\"".to_string(),
//                                Some(
//                                    vec![
//                                        "I was utterly astonished.".to_string(),
//                                        "\"You are in jest!\" I told him in dignified affront. THIS IS A VERY LONG LINE OF TEXT SO LONG ON MY SO LOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOONG!".to_string(),
//                                    ],
//                                ),
//                                Some(
//                                    "ending".to_string(),
//                                ),
//                            ),
//                        ],
//                        divert: None,
//                    },
//                    "ending".to_string() => Knot {
//                        title: "ending".to_string(),
//                        lines: vec![
//                            "THE END now.".to_string(),
//                        ],
//                        choices: vec![],
//                        divert: Some(
//                            "END".to_string(),
//                        ),
//                    },
//                    "paris".to_string() => Knot {
//                        title: "paris".to_string(),
//                        lines: vec![
//                            "We are in paris.".to_string(),
//                        ],
//                        choices: vec![],
//                        divert: Some(
//                            "ending".to_string(),
//                        ),
//                    },
//                }
//            },
//            ""
//        ))
//    );
//
//    assert_eq!(
//        story().easy_parse(include_str!("../stories/spaces_before_divert.ink")),
//        Ok((
//            Story {
//                knots: btreemap! {
//                    "INTRO".to_string() => Knot {
//                        title: "INTRO".to_string(),
//                        lines: vec![
//                            "a thing".to_string()
//                        ],
//                        choices: vec![
//                            (
//                                "🙁".to_string(),
//                                None,
//                                Some("ending".to_string())
//                            )
//                        ],
//                        divert: None
//                    }
//                }
//            },
//            ""
//        ))
//    );
//
//    // TODO: need to have "stitches" (sub knots) first
//    //assert_eq!(
//    //    story().easy_parse(include_str!("../stories/too_many_blank_lines.ink")),
//    //    Ok((Story::default(), ""))
//    //);
//}
//

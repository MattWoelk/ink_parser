#[cfg(test)]
use crate::*;
#[cfg(test)]
use combine::easy::Error;
#[cfg(test)]
use combine::easy::Errors;
#[cfg(test)]
use combine::stream::PointerOffset;
#[cfg(test)]
use maplit::btreemap;
#[cfg(test)]
use pretty_assertions::assert_eq;

// TODO: use this to both make errors easier to read, and easier to test (and test that it works as it should...)
//       BUT it seem like it's wrong, and the index will overflow...
//       Could it be that the index is in bytes instead of ... other things?
#[cfg(test)]
fn pointer_offset_to_row_col(pointer_offset: PointerOffset<str>, text: &str) -> (usize, usize) {
    let index = pointer_offset.translate_position(text);

    index_to_row_col(index, text)
}

#[cfg(test)]
fn map_combine_error<'a>(
    errors: Errors<char, &'a str, PointerOffset<str>>,
    text: &str,
) -> ((usize, usize), Vec<Error<char, &'a str>>) {
    (
        pointer_offset_to_row_col(errors.position, text),
        errors.errors,
    )
}

#[cfg(test)]
/// row and column are 1-indexed
fn index_to_row_col(index: usize, text: &str) -> (usize, usize) {
    if text.len() == 0 {
        return (0, 0); // TODO: should this return an error instead? or return (1,1)?
    }

    if index > text.len() - 1 {
        return (0, 1); // TODO: an error instead?
    }

    dbg!(index);

    //let text = &;
    let last_newline = text[0..index + 1].rfind('\n').unwrap_or(0);
    let number_of_newlines = text[..index].chars().filter(|&c| c == '\n').count();

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
    //color_backtrace::install();

    assert_eq!(
        dialog_line().easy_parse("no line endings"),
        Ok((
            DialogLine {
                text: "no line endings".to_string(),
                tags: vec![]
            },
            ""
        ))
    );

    assert_eq!(
        dialog_line().easy_parse("one line ending\n"),
        Ok((
            DialogLine {
                text: "one line ending".to_string(),
                tags: vec![]
            },
            ""
        ))
    );

    assert_eq!(
        dialog_line().easy_parse("both line endings\r\n"),
        Ok((
            DialogLine {
                text: "both line endings".to_string(),
                tags: vec![]
            },
            ""
        ))
    );

    assert_eq!(
        dialog_line().easy_parse("          line starting with spaces\r\n"),
        Ok((
            DialogLine {
                text: "line starting with spaces".to_string(),
                tags: vec![]
            },
            ""
        ))
    );

    assert_eq!(
        dialog_line().easy_parse("       \n \r\n   line starting with newlines and spaces\r\n"),
        Ok((
            DialogLine {
                text: "line starting with newlines and spaces".to_string(),
                tags: vec![]
            },
            ""
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
    let text = "-> divert";
    assert_eq!(
        parse_choice()
            .easy_parse(text)
            .map_err(|e| map_combine_error(e, text))
            .unwrap_err()
            .0,
        (1, 1)
    );

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
        parse_choice().easy_parse("+ yeah\n   -> divert"),
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
                dialog_lines: vec!["one".into(), "two".into(), "three".into()],
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
                dialog_lines: vec!["one".into(), "two".into()],
                divert: Divert {
                    knot_title: "paris".to_string()
                }
            },
            ""
        ))
    );

    assert_eq!(
        parse_choice().easy_parse("+ yeah\n  one\ntwo\n   -> paris"),
        Ok((
            Choice {
                text: "yeah".to_string(),
                dialog_lines: vec!["one".into(), "two".into()],
                divert: Divert {
                    knot_title: "paris".to_string()
                }
            },
            ""
        ))
    );

    // consume and ignore empty dialog lines
    assert_eq!(
        parse_choice().easy_parse("+ yeah\n  \n  one\n   \ntwo\n   -> paris"),
        Ok((
            Choice {
                text: "yeah".to_string(),
                dialog_lines: vec!["one".into(), "two".into()],
                divert: Divert {
                    knot_title: "paris".to_string()
                }
            },
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
                        dialog_lines: vec![
                            "to paris".into()
                        ],
                        ending: KnotEnding::DIVERT("paris".into()),
                    },
                    "paris".to_string() => Knot {
                        title: "paris".to_string(),
                        dialog_lines: vec![
                            "We are in paris.".into()
                        ],
                        ending: KnotEnding::DIVERT(
                            "ending".into()
                        ),
                    },
                    "ending".to_string() => Knot {
                        title: "ending".to_string(),
                        dialog_lines: vec![
                            "THE END now.".into()
                        ],
                        ending: KnotEnding::DIVERT(
                            "END".into()
                        )
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
                        dialog_lines: vec![
                            "to paris?".into()
                        ],
                        ending: KnotEnding::CHOICES(vec![
                            Choice {
                                text: "yeah".to_string(),
                                dialog_lines: vec!["yes, please".into()],
                                divert: "paris".into(),
                            },
                            Choice {
                                text: "no".to_string(),
                                dialog_lines: vec!["no, thank you".into()],
                                divert: "ending".into(),
                            }
                        ]),
                    },
                    "paris".to_string() => Knot {
                        title: "paris".to_string(),
                        dialog_lines: vec![
                            "We are in paris.".into()
                        ],
                        ending: KnotEnding::DIVERT(
                            "ending".into()
                        ),
                    },
                    "ending".to_string() => Knot {
                        title: "ending".to_string(),
                        dialog_lines: vec![
                            "THE END now.".into()
                        ],
                        ending: KnotEnding::DIVERT(
                            "END".into()
                        )
                    },
                }
            },
            ""
        ))
    );

    assert_eq!(
        story().easy_parse(include_str!("../stories/basic_story.ink")),
        Ok((
            Story {
                knots: btreemap! {
                    "INTRO".to_string() => Knot {
                        title: "INTRO".to_string(),
                        dialog_lines: vec![
                            "Want to go to paris?".into(),
                            "PLEASE!?".into(),
                            "will you?????????".into(),
                        ],
                        ending: KnotEnding::CHOICES(vec![
                            Choice {
                                text: "yeah!".to_string(),
                                dialog_lines: vec![],
                                divert: "paris".into(),
                            },
                            Choice {
                                text: "\"Around the world, Monsieur?\"".to_string(),
                                dialog_lines: vec![
                                    "I was utterly astonished.".into(),
                                    concat!("\"You are in jest!\" I told him in dignified affront. ",
                                        "THIS IS A VERY LONG LINE OF TEXT SO LONG ON MY SO LOOOOOO",
                                        "OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO",
                                        "OOOOOOOOOOOOOOOOOOOOOOOONG!").into(),
                                ],
                                divert: "ending".into(),
                            },
                        ]),
                    },
                    "paris".to_string() => Knot {
                        title: "paris".to_string(),
                        dialog_lines: vec![
                            "We are in paris.".into()
                        ],
                        ending: KnotEnding::DIVERT(
                            "ending".into()
                        ),
                    },
                    "ending".to_string() => Knot {
                        title: "ending".to_string(),
                        dialog_lines: vec![
                            "THE END now.".into()
                        ],
                        ending: KnotEnding::DIVERT(
                            "END".into()
                        )
                    },
                }
            },
            ""
        ))
    );

    assert_eq!(
        story().easy_parse(include_str!("../stories/spaces_before_divert.ink")),
        Ok((
            Story {
                knots: btreemap! {
                    "INTRO".to_string() => Knot {
                        title: "INTRO".to_string(),
                        dialog_lines: vec![
                            "a thing".into()
                        ],
                        ending: KnotEnding::CHOICES(vec![
                            Choice {
                                text: "🙁".to_string(),
                                dialog_lines: vec![],
                                divert: "ending".into(),
                            }
                        ]),
                    }
                }
            },
            ""
        ))
    );

    // TODO: need to have "stitches" (sub knots) first
    //assert_eq!(
    //    story().easy_parse(include_str!("../stories/too_many_blank_lines.ink")),
    //    Ok((Story::default(), ""))
    //);
}

#[test]
fn test_comments() {
    assert_eq!(
        multi_line_comment().easy_parse("/* comment */"),
        Ok(((), ""))
    );

    assert_eq!(
        multi_line_comment().easy_parse("/*\n comment \n*/"),
        Ok(((), ""))
    );

    assert_eq!(
        single_line_comment().easy_parse("//cool\n text \n"),
        Ok(((), " text \n"))
    );

    assert_eq!(
        rest_of_the_line_ignoring_comments_with_tags().easy_parse("text // comment"),
        Ok(("text".into(), ""))
    );

    assert_eq!(
        rest_of_the_line_ignoring_comments_with_tags().easy_parse("text /* comment */"),
        Ok(("text".into(), ""))
    );

    assert_eq!(
        dialog_lines().easy_parse("text /* comment\n */\ncool // comment 2\n yeah"),
        Ok((vec!["text".into(), "cool".into(), "yeah".into()], ""))
    );

    assert_eq!(
        dialog_lines().easy_parse("dialog 1\n// comment 1\ndialog 2"),
        Ok((vec!["dialog 1".into(), "dialog 2".into()], ""))
    );

    assert_eq!(
        dialog_lines().easy_parse("text /* comment\n */\ncool // comment 2\n yeah"),
        Ok((vec!["text".into(), "cool".into(), "yeah".into()], ""))
    );

    //    //    assert_eq!(
    //    //        dialog_lines().easy_parse(
    //    //            "dialog 1
    //    //// comment 1
    //    //dialog 2
    //    ///*
    //    //    comment 2
    //    //*/
    //    //dialog 3 // comment 3
    //    //dialog /* comment 4 */4"
    //    //        ),
    //    //        Ok((
    //    //            vec![
    //    //                "dialog 1".into(),
    //    //                "dialog 2".into(),
    //    //                "dialog 3".into(),
    //    //                "dialog 4".into()
    //    //            ],
    //    //            ""
    //    //        ))
    //    //    );

    assert_eq!(
        knot_without_title().easy_parse(
            "dialog 1
// comment 1
-> END"
        ),
        Ok((
            Knot {
                title: "INTRO".to_string(),
                dialog_lines: vec!["dialog 1".into()],
                ending: KnotEnding::DIVERT("END".into())
            },
            ""
        ))
    );

    //    //    assert_eq!(
    //    //        knot_without_title().easy_parse(
    //    //            "dialog 1
    //    //// comment 1
    //    //dialog 2
    //    ///*
    //    //    comment 2
    //    //*/
    //    //dialog 3 // comment 3
    //    //dialog /* comment 4 */4
    //    //-> END"
    //    //        ),
    //    //        Ok((
    //    //            Knot {
    //    //                title: "INTRO".to_string(),
    //    //                dialog_lines: vec![
    //    //                    "dialog 1".into(),
    //    //                    "dialog 2".into(),
    //    //                    "dialog 3".into(),
    //    //                    "dialog 4".into()
    //    //                ],
    //    //                ending: KnotEnding::DIVERT("END".into())
    //    //            },
    //    //            ""
    //    //        ))
    //    //    );

    ////    assert_eq!(
    ////        story().easy_parse(
    ////            "dialog 1
    ////// comment 1
    ////dialog 2
    /////*
    ////    comment 2
    ////*/
    ////dialog 3 // comment 3
    ////dialog /* comment 4 */4
    ////-> END"
    ////        ),
    ////        Ok((
    ////            Story {
    ////                knots: btreemap! {
    ////                    "INTRO".to_string() => Knot {
    ////                        title: "INTRO".to_string(),
    ////                        dialog_lines: vec![
    ////                            "dialog 1".into(),
    ////                            "dialog 2".into(),
    ////                            "dialog 3".into(),
    ////                            "dialog 4".into(),
    ////                        ],
    ////                        ending: KnotEnding::DIVERT(
    ////                            "END".into()
    ////                        ),
    ////                    }
    ////                }
    ////            },
    ////            ""
    ////        ))
    ////    );
}

#[test]
fn test_knot() {
    assert_eq!(
        knot_title().easy_parse(" === title1\nthing\n"),
        Ok(("title1".to_string(), "thing\n"))
    );

    assert!(knot_title()
        .easy_parse("=== Knot name has spaces\n")
        .is_err());

    assert!(knot_title().easy_parse("=== Knot =\n").is_err());

    assert_eq!(
        knot_title().easy_parse("=== Knot\nbody\n"),
        Ok(("Knot".into(), "body\n"))
    );

    assert_eq!(
        knot_title().easy_parse("=== Knot ===\nbody\n"),
        Ok(("Knot".into(), "body\n"))
    );
}

#[test]
fn test_tags() {
    assert!(tag().easy_parse("no line endings").is_err());

    assert_eq!(
        many1::<Vec<String>, _, _>(tag()).easy_parse("# one # two"),
        Ok((vec!["one".into(), "two".into()], ""))
    );

    assert_eq!(
        dialog_line().easy_parse("Passepartout: Really, Monsieur. # surly # really_monsieur.ogg"),
        Ok((
            DialogLine {
                text: "Passepartout: Really, Monsieur.".to_string(),
                tags: vec!["surly".into(), "really_monsieur.ogg".into()]
            }
            .into(),
            ""
        ))
    );
}

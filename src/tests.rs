#[cfg(test)]
use crate::*;
#[cfg(test)]
use combine::error::StringStreamError;
#[cfg(test)]
use pretty_assertions::{assert_eq, assert_ne};

#[test]
fn test_line() {
    assert_eq!(
        line().parse("+ option"),
        Err(StringStreamError::UnexpectedParse)
    );
    assert_eq!(
        line().parse("-> divert"),
        Err(StringStreamError::UnexpectedParse)
    );
    assert_eq!(
        line().easy_parse("no line endings"),
        Ok(("no line endings".to_string(), ""))
    );
    assert_eq!(
        line().easy_parse("one line ending\n"),
        Ok(("one line ending".to_string(), ""))
    );
    assert_eq!(
        line().easy_parse("both line endings\r\n"),
        Ok(("both line endings".to_string(), ""))
    );
    assert_eq!(
        line().easy_parse("          line starting with spaces\r\n"),
        Ok(("          line starting with spaces".to_string(), ""))
    );
    assert_eq!(
        line().easy_parse("       \n \r\n   line starting with newlines and spaces\r\n"),
        Ok((
            "       ".to_string(),
            " \r\n   line starting with newlines and spaces\r\n"
        ))
    );
}

#[test]
fn test_divert() {
    assert_eq!(
        divert_line().easy_parse("-> yeah"),
        Ok((LineTypes::DIVERT(("yeah".to_string())), ""))
    );

    //assert_eq!(
    //    divert_line().easy_parse("    -> yeah"),
    //    Ok((LineTypes::DIVERT(("yeah".to_string())), ""))
    //);
}

#[test]
fn test_choice() {
    assert_eq!(
        choice_lines().easy_parse("+ yeah"),
        Ok((LineTypes::CHOICE(("yeah".to_string(), None, None)), ""))
    );

    assert_eq!(
        choice_lines().easy_parse("+ yeah\n  one\ntwo\n     three"),
        Ok((
            LineTypes::CHOICE((
                "yeah".to_string(),
                Some(vec![
                    "one".to_string(),
                    "two".to_string(),
                    "three".to_string()
                ]),
                None
            )),
            ""
        ))
    );

    assert_eq!(
        choice_lines().easy_parse("+ yeah\n  one\ntwo\n-> paris"),
        Ok((
            LineTypes::CHOICE((
                "yeah".to_string(),
                Some(vec!["one".to_string(), "two".to_string(),]),
                Some("paris".to_string())
            )),
            ""
        ))
    );

    assert_eq!(
        lines().easy_parse("  one\ntwo\n   + paris"),
        Ok((vec!["one".to_string(), "two".to_string(),], "+ paris"))
    );

    assert_eq!(
        lines().easy_parse("  one\ntwo\n   -> paris"),
        Ok((vec!["one".to_string(), "two".to_string(),], "-> paris"))
    );

    assert_eq!(
        choice_lines().easy_parse("+ yeah\n  one\ntwo\n   -> paris"),
        Ok((
            LineTypes::CHOICE((
                "yeah".to_string(),
                Some(vec!["one".to_string(), "two".to_string(),]),
                Some("paris".to_string())
            )),
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
                        lines: vec![
                            "to paris".to_string()
                        ],
                        choices: vec![],
                        divert: Some("paris".to_string()),
                    },
                    "paris".to_string() => Knot {
                        title: "paris".to_string(),
                        lines: vec![
                            "We are in paris.".to_string()
                        ],
                        choices: vec![],
                        divert: Some("ending".to_string()),
                    },
                    "ending".to_string() => Knot {
                        title: "ending".to_string(),
                        lines: vec![
                            "THE END now.".to_string()
                        ],
                        choices: vec![],
                        divert: Some("END".to_string()),
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
                        lines: vec![
                            "to paris?".to_string()
                        ],
                        choices: vec![
                            ("yeah".to_string(), Some(vec!["yes, please".to_string()]), Some("paris".to_string())),
                            ("no".to_string(), Some(vec!["no, thank you".to_string()]), Some("ending".to_string())),
                        ],
                        divert: None,
                    },
                    "paris".to_string() => Knot {
                        title: "paris".to_string(),
                        lines: vec![
                            "We are in paris.".to_string()
                        ],
                        choices: vec![],
                        divert: Some("ending".to_string()),
                    },
                    "ending".to_string() => Knot {
                        title: "ending".to_string(),
                        lines: vec![
                            "THE END now.".to_string()
                        ],
                        choices: vec![],
                        divert: Some("END".to_string()),
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
                        lines: vec![
                            "Want to go to paris?".to_string(),
                            "PLEASE!?".to_string(),
                            "will you?????????".to_string(),
                        ],
                        choices: vec![
                            (
                                "yeah!".to_string(),
                                None,
                                Some(
                                    "paris".to_string(),
                                ),
                            ),
                            (
                                "\"Around the world, Monsieur?\"".to_string(),
                                Some(
                                    vec![
                                        "I was utterly astonished.".to_string(),
                                        "\"You are in jest!\" I told him in dignified affront. THIS IS A VERY LONG LINE OF TEXT SO LONG ON MY SO LOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOONG!".to_string(),
                                    ],
                                ),
                                Some(
                                    "ending".to_string(),
                                ),
                            ),
                        ],
                        divert: None,
                    },
                    "ending".to_string() => Knot {
                        title: "ending".to_string(),
                        lines: vec![
                            "THE END now.".to_string(),
                        ],
                        choices: vec![],
                        divert: Some(
                            "END".to_string(),
                        ),
                    },
                    "paris".to_string() => Knot {
                        title: "paris".to_string(),
                        lines: vec![
                            "We are in paris.".to_string(),
                        ],
                        choices: vec![],
                        divert: Some(
                            "ending".to_string(),
                        ),
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
                        lines: vec![
                            "a thing".to_string()
                        ],
                        choices: vec![
                            (
                                "🙁".to_string(),
                                None,
                                Some("ending".to_string())
                            )
                        ],
                        divert: None
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

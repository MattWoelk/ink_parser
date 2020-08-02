use nom::branch::alt;
use nom::bytes::complete::{take_while, take_while1, take_while_m_n};
use nom::sequence::{preceded, terminated};
use nom::IResult;

/// whitespace that is not a newline character
fn nbsp(input: &str) -> IResult<&str, &str> {
    take_while(|c| " \t".contains(c))(input)
}

/// newline characters
fn newline(input: &str) -> IResult<&str, &str> {
    take_while(|c| "\n\r".contains(c))(input)
}

fn title(input: &str) -> IResult<&str, &str> {
    take_while1(|c| !"=\n\r".contains(c))(input)
}

pub fn parse_knot_header(input: &str) -> IResult<&str, &str> {
    let equal_signs = terminated(take_while_m_n(2, 999, |c| c == '='), nbsp);

    let (input, _) = preceded(nbsp, terminated(&equal_signs, nbsp))(input)?;

    let (input, title) = terminated(title, alt((&equal_signs, newline)))(input)?;

    Ok((input, title.trim_end()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_knot_header() {
        assert_eq!(
            parse_knot_header("== train station"),
            Ok(("", "train station"))
        );

        assert_eq!(
            parse_knot_header("== train station =="),
            Ok(("", "train station"))
        );

        assert_eq!(
            parse_knot_header(" === train station === "),
            Ok(("", "train station"))
        );

        assert!(parse_knot_header(" = train station = ").is_err());
    }
}

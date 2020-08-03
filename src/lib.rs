use nom::bytes::complete::{is_not, take_while, take_while1, take_while_m_n};
use nom::combinator::opt;
use nom::sequence::{preceded, terminated, tuple};
use nom::IResult;

/// whitespace that is not a newline character
fn nbsp_opt(input: &str) -> IResult<&str, &str> {
    take_while(|c| " \t".contains(c))(input)
}

/// newline characters
fn newline(input: &str) -> IResult<&str, &str> {
    preceded(nbsp_opt, take_while1(|c| "\n\r".contains(c)))(input)
}

fn equal_signs(input: &str) -> IResult<&str, &str> {
    preceded(nbsp_opt, take_while_m_n(2, 999, |c| c == '='))(input)
}

pub fn parse_knot_header(input: &str) -> IResult<&str, &str> {
    let (input, _) = terminated(&equal_signs, nbsp_opt)(input)?;

    let (input, title) = terminated(is_not(" \r\n\t=()/\\!@#$%^&*"), nbsp_opt)(input)?;

    let (input, _) = tuple((opt(equal_signs), newline))(input)?;

    Ok((input, title.trim_end()))
}

//pub fn parse_

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_knot_header() {
        assert_eq!(
            parse_knot_header("== train_station\n"),
            Ok(("", "train_station"))
        );

        assert_eq!(
            parse_knot_header("== train_station ==\n"),
            Ok(("", "train_station"))
        );

        assert_eq!(
            parse_knot_header(" === train_station === \n"),
            Ok(("", "train_station"))
        );

        assert_eq!(
            parse_knot_header("===train_station===\n"),
            Ok(("", "train_station"))
        );

        assert!(parse_knot_header(" = train_station = \n").is_err()); // not enough equals signs
        assert!(parse_knot_header("== train station ==\n").is_err()); // no spaces allowed
        assert!(parse_knot_header("train station\n").is_err()); // no equals at the start
    }
}

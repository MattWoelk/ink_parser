use nom::bytes::complete::{take_till, take_till1, take_while};
use nom::{bytes::complete::tag, IResult};

pub fn parse_knot_header(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("==")(input)?;
    let (input, _) = take_while(|c| c == '=')(input)?;
    let (input, _) = take_till1(|c| c != ' ' && c != '\t')(input)?;

    let (input, title) = take_till1(|c| c == '=' || c == '\n' || c == '\r')(input)?;

    dbg!(input);

    // TODO: parse "space_then_word" (a bunch of these) so we know when to stop before more equals

    let (input, _) = take_till(|c| c == '\r' || c == '\n')(input)?;

    dbg!(input);

    Ok((input, title.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_knot_header() {
        assert_eq!(
            parse_knot_header("== station"),
            Ok(("", "station".to_string()))
        );

        // TODO:
        //assert_eq!(
        //    parse_knot_header("== station =="),
        //    Ok(("", "station".to_string()))
        //);
    }
}

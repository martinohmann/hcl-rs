use super::{
    context::{StrContext, StrContextValue},
    error::ContextError,
    string::from_utf8_unchecked,
    Input,
};
use crate::Number;
use std::str::FromStr;
use winnow::{
    ascii::digit1,
    combinator::{alt, cut_err, opt, preceded, terminated},
    token::one_of,
    PResult, Parser,
};

pub(super) fn number<'a>(input: &mut Input<'a>) -> PResult<Number, ContextError<Input<'a>>> {
    alt((
        float.verify_map(Number::from_f64),
        integer.map(Number::from),
    ))
    .parse_next(input)
}

fn integer<'a>(input: &mut Input<'a>) -> PResult<u64, ContextError<Input<'a>>> {
    digit1
        .try_map(|s: &[u8]| {
            u64::from_str(unsafe { from_utf8_unchecked(s, "`digit1` filters out non-ascii") })
        })
        .parse_next(input)
}

fn float<'a>(input: &mut Input<'a>) -> PResult<f64, ContextError<Input<'a>>> {
    let fraction = preceded(b'.', digit1);

    terminated(digit1, alt((terminated(fraction, opt(exponent)), exponent)))
        .recognize()
        .try_map(|s: &[u8]| {
            f64::from_str(unsafe {
                from_utf8_unchecked(s, "`digit1` and `exponent` filter out non-ascii")
            })
        })
        .parse_next(input)
}

fn exponent<'a>(input: &mut Input<'a>) -> PResult<&'a [u8], ContextError<Input<'a>>> {
    (
        one_of(b"eE"),
        opt(one_of(b"+-")),
        cut_err(digit1).context(StrContext::Expected(StrContextValue::Description("digit"))),
    )
        .recognize()
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_integer() {
        let tests: &[(&str, u64)] = &[
            ("1", 1),
            ("99", 99),
            ("0", 0),
            ("18446744073709551615", u64::MAX),
        ];

        for (input, expected) in tests {
            let parsed = integer.parse(Input::new(input.as_bytes()));
            assert!(parsed.is_ok(), "expected `{input}` to parse correctly");
            assert_eq!(parsed.unwrap(), *expected);
        }
    }

    #[test]
    fn parse_float() {
        let tests: &[(&str, f64)] = &[
            ("1.0", 1.0),
            ("1e10", 10000000000.0),
            ("2.5E3", 2500.0),
            ("42e-3", 0.042),
            ("0.1E-4", 0.00001),
            ("1.7976931348623157e308", f64::MAX),
        ];

        for (input, expected) in tests {
            let parsed = float.parse(Input::new(input.as_bytes()));
            assert!(parsed.is_ok(), "expected `{input}` to parse correctly");
            assert_eq!(parsed.unwrap(), *expected);
        }
    }
}

use super::{
    context::{Context, Expected},
    from_utf8_unchecked, IResult, Input,
};
use crate::Number;
use std::str::FromStr;
use winnow::{
    branch::alt,
    bytes::one_of,
    character::digit1,
    combinator::{cut_err, opt},
    sequence::{preceded, terminated},
    Parser,
};

pub(super) fn number(input: Input) -> IResult<Input, Number> {
    alt((
        float.verify_map(Number::from_f64),
        integer.map(Number::from),
    ))(input)
}

fn integer(input: Input) -> IResult<Input, u64> {
    digit1
        .map_res(|s: &[u8]| {
            u64::from_str(unsafe { from_utf8_unchecked(s, "`digit1` filters out non-ascii") })
        })
        .parse_next(input)
}

fn float(input: Input) -> IResult<Input, f64> {
    let fraction = preceded(b'.', digit1);

    terminated(digit1, alt((terminated(fraction, opt(exponent)), exponent)))
        .recognize()
        .map_res(|s: &[u8]| {
            f64::from_str(unsafe {
                from_utf8_unchecked(s, "`digit1` and `exponent` filter out non-ascii")
            })
        })
        .parse_next(input)
}

fn exponent(input: Input) -> IResult<Input, &[u8]> {
    (
        one_of("eE"),
        opt(one_of("+-")),
        cut_err(digit1).context(Context::Expected(Expected::Description("digit"))),
    )
        .recognize()
        .parse_next(input)
}

use super::{IResult, Input};
use winnow::{
    branch::alt,
    bytes::{any, take_until0},
    character::{multispace0, not_line_ending, space0},
    combinator::{fail, peek},
    dispatch,
    multi::many0,
    sequence::{delimited, preceded},
    Parser,
};

pub(super) fn ws(input: Input) -> IResult<Input, ()> {
    (
        multispace0.void(),
        void(many0((comment, multispace0.void()))),
    )
        .void()
        .parse_next(input)
}

pub(super) fn sp(input: Input) -> IResult<Input, ()> {
    (space0.void(), void(many0((inline_comment, space0.void()))))
        .void()
        .parse_next(input)
}

fn comment(input: Input) -> IResult<Input, ()> {
    dispatch! {peek(any);
        b'#' => hash_line_comment,
        b'/' => alt((double_slash_line_comment, inline_comment)),
        _ => fail,
    }
    .parse_next(input)
}

pub(super) fn line_comment(input: Input) -> IResult<Input, ()> {
    dispatch! {peek(any);
        b'#' => hash_line_comment,
        b'/' => double_slash_line_comment,
        _ => fail,
    }
    .parse_next(input)
}

fn hash_line_comment(input: Input) -> IResult<Input, ()> {
    preceded(b'#', not_line_ending).void().parse_next(input)
}

fn double_slash_line_comment(input: Input) -> IResult<Input, ()> {
    preceded(b"//", not_line_ending).void().parse_next(input)
}

fn inline_comment(input: Input) -> IResult<Input, ()> {
    delimited(b"/*", take_until0("*/"), b"*/")
        .void()
        .parse_next(input)
}

#[inline]
pub(super) fn void<P, I, E>(inner: P) -> impl Parser<I, (), E>
where
    P: Parser<I, (), E>,
{
    inner
}

#[cfg(test)]
mod tests {
    use super::*;
    use winnow::FinishIResult;

    #[test]
    fn parse_comments() {
        let inline_comments = [
            "",
            " ",
            "/**/  ",
            "/*foo*/",
            " /* foo
                bar
                */  /* baz */\t",
        ];

        let multiline_comments = [
            "# foo
                # bar",
            "
            /* foo
                bar
                */  # baz */",
            "
                // foo #
                // bar /*
                # baz",
        ];

        for input in inline_comments {
            let parsed = sp.parse_next(Input::new(input.as_bytes())).finish();
            assert!(parsed.is_ok(), "expected `{input}` to parse correctly");
        }

        for input in multiline_comments {
            let parsed = sp.parse_next(Input::new(input.as_bytes())).finish();
            assert!(parsed.is_err(), "expected parse error for `{input}`");
        }

        for input in inline_comments.iter().chain(multiline_comments.iter()) {
            let parsed = ws.parse_next(Input::new(input.as_bytes())).finish();
            assert!(parsed.is_ok(), "expected `{input}` to parse correctly");
        }
    }
}

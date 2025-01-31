use super::prelude::*;

use winnow::ascii::{multispace0, space0, till_line_ending};
use winnow::combinator::{alt, delimited, fail, peek, preceded, repeat};
use winnow::token::{any, take_until};

pub(super) fn ws(input: &mut Input) -> ModalResult<()> {
    (
        multispace0.void(),
        void(repeat(0.., (comment, multispace0.void()))),
    )
        .void()
        .parse_next(input)
}

pub(super) fn sp(input: &mut Input) -> ModalResult<()> {
    (
        space0.void(),
        void(repeat(0.., (inline_comment, space0.void()))),
    )
        .void()
        .parse_next(input)
}

fn comment(input: &mut Input) -> ModalResult<()> {
    dispatch! {peek(any);
        '#' => hash_line_comment,
        '/' => alt((double_slash_line_comment, inline_comment)),
        _ => fail,
    }
    .parse_next(input)
}

pub(super) fn line_comment(input: &mut Input) -> ModalResult<()> {
    dispatch! {peek(any);
        '#' => hash_line_comment,
        '/' => double_slash_line_comment,
        _ => fail,
    }
    .parse_next(input)
}

fn hash_line_comment(input: &mut Input) -> ModalResult<()> {
    preceded('#', till_line_ending).void().parse_next(input)
}

fn double_slash_line_comment(input: &mut Input) -> ModalResult<()> {
    preceded("//", till_line_ending).void().parse_next(input)
}

fn inline_comment(input: &mut Input) -> ModalResult<()> {
    delimited("/*", take_until(0.., "*/"), "*/")
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
            let parsed = sp.parse(Input::new(input));
            assert!(parsed.is_ok(), "expected `{input}` to parse correctly");
        }

        for input in multiline_comments {
            let parsed = sp.parse(Input::new(input));
            assert!(parsed.is_err(), "expected parse error for `{input}`");
        }

        for input in inline_comments.iter().chain(multiline_comments.iter()) {
            let parsed = ws.parse(Input::new(input));
            assert!(parsed.is_ok(), "expected `{input}` to parse correctly");
        }
    }
}

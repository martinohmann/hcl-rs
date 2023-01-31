use super::ast::{Attribute, Block, Body, Structure};
use super::{char_or_cut, expr::expr, ident, line_comment, sp, string, ws, IResult};
use super::{decorated, prefix_decorated, spanned, suffix_decorated, Span, Spanned};
use crate::structure::BlockLabel;
use nom::{
    branch::alt,
    character::complete::{char, line_ending},
    combinator::{cut, eof, map, opt, value},
    multi::many0,
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
};

fn line_ending_terminated<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O>,
{
    terminated(
        inner,
        preceded(
            sp,
            alt((value((), line_ending), value((), eof), line_comment)),
        ),
    )
}

fn attribute(input: Span) -> IResult<Span, Attribute> {
    map(
        separated_pair(
            suffix_decorated(ident, sp),
            char('='),
            cut(prefix_decorated(sp, expr)),
        ),
        |(key, expr)| Attribute { key, expr },
    )(input)
}

fn block(input: Span) -> IResult<Span, Block> {
    map(
        tuple((
            suffix_decorated(ident, sp),
            many0(suffix_decorated(block_label, sp)),
            alt((
                // Multiline block.
                delimited(
                    line_ending_terminated(char_or_cut('{')),
                    body,
                    char_or_cut('}'),
                ),
                // One-line block.
                spanned(map(
                    delimited(
                        char_or_cut('{'),
                        opt(cut(decorated(sp, map(attribute, Structure::Attribute), sp))),
                        char_or_cut('}'),
                    ),
                    |attr| Body {
                        structures: attr.map(|attr| vec![attr]).unwrap_or_default(),
                    },
                )),
            )),
        )),
        |(identifier, labels, body)| Block {
            identifier,
            labels,
            body,
        },
    )(input)
}

fn block_label(input: Span) -> IResult<Span, BlockLabel> {
    alt((
        map(ident, BlockLabel::Identifier),
        map(string, BlockLabel::String),
    ))(input)
}

fn structure(input: Span) -> IResult<Span, Structure> {
    alt((
        map(attribute, Structure::Attribute),
        map(block, Structure::Block),
    ))(input)
}

pub fn body(input: Span) -> IResult<Span, Spanned<Body>> {
    suffix_decorated(
        map(
            many0(prefix_decorated(ws, line_ending_terminated(structure))),
            |structures| Body { structures },
        ),
        ws,
    )(input)
}

use super::Span;
use super::{
    char_or_cut, expr::expr, ident, line_comment, sp_delimited, sp_preceded, sp_terminated, string,
    ws_preceded, ws_terminated, IResult,
};
use crate::structure::{Attribute, Block, BlockLabel, Body, Structure};
use nom::{
    branch::alt,
    character::complete::{char, line_ending},
    combinator::{cut, eof, map, opt, value},
    multi::many0,
    sequence::{delimited, separated_pair, terminated, tuple},
};

fn line_ending_terminated<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O>,
{
    terminated(
        inner,
        sp_preceded(alt((value((), line_ending), value((), eof), line_comment))),
    )
}

fn attribute(input: Span) -> IResult<Span, Attribute> {
    map(
        separated_pair(ident, sp_delimited(char('=')), cut(expr)),
        |(key, expr)| Attribute { key, expr },
    )(input)
}

fn block(input: Span) -> IResult<Span, Block> {
    map(
        tuple((
            sp_terminated(ident),
            many0(sp_terminated(block_label)),
            alt((
                // Multiline block.
                delimited(
                    line_ending_terminated(char_or_cut('{')),
                    body,
                    char_or_cut('}'),
                ),
                // One-line block.
                map(
                    delimited(
                        char_or_cut('{'),
                        sp_delimited(opt(cut(attribute))),
                        char_or_cut('}'),
                    ),
                    |attr| attr.map(Body::from).unwrap_or_default(),
                ),
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

pub fn body(input: Span) -> IResult<Span, Body> {
    ws_terminated(map(
        many0(ws_preceded(line_ending_terminated(structure))),
        Body::from,
    ))(input)
}

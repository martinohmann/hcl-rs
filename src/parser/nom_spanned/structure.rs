use super::ast::{Attribute, Block, Body, Expression, Structure};
use super::{
    char_or_cut, decorated, expr::expr, ident, line_comment, prefix_decorated, sp, spanned, string,
    suffix_decorated, ws, IResult, Node, Span,
};
use crate::structure::BlockLabel;
use nom::{
    branch::alt,
    character::complete::{anychar, char, line_ending},
    combinator::{cut, eof, map, opt, peek, value},
    multi::many0,
    sequence::{delimited, pair, preceded, terminated},
};

fn line_trailing(input: Span) -> IResult<Span, ()> {
    preceded(
        sp,
        alt((value((), line_ending), value((), eof), line_comment)),
    )(input)
}

fn attribute_expr(input: Span) -> IResult<Span, Node<Expression>> {
    preceded(char('='), prefix_decorated(sp, cut(expr)))(input)
}

fn block_body(input: Span) -> IResult<Span, Node<Body>> {
    let single_attribute = map(
        pair(suffix_decorated(ident, sp), attribute_expr),
        |(key, expr)| Structure::Attribute(Attribute { key, expr }),
    );

    delimited(
        char_or_cut('{'),
        alt((
            // Multiline block.
            preceded(line_trailing, body),
            // One-line block.
            spanned(map(
                opt(decorated(sp, cut(single_attribute), sp)),
                |structure| Body {
                    structures: structure.map(|s| vec![s]).unwrap_or_default(),
                },
            )),
        )),
        char_or_cut('}'),
    )(input)
}

fn block_labels(input: Span) -> IResult<Span, Vec<Node<BlockLabel>>> {
    many0(suffix_decorated(block_label, sp))(input)
}

fn block_parts(input: Span) -> IResult<Span, (Vec<Node<BlockLabel>>, Node<Body>)> {
    pair(block_labels, block_body)(input)
}

fn block_label(input: Span) -> IResult<Span, BlockLabel> {
    alt((
        map(string, BlockLabel::String),
        map(ident, BlockLabel::Identifier),
    ))(input)
}

fn structure(input: Span) -> IResult<Span, Structure> {
    let (input, ident) = suffix_decorated(ident, sp)(input)?;
    let (input, ch) = peek(anychar)(input)?;

    if ch == '=' {
        let (input, expr) = attribute_expr(input)?;
        Ok((input, Structure::Attribute(Attribute { key: ident, expr })))
    } else {
        let (input, (labels, body)) = block_parts(input)?;
        Ok((
            input,
            Structure::Block(Block {
                identifier: ident,
                labels,
                body,
            }),
        ))
    }
}

pub fn body(input: Span) -> IResult<Span, Node<Body>> {
    suffix_decorated(
        map(
            many0(prefix_decorated(ws, terminated(structure, line_trailing))),
            |structures| Body { structures },
        ),
        ws,
    )(input)
}

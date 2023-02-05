use super::ast::{Attribute, Block, BlockBody, Body, Expression, Structure};
use super::{
    char_or_cut, decorated, expr::expr, ident, prefix_decorated, sp, span, spc, string,
    suffix_decorated, ws, Formatted, IResult, Input,
};
use crate::structure::BlockLabel;
use nom::{
    branch::alt,
    character::complete::{anychar, char, line_ending},
    combinator::{cut, eof, map, opt, peek, value},
    multi::many0,
    sequence::{delimited, pair, preceded, terminated},
};

fn line_trailing(input: Input) -> IResult<Input, ()> {
    value((), alt((line_ending, eof)))(input)
}

fn attribute_expr(input: Input) -> IResult<Input, Formatted<Expression>> {
    preceded(char('='), prefix_decorated(sp, cut(expr)))(input)
}

fn block_body(input: Input) -> IResult<Input, BlockBody> {
    let single_attribute = map(
        pair(suffix_decorated(ident, sp), attribute_expr),
        |(key, expr)| Attribute { key, expr },
    );

    delimited(
        char_or_cut('{'),
        alt((
            // Multiline block.
            map(
                pair(terminated(span(spc), line_trailing), body),
                |(prefix_span, mut body)| {
                    body.decor_mut().set_prefix(prefix_span);
                    BlockBody::Multiline(body)
                },
            ),
            // One-line block.
            map(
                decorated(sp, map(opt(cut(single_attribute)), Box::new), sp),
                BlockBody::Oneline,
            ),
        )),
        char_or_cut('}'),
    )(input)
}

fn block_labels(input: Input) -> IResult<Input, Vec<Formatted<BlockLabel>>> {
    many0(suffix_decorated(block_label, sp))(input)
}

fn block_parts(input: Input) -> IResult<Input, (Vec<Formatted<BlockLabel>>, BlockBody)> {
    pair(block_labels, block_body)(input)
}

fn block_label(input: Input) -> IResult<Input, BlockLabel> {
    alt((
        map(string, BlockLabel::String),
        map(ident, BlockLabel::Identifier),
    ))(input)
}

fn structure(input: Input) -> IResult<Input, Structure> {
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

pub fn body(input: Input) -> IResult<Input, Formatted<Body>> {
    suffix_decorated(
        map(
            many0(terminated(decorated(ws, structure, spc), line_trailing)),
            |structures| Body { structures },
        ),
        ws,
    )(input)
}

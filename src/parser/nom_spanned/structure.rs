use super::ast::{Attribute, Block, BlockBody, BlockLabel, Body, Expression, Structure};
use super::{
    char_or_cut, decorated, expr::expr, ident, prefix_decorated, sp, span, spc, string,
    suffix_decorated, with_decor, with_decor_suffix, with_span, ws, Formatted, IResult, Input,
};
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
        with_span(pair(suffix_decorated(ident, sp), attribute_expr)),
        |((key, expr), span)| {
            let mut attr = Attribute::new(key, expr);
            attr.set_span(span);
            attr
        },
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

fn block_labels(input: Input) -> IResult<Input, Vec<BlockLabel>> {
    many0(map(
        with_decor_suffix(with_span(block_label), sp),
        |((mut label, span), decor)| {
            label.set_span(span);
            label.decor_mut().replace(decor);
            label
        },
    ))(input)
}

fn block_parts(input: Input) -> IResult<Input, (Vec<BlockLabel>, BlockBody)> {
    pair(block_labels, block_body)(input)
}

fn block_label(input: Input) -> IResult<Input, BlockLabel> {
    alt((
        map(string, |string| BlockLabel::String(Formatted::new(string))),
        map(ident, |ident| BlockLabel::Identifier(Formatted::new(ident))),
    ))(input)
}

fn structure(input: Input) -> IResult<Input, Structure> {
    let (input, ident) = suffix_decorated(ident, sp)(input)?;
    let (input, ch) = peek(anychar)(input)?;

    if ch == '=' {
        let (input, expr) = attribute_expr(input)?;
        Ok((input, Structure::Attribute(Attribute::new(ident, expr))))
    } else {
        let (input, (labels, body)) = block_parts(input)?;
        Ok((
            input,
            Structure::Block(Block::new_with_labels(ident, labels, body)),
        ))
    }
}

pub fn body(input: Input) -> IResult<Input, Formatted<Body>> {
    suffix_decorated(
        map(
            many0(terminated(
                map(
                    with_decor(ws, with_span(structure), spc),
                    |((mut structure, span), decor)| {
                        structure.decor_mut().replace(decor);
                        structure.set_span(span);
                        structure
                    },
                ),
                line_trailing,
            )),
            |structures| Body { structures },
        ),
        ws,
    )(input)
}

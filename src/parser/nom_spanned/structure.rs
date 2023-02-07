use super::ast::{Attribute, Block, BlockBody, BlockLabel, Body, Expression, Structure};
use super::repr::{Decorate, Decorated};
use super::{
    char_or_cut, decor, expr::expr, ident, prefix_decor, sp, span, spanned, spc, string,
    suffix_decor, ws, IResult, Input,
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

fn attribute_expr(input: Input) -> IResult<Input, Expression> {
    preceded(char('='), prefix_decor(sp, cut(expr)))(input)
}

fn block_body(input: Input) -> IResult<Input, BlockBody> {
    let single_attribute = spanned(map(
        pair(suffix_decor(ident, sp), attribute_expr),
        |(key, expr)| Attribute::new(key, expr),
    ));

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
                decor(sp, map(opt(cut(single_attribute)), Box::new), sp),
                BlockBody::Oneline,
            ),
        )),
        char_or_cut('}'),
    )(input)
}

fn block_labels(input: Input) -> IResult<Input, Vec<BlockLabel>> {
    many0(suffix_decor(block_label, sp))(input)
}

fn block_parts(input: Input) -> IResult<Input, (Vec<BlockLabel>, BlockBody)> {
    pair(block_labels, block_body)(input)
}

fn block_label(input: Input) -> IResult<Input, BlockLabel> {
    alt((
        map(string, |string| BlockLabel::String(string.into())),
        map(ident, |ident| BlockLabel::Identifier(ident.into())),
    ))(input)
}

fn structure(input: Input) -> IResult<Input, Structure> {
    let (input, ident) = suffix_decor(ident, sp)(input)?;
    let (input, ch) = peek(anychar)(input)?;

    if ch == '=' {
        let (input, expr) = attribute_expr(input)?;
        Ok((
            input,
            Structure::Attribute(Attribute::new(ident, expr).into()),
        ))
    } else {
        let (input, (labels, body)) = block_parts(input)?;
        Ok((
            input,
            Structure::Block(Block::new_with_labels(ident, labels, body).into()),
        ))
    }
}

pub fn body(input: Input) -> IResult<Input, Decorated<Body>> {
    suffix_decor(
        map(
            many0(terminated(decor(ws, structure, spc), line_trailing)),
            |structures| Body { structures },
        ),
        ws,
    )(input)
}

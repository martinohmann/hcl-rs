use super::ast::{Attribute, Block, BlockBody, BlockLabel, Body, Expression, Structure};
use super::{
    cut_char, cut_context, decor,
    error::{Context, Expected},
    expr::expr,
    ident, prefix_decor, sp, span, spc, string, suffix_decor, ws, IResult, Input,
};
use nom::{
    branch::alt,
    character::complete::{anychar, char, line_ending},
    combinator::{eof, map, peek},
    multi::many0,
    sequence::{delimited, pair, preceded, terminated},
};

fn attribute_expr(input: Input) -> IResult<Input, Expression> {
    preceded(char('='), prefix_decor(sp, expr))(input)
}

fn block_body(input: Input) -> IResult<Input, BlockBody> {
    let attribute = map(
        pair(suffix_decor(ident, sp), attribute_expr),
        |(key, expr)| Attribute::new(key, expr),
    );

    delimited(
        cut_char('{'),
        alt((
            // Multiline block.
            map(
                prefix_decor(spc, preceded(line_ending, body)),
                BlockBody::Multiline,
            ),
            // One-line block.
            map(decor(sp, attribute, sp), BlockBody::Oneline),
            // Empty block.
            map(span(sp), |span| BlockBody::Empty(span.into())),
        )),
        cut_char('}'),
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
            Structure::Attribute(Box::new(Attribute::new(ident, expr))),
        ))
    } else {
        let (input, (labels, body)) = block_parts(input)?;
        Ok((
            input,
            Structure::Block(Box::new(Block::new_with_labels(ident, labels, body))),
        ))
    }
}

pub fn body(input: Input) -> IResult<Input, Body> {
    suffix_decor(
        many0(terminated(
            decor(ws, structure, spc),
            cut_context(
                alt((line_ending, eof)),
                Context::Expected(Expected::Description("newline or eof")),
            ),
        )),
        ws,
    )(input)
}

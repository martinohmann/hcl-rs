use super::{
    context::{cut_char, Context, Expected},
    expr::expr,
    ident::ident,
    repr::{decor, prefix_decor, suffix_decor},
    string::{raw, string},
    trivia::{line_comment, sp, ws},
    IResult, Input,
};
use crate::{
    expr::Expression,
    structure::{Attribute, Block, BlockBody, BlockLabel, Body, Structure},
};
use winnow::{
    branch::alt,
    bytes::any,
    character::line_ending,
    combinator::{cut_err, eof, opt, peek},
    multi::many0,
    prelude::*,
    sequence::{delimited, preceded, terminated},
};

fn attribute_expr(input: Input) -> IResult<Input, Expression> {
    preceded(b'=', prefix_decor(sp, expr))(input)
}

fn block_body(input: Input) -> IResult<Input, BlockBody> {
    let attribute =
        (suffix_decor(ident, sp), attribute_expr).map(|(key, expr)| Attribute::new(key, expr));

    delimited(
        cut_char('{'),
        alt((
            // Multiline block.
            prefix_decor(
                (sp, opt(line_comment)),
                preceded(line_ending, body.map(Box::new)),
            )
            .map(BlockBody::Multiline),
            // One-line block.
            decor(sp, attribute.map(Box::new), sp).map(BlockBody::Oneline),
            // Empty block.
            raw(sp).map(BlockBody::Empty),
        )),
        cut_char('}'),
    )(input)
}

fn block_labels(input: Input) -> IResult<Input, Vec<BlockLabel>> {
    many0(suffix_decor(block_label, sp))(input)
}

fn block_parts(input: Input) -> IResult<Input, (Vec<BlockLabel>, BlockBody)> {
    (block_labels, block_body).parse_next(input)
}

fn block_label(input: Input) -> IResult<Input, BlockLabel> {
    alt((
        string.map(|string| BlockLabel::String(string.into())),
        ident.map(BlockLabel::Identifier),
    ))(input)
}

fn structure(input: Input) -> IResult<Input, Structure> {
    let (input, ident) = suffix_decor(ident, sp).parse_next(input)?;
    let (input, ch) = peek(any)(input)?;

    if ch == b'=' {
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
            decor(ws, structure, (sp, opt(line_comment))),
            cut_err(alt((line_ending, eof)))
                .context(Context::Expected(Expected::Description("newline")))
                .context(Context::Expected(Expected::Description("eof"))),
        ))
        .map(Body::new),
        ws,
    )
    .parse_next(input)
}

use std::cell::RefCell;

use super::{
    context::{cut_char, Context, Expected},
    expr::expr,
    repr::{decorated, prefix_decorated, suffix_decorated},
    state::BodyParseState,
    string::{ident, raw_string, string},
    trivia::{line_comment, sp, void, ws},
    IResult, Input,
};
use crate::{
    expr::Expression,
    repr::{Decorate, SetSpan},
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
    stream::Location,
};

pub(super) fn body(input: Input) -> IResult<Input, Body> {
    let state = RefCell::new(BodyParseState::default());

    let (input, (span, suffix)) = (
        void(many0(terminated(
            (
                ws.span().map(|span| state.borrow_mut().on_ws(span)),
                structure(&state),
                (sp, opt(line_comment))
                    .span()
                    .map(|span| state.borrow_mut().on_ws(span)),
            ),
            cut_err(alt((line_ending, eof)).map(|_| state.borrow_mut().on_line_ending()))
                .context(Context::Expected(Expected::Description("newline")))
                .context(Context::Expected(Expected::Description("eof"))),
        )))
        .span(),
        raw_string(ws),
    )
        .parse_next(input)?;

    let mut body = state.into_inner().into_body();
    body.set_span(span);
    body.decor_mut().set_suffix(suffix);
    Ok((input, body))
}

fn structure<'i, 's>(
    state: &'s RefCell<BodyParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        let start = input.location();
        let (input, ident) = suffix_decorated(ident, sp).parse_next(input)?;
        let (input, ch) = peek(any)(input)?;

        let (input, mut structure) = if ch == b'=' {
            let (input, expr) = attribute_expr(input)?;
            let attr = Structure::Attribute(Attribute::new(ident, expr));
            (input, attr)
        } else {
            let (input, labels) = block_labels(input)?;
            let (input, body) = block_body(input)?;
            let block = Structure::Block(Block::new_with_labels(ident, labels, body));
            (input, block)
        };

        let end = input.location();
        structure.set_span(start..end);
        state.borrow_mut().on_structure(structure);
        Ok((input, ()))
    }
}

fn attribute_expr(input: Input) -> IResult<Input, Expression> {
    preceded(b'=', prefix_decorated(sp, expr))(input)
}

fn block_labels(input: Input) -> IResult<Input, Vec<BlockLabel>> {
    many0(suffix_decorated(block_label, sp))(input)
}

fn block_label(input: Input) -> IResult<Input, BlockLabel> {
    alt((
        string.map(|string| BlockLabel::String(string.into())),
        ident.map(BlockLabel::Identifier),
    ))(input)
}

fn block_body(input: Input) -> IResult<Input, BlockBody> {
    let attribute =
        (suffix_decorated(ident, sp), attribute_expr).map(|(key, expr)| Attribute::new(key, expr));

    delimited(
        cut_char('{'),
        alt((
            // Multiline block.
            prefix_decorated((sp, opt(line_comment)), preceded(line_ending, body))
                .map(BlockBody::Multiline),
            // One-line block.
            decorated(sp, attribute, sp).map(BlockBody::Oneline),
            // Empty block.
            raw_string(sp).map(BlockBody::Empty),
        )),
        cut_char('}'),
    )(input)
}

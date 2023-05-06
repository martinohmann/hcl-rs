use super::{
    context::{cut_char, cut_str_ident, Context, Expected},
    expr::expr,
    repr::{decorated, prefix_decorated, suffix_decorated},
    state::BodyParseState,
    string::{ident, is_id_start, raw_string, string},
    trivia::{line_comment, sp, void, ws},
    IResult, Input,
};
use crate::{
    expr::Expression,
    repr::{Decorate, Decorated, SetSpan},
    structure::{Attribute, Block, BlockLabel, Body, Structure},
};
use hcl_primitives::Ident;
use std::cell::RefCell;
use winnow::{
    ascii::line_ending,
    branch::alt,
    bytes::{any, one_of},
    combinator::{cut_err, eof, fail, opt, peek, repeat0},
    prelude::*,
    sequence::{delimited, preceded, terminated},
    stream::Location,
};

pub(super) fn body(input: Input) -> IResult<Input, Body> {
    let state = RefCell::new(BodyParseState::default());

    let (input, (span, suffix)) = (
        void(repeat0(terminated(
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
    state: &'s RefCell<BodyParseState<'i>>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        let start = input.location();
        let initial_input = input.clone();
        let (input, _) = peek(one_of(is_id_start)).parse_next(input)?;
        let (input, ident) = cut_str_ident.parse_next(input)?;
        let (input, suffix) = raw_string(sp).parse_next(input)?;
        let (input, ch) = peek(any).parse_next(input)?;

        let (input, mut structure) = match ch {
            b'=' => {
                if state.borrow_mut().is_redefined(ident) {
                    return cut_err(fail)
                        .context(Context::Expression("attribute"))
                        .context(Context::Expected(Expected::Description(
                            "unique attribute key; found redefined attribute",
                        )))
                        .parse_next(initial_input);
                }

                let (input, expr) = attribute_expr(input)?;
                let mut ident = Decorated::new(Ident::new_unchecked(ident));
                ident.decor_mut().set_suffix(suffix);
                let attr = Attribute::new(ident, expr);
                (input, Structure::Attribute(attr))
            }
            b'{' => {
                let (input, body) = block_body(input)?;
                let mut ident = Decorated::new(Ident::new_unchecked(ident));
                ident.decor_mut().set_suffix(suffix);
                let mut block = Block::new(ident);
                block.body = body;
                (input, Structure::Block(block))
            }
            ch if ch == b'"' || is_id_start(ch) => {
                let (input, labels) = block_labels(input)?;
                let (input, body) = block_body(input)?;
                let mut ident = Decorated::new(Ident::new_unchecked(ident));
                ident.decor_mut().set_suffix(suffix);
                let mut block = Block::new(ident);
                block.body = body;
                block.labels = labels;
                (input, Structure::Block(block))
            }
            _ => {
                return cut_err(fail)
                    .context(Context::Expression("structure"))
                    .context(Context::Expected(Expected::Char('{')))
                    .context(Context::Expected(Expected::Char('=')))
                    .context(Context::Expected(Expected::Char('"')))
                    .context(Context::Expected(Expected::Description("identifier")))
                    .parse_next(input)
            }
        };

        let end = input.location();
        structure.set_span(start..end);
        state.borrow_mut().on_structure(structure);
        Ok((input, ()))
    }
}

fn attribute_expr(input: Input) -> IResult<Input, Expression> {
    preceded(
        cut_char('=').context(Context::Expression("attribute")),
        prefix_decorated(sp, expr),
    )
    .parse_next(input)
}

fn block_labels(input: Input) -> IResult<Input, Vec<BlockLabel>> {
    repeat0(suffix_decorated(block_label, sp)).parse_next(input)
}

fn block_label(input: Input) -> IResult<Input, BlockLabel> {
    alt((
        string.map(|string| BlockLabel::String(Decorated::new(string))),
        ident.map(BlockLabel::Ident),
    ))
    .parse_next(input)
}

fn block_body(input: Input) -> IResult<Input, Body> {
    let attribute =
        (suffix_decorated(ident, sp), attribute_expr).map(|(key, expr)| Attribute::new(key, expr));

    delimited(
        cut_char('{'),
        alt((
            // Multiline block.
            prefix_decorated((sp, opt(line_comment)), preceded(line_ending, body)),
            // One-line block.
            (opt(decorated(sp, attribute, sp)), raw_string(sp)).map(|(attr, suffix)| {
                let mut body = Body::new();
                body.set_prefer_oneline(true);
                body.decor_mut().set_suffix(suffix);
                if let Some(attr) = attr {
                    body.push(attr);
                }
                body
            }),
        )),
        cut_char('}')
            .context(Context::Expression("block body"))
            .context(Context::Expected(Expected::Char('\n')))
            .context(Context::Expected(Expected::Description("identifier"))),
    )
    .parse_next(input)
}

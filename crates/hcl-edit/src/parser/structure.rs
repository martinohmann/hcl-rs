use super::prelude::*;

use super::expr::expr;
use super::repr::{decorated, prefix_decorated, suffix_decorated};
use super::state::BodyParseState;
use super::string::{cut_char, cut_str_ident, ident, raw_string, string};
use super::trivia::{line_comment, sp, void, ws};

use crate::expr::Expression;
use crate::structure::{Attribute, Block, BlockLabel, Body, Structure};
use crate::{Decorate, Decorated, Ident, SetSpan};

use hcl_primitives::ident::is_id_start;
use std::cell::RefCell;
use winnow::ascii::line_ending;
use winnow::combinator::{
    alt, cut_err, delimited, eof, fail, opt, peek, preceded, repeat, terminated,
};
use winnow::stream::Location;
use winnow::token::{any, one_of};

pub(super) fn body(input: &mut Input) -> ModalResult<Body> {
    let state = RefCell::new(BodyParseState::default());

    let (span, suffix) = (
        void(repeat(
            0..,
            terminated(
                (
                    ws.span().map(|span| state.borrow_mut().on_ws(span)),
                    structure(&state),
                    (sp, opt(line_comment))
                        .span()
                        .map(|span| state.borrow_mut().on_ws(span)),
                ),
                cut_err(alt((
                    line_ending.map(|_| state.borrow_mut().on_line_ending()),
                    eof.map(|_| state.borrow_mut().on_eof()),
                )))
                .context(StrContext::Expected(StrContextValue::Description(
                    "newline",
                )))
                .context(StrContext::Expected(StrContextValue::Description("eof"))),
            ),
        ))
        .span(),
        raw_string(ws),
    )
        .parse_next(input)?;

    let mut body = state.into_inner().into_body();
    body.set_span(span);
    body.decor_mut().set_suffix(suffix);
    Ok(body)
}

fn structure<'i, 's>(
    state: &'s RefCell<BodyParseState<'i>>,
) -> impl ModalParser<Input<'i>, (), ContextError> + 's {
    move |input: &mut Input<'i>| {
        let start = input.current_token_start();
        let checkpoint = input.checkpoint();
        peek(one_of(is_id_start)).parse_next(input)?;
        let (ident, ident_span) = cut_str_ident.with_span().parse_next(input)?;
        let suffix = raw_string(sp).parse_next(input)?;

        let mut structure = match peek(any).parse_next(input)? {
            '=' => {
                if state.borrow_mut().is_redefined(ident) {
                    input.reset(&checkpoint);
                    return cut_err(fail)
                        .context(StrContext::Label("attribute"))
                        .context(StrContext::Expected(StrContextValue::Description(
                            "unique attribute key; found redefined attribute",
                        )))
                        .parse_next(input);
                }

                let expr = attribute_expr(input)?;
                let mut ident = Decorated::new(Ident::new_unchecked(ident));
                ident.decor_mut().set_suffix(suffix);
                ident.set_span(ident_span);
                let attr = Attribute::new(ident, expr);
                Structure::Attribute(attr)
            }
            '{' => {
                let body = block_body(input)?;
                let mut ident = Decorated::new(Ident::new_unchecked(ident));
                ident.decor_mut().set_suffix(suffix);
                ident.set_span(ident_span);
                let mut block = Block::new(ident);
                block.body = body;
                Structure::Block(block)
            }
            ch if ch == '"' || is_id_start(ch) => {
                let labels = block_labels(input)?;
                let body = block_body(input)?;
                let mut ident = Decorated::new(Ident::new_unchecked(ident));
                ident.decor_mut().set_suffix(suffix);
                ident.set_span(ident_span);
                let mut block = Block::new(ident);
                block.body = body;
                block.labels = labels;
                Structure::Block(block)
            }
            _ => {
                return cut_err(fail)
                    .context(StrContext::Label("structure"))
                    .context(StrContext::Expected(StrContextValue::CharLiteral('{')))
                    .context(StrContext::Expected(StrContextValue::CharLiteral('=')))
                    .context(StrContext::Expected(StrContextValue::CharLiteral('"')))
                    .context(StrContext::Expected(StrContextValue::Description(
                        "identifier",
                    )))
                    .parse_next(input)
            }
        };

        let end = input.previous_token_end();
        structure.set_span(start..end);
        state.borrow_mut().on_structure(structure);
        Ok(())
    }
}

fn attribute_expr(input: &mut Input) -> ModalResult<Expression> {
    preceded(
        cut_char('=').context(StrContext::Label("attribute")),
        prefix_decorated(sp, expr),
    )
    .parse_next(input)
}

fn block_labels(input: &mut Input) -> ModalResult<Vec<BlockLabel>> {
    repeat(0.., suffix_decorated(block_label, sp)).parse_next(input)
}

fn block_label(input: &mut Input) -> ModalResult<BlockLabel> {
    alt((
        string.map(|string| BlockLabel::String(Decorated::new(string))),
        ident.map(BlockLabel::Ident),
    ))
    .parse_next(input)
}

fn block_body(input: &mut Input) -> ModalResult<Body> {
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
            .context(StrContext::Label("block body"))
            .context(StrContext::Expected(StrContextValue::CharLiteral('\n')))
            .context(StrContext::Expected(StrContextValue::Description(
                "identifier",
            ))),
    )
    .parse_next(input)
}

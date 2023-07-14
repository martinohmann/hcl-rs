use super::prelude::*;

use super::expr::expr;
use super::repr::{decorated, prefix_decorated, suffix_decorated};
use super::state::BodyParseState;
use super::string::{cut_char, cut_str_ident, ident, is_id_start, raw_string, string};
use super::trivia::{line_comment, sp, void, ws};

use crate::expr::Expression;
use crate::structure::{Attribute, Block, BlockLabel, Body, Structure};
use crate::{Decorate, Decorated, Ident, SetSpan};

use std::cell::RefCell;
use winnow::ascii::line_ending;
use winnow::combinator::{
    alt, cut_err, delimited, eof, fail, opt, peek, preceded, repeat, terminated,
};
use winnow::stream::Location;
use winnow::token::{any, one_of};

pub(super) fn body(input: &mut Input) -> PResult<Body> {
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
                cut_err(alt((line_ending, eof)).map(|_| state.borrow_mut().on_line_ending()))
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
) -> impl Parser<Input<'i>, (), ContextError> + 's {
    move |input: &mut Input<'i>| {
        let start = input.location();
        let checkpoint = input.checkpoint();
        peek(one_of(is_id_start)).parse_next(input)?;
        let ident = cut_str_ident.parse_next(input)?;
        let suffix = raw_string(sp).parse_next(input)?;

        let mut structure = match peek(any).parse_next(input)? {
            b'=' => {
                if state.borrow_mut().is_redefined(ident) {
                    input.reset(checkpoint);
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
                let attr = Attribute::new(ident, expr);
                Structure::Attribute(attr)
            }
            b'{' => {
                let body = block_body(input)?;
                let mut ident = Decorated::new(Ident::new_unchecked(ident));
                ident.decor_mut().set_suffix(suffix);
                let mut block = Block::new(ident);
                block.body = body;
                Structure::Block(block)
            }
            ch if ch == b'"' || is_id_start(ch) => {
                let labels = block_labels(input)?;
                let body = block_body(input)?;
                let mut ident = Decorated::new(Ident::new_unchecked(ident));
                ident.decor_mut().set_suffix(suffix);
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

        let end = input.location();
        structure.set_span(start..end);
        state.borrow_mut().on_structure(structure);
        Ok(())
    }
}

fn attribute_expr(input: &mut Input) -> PResult<Expression> {
    preceded(
        cut_char('=').context(StrContext::Label("attribute")),
        prefix_decorated(sp, expr),
    )
    .parse_next(input)
}

fn block_labels(input: &mut Input) -> PResult<Vec<BlockLabel>> {
    repeat(0.., suffix_decorated(block_label, sp)).parse_next(input)
}

fn block_label(input: &mut Input) -> PResult<BlockLabel> {
    alt((
        string.map(|string| BlockLabel::String(Decorated::new(string))),
        ident.map(BlockLabel::Ident),
    ))
    .parse_next(input)
}

fn block_body(input: &mut Input) -> PResult<Body> {
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

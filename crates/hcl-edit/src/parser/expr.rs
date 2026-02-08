use super::prelude::*;

use super::number::number as num;
use super::repr::{decorated, prefix_decorated, spanned, suffix_decorated};
use super::state::ExprParseState;
use super::string::{cut_char, cut_ident, cut_tag, ident, raw_string, str_ident, string};
use super::template::{heredoc_template, string_template};
use super::trivia::{line_comment, sp, ws};

use crate::expr::{
    Array, BinaryOperator, Expression, ForCond, ForExpr, ForIntro, FuncArgs, FuncCall, FuncName,
    Object, ObjectKey, ObjectValue, ObjectValueAssignment, ObjectValueTerminator, Parenthesis,
    Splat, TraversalOperator, UnaryOp, UnaryOperator,
};
use crate::template::HeredocTemplate;
use crate::{Decorate, Decorated, Formatted, Ident, RawString, SetSpan, Spanned};

use hcl_primitives::ident::is_id_start;
use std::cell::RefCell;
use winnow::ascii::{crlf, dec_uint, line_ending, newline, space0};
use winnow::combinator::{
    alt, cut_err, delimited, empty, fail, not, opt, peek, preceded, repeat, separated,
    separated_pair, terminated,
};
use winnow::token::{any, none_of, one_of, take};

fn ws_or_sp<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        if state.borrow().newlines_allowed() {
            ws.parse_next(input)
        } else {
            sp.parse_next(input)
        }
    }
}

pub(super) fn expr(input: &mut Input) -> ModalResult<Expression> {
    parse_expr(RefCell::default(), input)
}

pub(super) fn multiline_expr(input: &mut Input) -> ModalResult<Expression> {
    let mut state = ExprParseState::default();
    state.allow_newlines();
    parse_expr(RefCell::new(state), input)
}

fn expr_with_state<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, Expression, ContextError> + '_ {
    move |input: &mut Input<'i>| parse_expr(state.clone(), input)
}

fn expr_term_with_state<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, Expression, ContextError> + '_ {
    move |input: &mut Input<'i>| parse_expr_term(state.clone(), input)
}

#[inline]
fn parse_expr(state: RefCell<ExprParseState>, input: &mut Input) -> ModalResult<Expression> {
    let span = expr_inner(&state).span().parse_next(input)?;
    let mut expr = state.into_inner().into_expr();
    expr.set_span(span);
    Ok(expr)
}

#[inline]
fn parse_expr_term(state: RefCell<ExprParseState>, input: &mut Input) -> ModalResult<Expression> {
    expr_term_inner(&state).parse_next(input)?;
    Ok(state.into_inner().into_expr())
}

fn expr_inner<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        expr_term_inner(state).parse_next(input)?;

        loop {
            let checkpoint = input.checkpoint();
            // Parse the next whitespace sequence and only add it as decor suffix to the expression
            // if we encounter conditional. We'll rewind the parser if none follows.
            let suffix = ws_or_sp(state).span().parse_next(input)?;

            // Peek the next two bytes to identify the following operation, if any.
            match peek(take::<_, _, ContextError>(2usize)).parse_next(input) {
                // Conditional.
                Ok(b) if b.starts_with('?') => {
                    state.borrow_mut().on_ws(suffix);
                    return conditional(state).parse_next(input);
                }
                // Binary operations.
                //
                // Note: `//` and `/*` are comment starts. Do not mistakenly parse a `/` as binary
                // division operator.
                //
                // Also, matching a single `=` is ambiguous as it could also be an object
                // key-value separator, so we'll need to match on `==`.
                //
                // After the binary operations are consumed, the loop is entered again to consume
                // a potentially following conditional.
                Ok(b)
                    if !(b == "//" || b == "/*")
                        && (b == "=="
                            || b.starts_with([
                                '!', '<', '>', '+', '-', '*', '/', '%', '&', '|',
                            ])) =>
                {
                    input.reset(&checkpoint);
                    binary_ops(state).parse_next(input)?;
                }
                // None of the above matched or we hit the end of input.
                _ => {
                    input.reset(&checkpoint);
                    return Ok(());
                }
            }
        }
    }
}

fn expr_term_inner<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        dispatch! {peek(any);
            '"' => stringlike(state),
            '[' => array(state),
            '{' => object(state),
            '0'..='9' => number(state),
            '<' => heredoc(state),
            '-' => alt((neg_number(state), unary_op(state))),
            '!' => unary_op(state),
            '(' => parenthesis(state),
            b if is_id_start(b) => identlike(state),
            _ => cut_err(fail)
                .context(StrContext::Label("expression"))
                .context(StrContext::Expected(StrContextValue::CharLiteral('"')))
                .context(StrContext::Expected(StrContextValue::CharLiteral('[')))
                .context(StrContext::Expected(StrContextValue::CharLiteral('{')))
                .context(StrContext::Expected(StrContextValue::CharLiteral('-')))
                .context(StrContext::Expected(StrContextValue::CharLiteral('!')))
                .context(StrContext::Expected(StrContextValue::CharLiteral('(')))
                .context(StrContext::Expected(StrContextValue::CharLiteral('_')))
                .context(StrContext::Expected(StrContextValue::CharLiteral('<')))
                .context(StrContext::Expected(StrContextValue::Description("letter")))
                .context(StrContext::Expected(StrContextValue::Description("digit"))),
        }
        .span()
        .parse_next(input)
        .map(|span| state.borrow_mut().on_span(span))?;

        let checkpoint = input.checkpoint();
        // Parse the next whitespace sequence and only add it as decor suffix to the expression if
        // we encounter a traversal. We'll rewind the parser if none follows.
        let suffix = ws_or_sp(state).span().parse_next(input)?;

        // Peek the next two bytes to identify the following operation, if any.
        match peek(take::<_, _, ContextError>(2usize)).parse_next(input) {
            // Traversal operator.
            //
            // The sequence `..` might introduce a `...` operator within a for object expr
            // or after the last argument of a function call, do not mistakenly parse it as
            // a `.` traversal operator.
            Ok(b) if b != ".." && b.starts_with(['.', '[']) => {
                state.borrow_mut().on_ws(suffix);
                traversal(state).parse_next(input)
            }
            // None of the above matched or we hit the end of input.
            _ => {
                input.reset(&checkpoint);
                Ok(())
            }
        }
    }
}

fn stringlike<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        alt((
            string.map(|string| {
                state
                    .borrow_mut()
                    .on_expr_term(Expression::String(Decorated::new(string)));
            }),
            string_template.map(|template| {
                state
                    .borrow_mut()
                    .on_expr_term(Expression::StringTemplate(template));
            }),
        ))
        .parse_next(input)
    }
}

fn number<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        num.with_taken()
            .map(|(num, repr)| {
                let mut num = Formatted::new(num);
                num.set_repr(repr);
                state.borrow_mut().on_expr_term(Expression::Number(num));
            })
            .parse_next(input)
    }
}

fn neg_number<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        preceded(('-', sp), num)
            .with_taken()
            .map(|(num, repr)| {
                let mut num = Formatted::new(-num);
                num.set_repr(repr);
                state.borrow_mut().on_expr_term(Expression::Number(num));
            })
            .parse_next(input)
    }
}

fn traversal<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        repeat(
            1..,
            prefix_decorated(
                ws_or_sp(state),
                // A `..` may indicate a for object expr containing a `...`, ensure there isn't a
                // subsequent `.` traversal operator and backtrack if there is.
                preceded(not(".."), traversal_operator.map(Decorated::new)),
            ),
        )
        .map(|operators| state.borrow_mut().on_traversal(operators))
        .parse_next(input)
    }
}

fn traversal_operator(input: &mut Input) -> ModalResult<TraversalOperator> {
    dispatch! {any;
        '.' => prefix_decorated(
            ws,
            dispatch! {peek(any);
                '*' => '*'.value(TraversalOperator::AttrSplat(Decorated::new(Splat))),
                '0'..='9' => dec_uint.map(|index: u64| TraversalOperator::LegacyIndex(index.into())),
                b if is_id_start(b) => ident.map(TraversalOperator::GetAttr),
                _ => cut_err(fail)
                    .context(StrContext::Label("traversal operator"))
                    .context(StrContext::Expected(StrContextValue::CharLiteral('*')))
                    .context(StrContext::Expected(StrContextValue::Description("identifier")))
                    .context(StrContext::Expected(StrContextValue::Description("unsigned integer"))),
            },
        ),
        '[' => terminated(
            decorated(
                ws,
                dispatch! {peek(any);
                    '*' => '*'.value(TraversalOperator::FullSplat(Decorated::new(Splat))),
                    _ => multiline_expr.map(TraversalOperator::Index),
                },
                ws,
            ),
            cut_char(']'),
        ),
        _ => fail,
    }
    .parse_next(input)
}

fn unary_op<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        (
            spanned(unary_operator.map(Spanned::new)),
            prefix_decorated(sp, expr_term_with_state(state)),
        )
            .map(|(operator, expr)| {
                state
                    .borrow_mut()
                    .on_expr_term(UnaryOp::new(operator, expr));
            })
            .parse_next(input)
    }
}

fn unary_operator(input: &mut Input) -> ModalResult<UnaryOperator> {
    dispatch! {any;
        '-' => empty.value(UnaryOperator::Neg),
        '!' => empty.value(UnaryOperator::Not),
        _ => fail,
    }
    .parse_next(input)
}

fn binary_ops<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        repeat(
            1..,
            (
                decorated(
                    ws_or_sp(state),
                    binary_operator.map(Decorated::new),
                    ws_or_sp(state),
                ),
                expr_term_with_state(state),
            ),
        )
        .map(|ops| state.borrow_mut().on_binary_ops(ops))
        .parse_next(input)
    }
}

fn binary_operator(input: &mut Input) -> ModalResult<BinaryOperator> {
    dispatch! {any;
        '=' => '='.value(BinaryOperator::Eq),
        '!' => '='.value(BinaryOperator::NotEq),
        '<' => alt((
            '='.value(BinaryOperator::LessEq),
            empty.value(BinaryOperator::Less),
        )),
        '>' => alt((
            '='.value(BinaryOperator::GreaterEq),
            empty.value(BinaryOperator::Greater),
        )),
        '+' => empty.value(BinaryOperator::Plus),
        '-' => empty.value(BinaryOperator::Minus),
        '*' => empty.value(BinaryOperator::Mul),
        '/' => not(one_of(b"/*")).value(BinaryOperator::Div),
        '%' => empty.value(BinaryOperator::Mod),
        '&' => '&'.value(BinaryOperator::And),
        '|' => '|'.value(BinaryOperator::Or),
        _ => fail,
    }
    .parse_next(input)
}

fn conditional<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        (
            preceded(
                '?',
                decorated(ws_or_sp(state), expr_with_state(state), ws_or_sp(state)),
            ),
            preceded(
                cut_char(':'),
                prefix_decorated(ws_or_sp(state), expr_with_state(state)),
            ),
        )
            .map(|(true_expr, false_expr)| state.borrow_mut().on_conditional(true_expr, false_expr))
            .parse_next(input)
    }
}

fn array<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        state.borrow_mut().allow_newlines();
        delimited(
            '[',
            for_expr_or_items(for_list_expr(state), array_items(state)),
            cut_char(']'),
        )
        .parse_next(input)
    }
}

fn for_list_expr<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        (
            for_intro,
            decorated(ws, expr_with_state(state), ws),
            opt(for_cond(state)),
        )
            .map(|(intro, value_expr, cond)| {
                let mut expr = ForExpr::new(intro, value_expr);
                expr.cond = cond;

                state
                    .borrow_mut()
                    .on_expr_term(Expression::ForExpr(Box::new(expr)));
            })
            .parse_next(input)
    }
}

fn array_items<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        let values = separated(
            0..,
            decorated(ws, preceded(not(']'), expr_with_state(state)), ws),
            ',',
        );

        (values, opt(','), raw_string(ws))
            .map(|(values, comma, trailing)| {
                let values: Vec<_> = values;
                let mut array = Array::from(values);
                if comma.is_some() {
                    array.set_trailing_comma(true);
                }
                array.set_trailing(trailing);
                state.borrow_mut().on_expr_term(Expression::Array(array));
            })
            .parse_next(input)
    }
}

fn object<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        delimited(
            '{',
            for_expr_or_items(for_object_expr(state), object_items(state)),
            cut_char('}'),
        )
        .parse_next(input)
    }
}

fn for_object_expr<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        state.borrow_mut().allow_newlines();
        (
            for_intro,
            separated_pair(
                decorated(ws, expr, ws),
                cut_tag("=>"),
                decorated(ws, expr, ws),
            ),
            opt(("...", raw_string(ws))),
            opt(for_cond(state)),
        )
            .map(|(intro, (key_expr, value_expr), grouping, cond)| {
                let mut expr = ForExpr::new(intro, value_expr);
                expr.key_expr = Some(key_expr);
                expr.cond = cond;
                if let Some((_, trailing)) = grouping {
                    expr.grouping = true;
                    if let Some(ref mut cond) = expr.cond {
                        cond.decor_mut().set_prefix(trailing);
                    } else {
                        expr.decor_mut().set_suffix(trailing);
                    }
                }

                state
                    .borrow_mut()
                    .on_expr_term(Expression::ForExpr(Box::new(expr)));
            })
            .parse_next(input)
    }
}

fn object_items<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        let mut object = Object::new();

        loop {
            let trailing = raw_string(ws).parse_next(input)?;
            let ch = peek(any).parse_next(input)?;

            if ch == '}' {
                object.set_trailing(trailing);
                state.borrow_mut().on_expr_term(Expression::Object(object));
                return Ok(());
            }

            let mut key = object_key(input)?;
            let mut value = object_value(input)?;
            key.decor_mut().set_prefix(trailing);

            // Look for the closing brace and return or consume the object item separator and proceed
            // with the next object item, if any.
            let ch = peek(any).parse_next(input)?;

            let value_terminator = match ch {
                '}' => {
                    value.set_terminator(ObjectValueTerminator::None);
                    object.insert(key, value);
                    state.borrow_mut().on_expr_term(Expression::Object(object));
                    return Ok(());
                }
                '\r' => crlf
                    .value(ObjectValueTerminator::Newline)
                    .parse_next(input)?,
                '\n' => newline
                    .value(ObjectValueTerminator::Newline)
                    .parse_next(input)?,
                ',' => take(1usize)
                    .value(ObjectValueTerminator::Comma)
                    .parse_next(input)?,
                '#' | '/' => {
                    let comment_span = line_comment.span().parse_next(input)?;

                    let decor = value.expr_mut().decor_mut();

                    // Associate the trailing comment with the item value, updating the span if it
                    // already has a decor suffix.
                    let suffix_span = match decor.suffix().and_then(RawString::span) {
                        Some(span) => span.start..comment_span.end,
                        None => comment_span,
                    };

                    decor.set_suffix(RawString::from_span(suffix_span));

                    line_ending
                        .value(ObjectValueTerminator::Newline)
                        .parse_next(input)?
                }
                _ => {
                    return cut_err(fail)
                        .context(StrContext::Label("object item"))
                        .context(StrContext::Expected(StrContextValue::CharLiteral('}')))
                        .context(StrContext::Expected(StrContextValue::CharLiteral(',')))
                        .context(StrContext::Expected(StrContextValue::CharLiteral('\n')))
                        .parse_next(input)
                }
            };

            value.set_terminator(value_terminator);
            object.insert(key, value);
        }
    }
}

fn object_key(input: &mut Input) -> ModalResult<ObjectKey> {
    suffix_decorated(
        expr.map(|expr| {
            // Variable identifiers without traversal are treated as identifier object keys.
            //
            // Handle this case here by converting the variable into an identifier. This
            // avoids re-parsing the whole key-value pair when an identifier followed by a
            // traversal operator is encountered.
            if let Expression::Variable(variable) = expr {
                ObjectKey::Ident(Decorated::new(variable.into_value()))
            } else {
                ObjectKey::Expression(expr)
            }
        }),
        sp,
    )
    .parse_next(input)
}

fn object_value(input: &mut Input) -> ModalResult<ObjectValue> {
    (object_value_assignment, decorated(sp, expr, sp))
        .map(|(assignment, expr)| {
            let mut value = ObjectValue::new(expr);
            value.set_assignment(assignment);
            value
        })
        .parse_next(input)
}

fn object_value_assignment(input: &mut Input) -> ModalResult<ObjectValueAssignment> {
    dispatch! {any;
        '=' => empty.value(ObjectValueAssignment::Equals),
        ':' => empty.value(ObjectValueAssignment::Colon),
        _ => cut_err(fail)
            .context(StrContext::Label("object value assignment"))
            .context(StrContext::Expected(StrContextValue::CharLiteral('=')))
            .context(StrContext::Expected(StrContextValue::CharLiteral(':'))),
    }
    .parse_next(input)
}

fn for_expr_or_items<'i, F, I>(
    mut for_expr_parser: F,
    mut items_parser: I,
) -> impl ModalParser<Input<'i>, (), ContextError>
where
    F: ModalParser<Input<'i>, (), ContextError>,
    I: ModalParser<Input<'i>, (), ContextError>,
{
    move |input: &mut Input<'i>| {
        // The `for` tag needs to be followed by either a space character or a comment start to
        // disambiguate. Otherwise an identifier like `format` will match both the `for` tag
        // and the following identifier which would fail parsing of arrays with identifier/func
        // call elements and objects with those as keys.
        match peek((ws, "for", one_of(b" \t#/\n"))).parse_next(input) {
            Ok(_) => for_expr_parser.parse_next(input),
            Err(_) => items_parser.parse_next(input),
        }
    }
}

fn for_intro(input: &mut Input) -> ModalResult<ForIntro> {
    prefix_decorated(
        ws,
        delimited(
            "for",
            (
                decorated(ws, cut_ident, ws),
                opt(preceded(',', decorated(ws, cut_ident, ws))),
                preceded(cut_tag("in"), decorated(ws, expr, ws)),
            ),
            cut_char(':'),
        )
        .map(|(first, second, expr)| match second {
            Some(second) => {
                let mut intro = ForIntro::new(second, expr);
                intro.key_var = Some(first);
                intro
            }
            None => ForIntro::new(first, expr),
        }),
    )
    .parse_next(input)
}

fn for_cond<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, ForCond, ContextError> + '_ {
    move |input: &mut Input| {
        prefix_decorated(
            ws,
            preceded("if", decorated(ws, expr_with_state(state), ws)).map(ForCond::new),
        )
        .parse_next(input)
    }
}

fn parenthesis<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        delimited(
            cut_char('('),
            decorated(ws, multiline_expr, ws)
                .map(|expr| state.borrow_mut().on_expr_term(Parenthesis::new(expr))),
            cut_char(')'),
        )
        .parse_next(input)
    }
}

fn heredoc<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        let (indented, delim) = heredoc_start(input)?;

        let (template, trailing) = (
            spanned(heredoc_template(delim)),
            terminated(
                raw_string(space0),
                cut_err(delim).context(StrContext::Expected(StrContextValue::Description(
                    "heredoc end delimiter",
                ))),
            ),
        )
            .parse_next(input)?;

        let mut heredoc = HeredocTemplate::new(Ident::new_unchecked(delim), template);

        if indented {
            heredoc.dedent();
        }

        heredoc.set_trailing(trailing);

        state
            .borrow_mut()
            .on_expr_term(Expression::HeredocTemplate(Box::new(heredoc)));
        Ok(())
    }
}

fn heredoc_start<'a>(input: &mut Input<'a>) -> ModalResult<(bool, &'a str)> {
    terminated(
        (
            preceded("<<", opt('-')).map(|indent| indent.is_some()),
            cut_err(str_ident).context(StrContext::Expected(StrContextValue::Description(
                "identifier",
            ))),
        ),
        cut_err(line_ending).context(StrContext::Expected(StrContextValue::CharLiteral('\n'))),
    )
    .parse_next(input)
}

fn identlike<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, (), ContextError> + '_ {
    move |input: &mut Input<'i>| {
        let (ident, span) = str_ident.with_span().parse_next(input)?;

        let checkpoint = input.checkpoint();

        // Parse the next whitespace sequence and only add it as decor suffix to the identifier if
        // we actually encounter a function call.
        let suffix = ws_or_sp(state).span().parse_next(input)?;

        let expr = match peek(take::<_, _, ContextError>(2usize)).parse_next(input) {
            Ok(peeked) if peeked == "::" || peeked.starts_with('(') => {
                // This is a function call: parsed identifier starts a function namespace, or function
                // arguments follow.
                let mut ident = Decorated::new(Ident::new_unchecked(ident));
                ident.decor_mut().set_suffix(RawString::from_span(suffix));
                ident.set_span(span);

                let func_name = if peeked == "::" {
                    // Consume the remaining namespace components and function name.
                    let mut namespace = func_namespace_components(state).parse_next(input)?;

                    // We already parsed the first namespace element before and the function name is
                    // now part of the remaining namspace components, so we have to correct this.
                    let name = namespace.pop().unwrap();
                    namespace.insert(0, ident);

                    FuncName { namespace, name }
                } else {
                    FuncName::from(ident)
                };

                let func_args = func_args(state).parse_next(input)?;
                let func_call = FuncCall::new(func_name, func_args);
                Expression::FuncCall(Box::new(func_call))
            }
            _ => {
                // This is not a function call: identifier is either keyword or variable name.
                input.reset(&checkpoint);

                match ident {
                    "null" => Expression::null(),
                    "true" => Expression::from(true),
                    "false" => Expression::from(false),
                    var => Expression::from(Ident::new_unchecked(var)),
                }
            }
        };

        state.borrow_mut().on_expr_term(expr);
        Ok(())
    }
}

fn func_namespace_components<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, Vec<Decorated<Ident>>, ContextError> + '_ {
    move |input: &mut Input<'i>| {
        repeat(
            1..,
            preceded(
                "::",
                decorated(
                    ws_or_sp(state),
                    cut_err(ident).context(StrContext::Expected(StrContextValue::Description(
                        "identifier",
                    ))),
                    ws_or_sp(state),
                ),
            ),
        )
        .parse_next(input)
    }
}

fn func_args<'i>(
    state: &RefCell<ExprParseState>,
) -> impl ModalParser<Input<'i>, FuncArgs, ContextError> + '_ {
    move |input: &mut Input| {
        #[derive(Copy, Clone)]
        enum Trailer {
            Comma,
            Ellipsis,
        }

        state.borrow_mut().allow_newlines();

        let args = separated(
            1..,
            decorated(
                ws,
                preceded(peek(none_of(b",.)")), expr_with_state(state)),
                ws,
            ),
            ',',
        );

        let trailer = dispatch! {any;
            ',' => empty.value(Trailer::Comma),
            '.' => cut_tag("..").value(Trailer::Ellipsis),
            _ => fail,
        };

        delimited(
            cut_char('('),
            (opt((args, opt(trailer))), raw_string(ws)).map(|(args, trailing)| {
                let mut args = match args {
                    Some((args, Some(trailer))) => {
                        let args: Vec<_> = args;
                        let mut args = FuncArgs::from(args);
                        if let Trailer::Ellipsis = trailer {
                            args.set_expand_final(true);
                        } else {
                            args.set_trailing_comma(true);
                        }
                        args
                    }
                    Some((args, None)) => FuncArgs::from(args),
                    None => FuncArgs::default(),
                };

                args.set_trailing(trailing);
                args
            }),
            cut_char(')').context(StrContext::Expected(StrContextValue::Description(
                "expression",
            ))),
        )
        .parse_next(input)
    }
}

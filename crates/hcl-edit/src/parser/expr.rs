use super::{
    context::{cut_char, cut_ident, cut_tag, Context, Expected},
    error::ParseError,
    number::number as num,
    repr::{decorated, prefix_decorated, spanned, suffix_decorated},
    state::ExprParseState,
    string::{from_utf8_unchecked, ident, is_id_start, raw_string, str_ident, string},
    template::{heredoc_template, string_template},
    trivia::{line_comment, sp, ws},
    IResult, Input,
};
use crate::{
    expr::*,
    repr::{Decorate, Decorated, Formatted, SetSpan, Spanned},
    template::HeredocTemplate,
    Ident, RawString,
};
use std::cell::RefCell;
use winnow::{
    branch::alt,
    bytes::{any, none_of, one_of, take},
    character::{crlf, dec_uint, line_ending, newline, space0},
    combinator::{cut_err, fail, not, opt, peek, success},
    dispatch,
    multi::{many1, separated0, separated1},
    sequence::{delimited, preceded, separated_pair, terminated},
    stream::Location,
    Parser,
};

pub(super) fn expr(input: Input) -> IResult<Input, Expression> {
    let state = RefCell::new(ExprParseState::default());
    let (input, _) = expr_inner(&state).parse_next(input)?;
    let expr = state.into_inner().into_expr();
    Ok((input, expr))
}

pub(super) fn expr_inner<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        let (mut input, span) = expr_term(state).span().parse_next(input)?;
        state.borrow_mut().on_span(span);

        loop {
            // Parse the next whitespace sequence and only add it as decor suffix to the expression if
            // we actually encounter a traversal, conditional or binary operation. We'll rewind the
            // parser if none of these follow.
            let (remaining_input, suffix) = sp.span().parse_next(input)?;

            // This is essentially a `peek` for the next two bytes to identify the following operation.
            if let Ok((_, peek)) = take::<_, _, ParseError<_>>(2usize).parse_next(remaining_input) {
                match peek {
                    // This might be a `...` operator within a for object expr or after the last
                    // argument of a function call, do not mistakenly parse it as a traversal
                    // operator.
                    b".." => return Ok((input, ())),
                    // Traversal operator.
                    //
                    // Note: after the traversal is consumed, the loop is entered again to consume
                    // a potentially following conditional or binary operation.
                    [b'.' | b'[', _] => {
                        state.borrow_mut().on_ws(suffix);
                        (input, _) = traversal(state).parse_next(remaining_input)?;
                        continue;
                    }
                    // Conditional.
                    [b'?', _] => {
                        state.borrow_mut().on_ws(suffix);
                        return conditional(state).parse_next(remaining_input);
                    }
                    // Binary operation.
                    //
                    // Note: matching a single `=` is ambiguous as it could also be an object
                    // key-value separator, so we'll need to match on `==`.
                    b"=="
                    | [b'!' | b'<' | b'>' | b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|', _] => {
                        state.borrow_mut().on_ws(suffix);
                        return binary_op(state).parse_next(remaining_input);
                    }
                    // None of the above matched.
                    _ => return Ok((input, ())),
                }
            }

            // We hit the end of input.
            return Ok((input, ()));
        }
    }
}

fn expr_term<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        dispatch! {peek(any);
            b'"' => stringlike(state),
            b'[' => array(state),
            b'{' => object(state),
            b'0'..=b'9' => number(state),
            b'<' => heredoc(state),
            b'-' => alt((neg_number(state), unary_op(state))),
            b'!' => unary_op(state),
            b'(' => parenthesis(state),
            b if is_id_start(b) => identlike(state),
            _ => cut_err(fail)
                .context(Context::Expression("expression"))
                .context(Context::Expected(Expected::Char('"')))
                .context(Context::Expected(Expected::Char('[')))
                .context(Context::Expected(Expected::Char('{')))
                .context(Context::Expected(Expected::Char('-')))
                .context(Context::Expected(Expected::Char('!')))
                .context(Context::Expected(Expected::Char('(')))
                .context(Context::Expected(Expected::Char('_')))
                .context(Context::Expected(Expected::Char('<')))
                .context(Context::Expected(Expected::Description("letter")))
                .context(Context::Expected(Expected::Description("digit"))),
        }
        .parse_next(input)
    }
}

fn stringlike<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        alt((
            string.map(|string| {
                state
                    .borrow_mut()
                    .on_expr_term(Expression::String(Decorated::new(string)))
            }),
            string_template.map(|template| {
                state
                    .borrow_mut()
                    .on_expr_term(Expression::Template(template))
            }),
        ))
        .parse_next(input)
    }
}

fn number<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        num.with_recognized()
            .map(|(num, repr)| {
                state
                    .borrow_mut()
                    .on_expr_term(Expression::Number(Formatted::new(num).with_repr(unsafe {
                        from_utf8_unchecked(repr, "`num` filters out non-ascii")
                    })))
            })
            .parse_next(input)
    }
}

fn neg_number<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        preceded((b'-', sp), num)
            .with_recognized()
            .map(|(num, repr)| {
                state
                    .borrow_mut()
                    .on_expr_term(Expression::Number(Formatted::new(-num).with_repr(unsafe {
                        from_utf8_unchecked(repr, "`num` filters out non-ascii")
                    })))
            })
            .parse_next(input)
    }
}

fn traversal<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        many1(prefix_decorated(sp, traversal_operator.map(Decorated::new)))
            .map(|operators| state.borrow_mut().on_traversal(operators))
            .parse_next(input)
    }
}

fn traversal_operator(input: Input) -> IResult<Input, TraversalOperator> {
    dispatch! {any;
        b'.' => prefix_decorated(
            ws,
            dispatch! {peek(any);
                b'*' => one_of(b'*').value(TraversalOperator::AttrSplat(Decorated::new(Splat))),
                b'0'..=b'9' => dec_uint.map(|index: u64| TraversalOperator::LegacyIndex(index.into())),
                b if is_id_start(b) => ident.map(TraversalOperator::GetAttr),
                _ => cut_err(fail)
                    .context(Context::Expression("traversal operator"))
                    .context(Context::Expected(Expected::Char('*')))
                    .context(Context::Expected(Expected::Description("identifier")))
                    .context(Context::Expected(Expected::Description("unsigned integer"))),
            },
        ),
        b'[' => terminated(
            decorated(
                ws,
                dispatch! {peek(any);
                    b'*' => one_of(b'*').value(TraversalOperator::FullSplat(Decorated::new(Splat))),
                    _ => expr.map(TraversalOperator::Index),
                },
                ws,
            ),
            cut_char(']'),
        ),
        _ => fail,
    }
    .parse_next(input)
}

fn unary_op<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        preceded(
            (spanned(unary_operator.map(Spanned::new)), sp.span())
                .map(|(operator, span)| state.borrow_mut().on_unary_op(operator, span)),
            expr_term(state),
        )
        .void()
        .parse_next(input)
    }
}

fn unary_operator(input: Input) -> IResult<Input, UnaryOperator> {
    dispatch! {any;
        b'-' => success(UnaryOperator::Neg),
        b'!' => success(UnaryOperator::Not),
        _ => fail,
    }
    .parse_next(input)
}

fn binary_op<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        (
            spanned(binary_operator.map(Spanned::new)),
            prefix_decorated(sp, expr),
        )
            .map(|(operator, rhs_expr)| state.borrow_mut().on_binary_op(operator, rhs_expr))
            .parse_next(input)
    }
}

fn binary_operator(input: Input) -> IResult<Input, BinaryOperator> {
    dispatch! {any;
        b'=' => one_of(b'=').value(BinaryOperator::Eq),
        b'!' => one_of(b'=').value(BinaryOperator::NotEq),
        b'<' => alt((
            one_of(b'=').value(BinaryOperator::LessEq),
            success(BinaryOperator::Less),
        )),
        b'>' => alt((
            one_of(b'=').value(BinaryOperator::GreaterEq),
            success(BinaryOperator::Greater),
        )),
        b'+' => success(BinaryOperator::Plus),
        b'-' => success(BinaryOperator::Minus),
        b'*' => success(BinaryOperator::Mul),
        b'/' => success(BinaryOperator::Div),
        b'%' => success(BinaryOperator::Mod),
        b'&' => one_of(b'&').value(BinaryOperator::And),
        b'|' => one_of(b'|').value(BinaryOperator::Or),
        _ => fail,
    }
    .parse_next(input)
}

fn conditional<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        (
            preceded(b'?', decorated(sp, expr, sp)),
            preceded(cut_char(':'), prefix_decorated(sp, expr)),
        )
            .map(|(true_expr, false_expr)| state.borrow_mut().on_conditional(true_expr, false_expr))
            .parse_next(input)
    }
}

fn array<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        delimited(
            b'[',
            alt((for_list_expr(state), array_items(state))),
            cut_char(']'),
        )(input)
    }
}

fn for_list_expr<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        (for_intro, decorated(ws, expr, ws), opt(for_cond))
            .map(|(intro, value_expr, cond)| {
                let mut expr = ForExpr::new(intro, value_expr);

                if let Some(cond) = cond {
                    expr.set_cond(cond);
                }

                state
                    .borrow_mut()
                    .on_expr_term(Expression::ForExpr(Box::new(expr)));
            })
            .parse_next(input)
    }
}

fn array_items<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        let values = separated0(decorated(ws, preceded(not(b']'), expr), ws), b',');

        (values, opt(b','), raw_string(ws))
            .map(|(values, comma, trailing)| {
                let mut array = Array::new(values);
                if comma.is_some() {
                    array.set_trailing_comma(true);
                }
                array.set_trailing(trailing);
                state.borrow_mut().on_expr_term(Expression::Array(array));
            })
            .parse_next(input)
    }
}

fn object<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        delimited(
            b'{',
            alt((for_object_expr(state), object_items(state))),
            cut_char('}'),
        )(input)
    }
}

fn for_object_expr<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        (
            for_intro,
            separated_pair(
                decorated(ws, expr, ws),
                cut_tag("=>"),
                decorated(ws, expr, ws),
            ),
            opt(b"..."),
            opt(for_cond),
        )
            .map(|(intro, (key_expr, value_expr), grouping, cond)| {
                let mut expr = ForExpr::new(intro, value_expr);
                expr.set_key_expr(key_expr);
                expr.set_grouping(grouping.is_some());

                if let Some(cond) = cond {
                    expr.set_cond(cond);
                }

                state
                    .borrow_mut()
                    .on_expr_term(Expression::ForExpr(Box::new(expr)));
            })
            .parse_next(input)
    }
}

fn object_items<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        let mut remaining_input = input;
        let mut items = Vec::new();

        loop {
            let start = remaining_input.location();

            let (input, trailing) = raw_string(ws).parse_next(remaining_input)?;
            let (input, ch) = peek(any)(input)?;

            let (input, mut item) = if ch == b'}' {
                let mut object = Object::new(items);
                object.set_trailing(trailing);
                state.borrow_mut().on_expr_term(Expression::Object(object));
                return Ok((input, ()));
            } else {
                let (input, mut item) = object_item(input)?;
                item.key_mut().decor_mut().set_prefix(trailing);
                (input, item)
            };

            // Look for the closing brace and return or consume the object item separator and proceed
            // with the next object item, if any.
            let (input, ch) = peek(any)(input)?;

            let (input, value_terminator) = match ch {
                b'}' => {
                    item.set_span(start..input.location());
                    item.set_value_terminator(ObjectValueTerminator::None);
                    items.push(item);
                    state
                        .borrow_mut()
                        .on_expr_term(Expression::Object(Object::new(items)));
                    return Ok((input, ()));
                }
                b'\r' => crlf
                    .value(ObjectValueTerminator::Newline)
                    .parse_next(input)?,
                b'\n' => newline
                    .value(ObjectValueTerminator::Newline)
                    .parse_next(input)?,
                b',' => take(1usize)
                    .value(ObjectValueTerminator::Comma)
                    .parse_next(input)?,
                b'#' | b'/' => {
                    let (input, comment_span) = line_comment.span().parse_next(input)?;

                    // Associate the trailing comment with the item value, updating
                    // the span if it already has a decor suffix.
                    let suffix_start = match item.value().decor().suffix() {
                        Some(suffix) => suffix.span().unwrap().start,
                        None => comment_span.start,
                    };

                    item.value_mut()
                        .decor_mut()
                        .set_suffix(RawString::from_span(suffix_start..comment_span.end));

                    line_ending
                        .value(ObjectValueTerminator::Newline)
                        .parse_next(input)?
                }
                _ => {
                    return cut_err(fail)
                        .context(Context::Expression("object item"))
                        .context(Context::Expected(Expected::Char('}')))
                        .context(Context::Expected(Expected::Char(',')))
                        .context(Context::Expected(Expected::Char('\n')))
                        .parse_next(input)
                }
            };

            item.set_span(start..input.location());
            item.set_value_terminator(value_terminator);
            items.push(item);
            remaining_input = input;
        }
    }
}

fn object_item(input: Input) -> IResult<Input, ObjectItem> {
    (
        suffix_decorated(object_key, sp),
        object_key_value_separator,
        decorated(sp, expr, sp),
    )
        .map(|(key, key_value_separator, value)| {
            let mut item = ObjectItem::new(key, value);
            item.set_key_value_separator(key_value_separator);
            item
        })
        .parse_next(input)
}

fn object_key(input: Input) -> IResult<Input, ObjectKey> {
    expr.map(|expr| {
        // Variable identifiers without traversal are treated as identifier object keys.
        //
        // Handle this case here by converting the variable into an identifier. This
        // avoids re-parsing the whole key-value pair when an identifier followed by a
        // traversal operator is encountered.
        if let Expression::Variable(variable) = expr {
            ObjectKey::Identifier(Decorated::new(variable.into_inner()))
        } else {
            ObjectKey::Expression(expr)
        }
    })
    .parse_next(input)
}

fn object_key_value_separator(input: Input) -> IResult<Input, ObjectKeyValueSeparator> {
    dispatch! {any;
        b'=' => success(ObjectKeyValueSeparator::Equals),
        b':' => success(ObjectKeyValueSeparator::Colon),
        _ => cut_err(fail)
            .context(Context::Expression("object key-value separator"))
            .context(Context::Expected(Expected::Char('=')))
            .context(Context::Expected(Expected::Char(':'))),
    }
    .parse_next(input)
}

fn for_intro(input: Input) -> IResult<Input, ForIntro> {
    prefix_decorated(
        ws,
        delimited(
            // The `for` tag needs to be followed by either a space character or a comment start to
            // disambiguate. Otherwise an identifier like `format` will match both the `for` tag
            // and the following identifier which would fail parsing of arrays with identifier/func
            // call elements and objects with those as keys.
            (b"for", peek(one_of(" \t#/"))),
            (
                decorated(ws, cut_ident, ws),
                opt(preceded(b',', decorated(ws, cut_ident, ws))),
                preceded(cut_tag("in"), decorated(ws, expr, ws)),
            ),
            cut_char(':'),
        )
        .map(|(first, second, expr)| match second {
            Some(second) => {
                let mut intro = ForIntro::new(second, expr);
                intro.set_key_var(first);
                intro
            }
            None => ForIntro::new(first, expr),
        }),
    )
    .parse_next(input)
}

fn for_cond(input: Input) -> IResult<Input, ForCond> {
    prefix_decorated(
        ws,
        preceded(b"if", decorated(ws, expr, ws)).map(ForCond::new),
    )
    .parse_next(input)
}

fn parenthesis<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        delimited(
            cut_char('('),
            decorated(ws, expr, ws).map(|expr| {
                state
                    .borrow_mut()
                    .on_expr_term(Expression::Parenthesis(Box::new(Decorated::new(expr))))
            }),
            cut_char(')'),
        )(input)
    }
}

fn heredoc<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        let (input, (indented, delim)) = heredoc_start(input)?;

        let (input, (template, trailing)) = (
            spanned(heredoc_template(delim)),
            terminated(
                raw_string(space0),
                cut_err(delim).context(Context::Expected(Expected::Description(
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
        Ok((input, ()))
    }
}

fn heredoc_start(input: Input) -> IResult<Input, (bool, &str)> {
    terminated(
        (
            preceded(b"<<", opt(b'-')).map(|indent| indent.is_some()),
            cut_err(str_ident).context(Context::Expected(Expected::Description("identifier"))),
        ),
        cut_err(line_ending).context(Context::Expected(Expected::Char('\n'))),
    )(input)
}

fn identlike<'i, 's>(
    state: &'s RefCell<ExprParseState>,
) -> impl FnMut(Input<'i>) -> IResult<Input<'i>, ()> + 's {
    move |input: Input<'i>| {
        (str_ident.with_span(), opt(prefix_decorated(ws, func_sig)))
            .map(|((ident, span), signature)| {
                let expr = match signature {
                    Some(signature) => {
                        let name = Decorated::new(Ident::new_unchecked(ident)).spanned(span);
                        let func_call = FuncCall::new(name, signature);
                        Expression::FuncCall(Box::new(func_call))
                    }
                    None => match ident {
                        "null" => Expression::Null(Null.into()),
                        "true" => Expression::Bool(true.into()),
                        "false" => Expression::Bool(false.into()),
                        var => Expression::Variable(Ident::new_unchecked(var).into()),
                    },
                };

                state.borrow_mut().on_expr_term(expr)
            })
            .parse_next(input)
    }
}

fn func_sig(input: Input) -> IResult<Input, FuncSig> {
    let args = separated1(
        decorated(ws, preceded(peek(none_of(",.)")), expr), ws),
        b',',
    );

    #[derive(Copy, Clone)]
    enum Trailer {
        Comma,
        Ellipsis,
    }

    let trailer = dispatch! {any;
        b',' => success(Trailer::Comma),
        b'.' => cut_tag("..").value(Trailer::Ellipsis),
        _ => fail,
    };

    delimited(
        b'(',
        (opt((args, opt(trailer))), raw_string(ws)).map(|(sig, trailing)| {
            let mut sig = match sig {
                Some((args, Some(trailer))) => {
                    let mut sig = FuncSig::new(args);
                    if let Trailer::Ellipsis = trailer {
                        sig.set_expand_final(true);
                    } else {
                        sig.set_trailing_comma(true);
                    }
                    sig
                }
                Some((args, None)) => FuncSig::new(args),
                None => FuncSig::new(Vec::new()),
            };

            sig.set_trailing(trailing);
            sig
        }),
        cut_char(')'),
    )(input)
}

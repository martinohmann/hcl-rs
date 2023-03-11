use super::{
    context::{cut_char, cut_ident, cut_tag, Context, Expected},
    error::ParseError,
    number::number,
    repr::{decorated, prefix_decorated, spanned, suffix_decorated},
    string::{ident, raw_string, str_ident, string},
    template::{heredoc_template, string_template},
    trivia::{line_comment, sp, ws},
    IResult, Input,
};
use crate::{
    expr::*,
    repr::{Decorate, Decorated, SetSpan, Spanned},
    template::HeredocTemplate,
    Ident, RawString,
};
use winnow::{
    branch::alt,
    bytes::{any, none_of, one_of, take},
    character::{crlf, dec_uint, line_ending, newline, space0},
    combinator::{cut_err, fail, opt, peek, success},
    dispatch,
    multi::{many1, separated1},
    sequence::{delimited, preceded, separated_pair, terminated},
    stream::{AsBytes, Location},
    Parser,
};

pub(super) fn expr(input: Input) -> IResult<Input, Expression> {
    let (mut input, mut expr) = spanned(expr_term).parse_next(input)?;

    loop {
        // Parse the next whitespace sequence and only add it as decor suffix to the expression if
        // we actually encounter a traversal, conditional or binary operation. We'll rewind the
        // parser if none of these follow.
        let (remaining_input, suffix) = raw_string(sp).parse_next(input)?;

        // This is essentially a `peek` for the next two bytes to identify the following operation.
        if let Ok((_, peek)) = take::<_, _, ParseError<_>>(2usize).parse_next(remaining_input) {
            match peek {
                // This might be a `...` operator within a for object expr or after the last
                // argument of a function call, do not mistakenly parse it as a traversal
                // operator.
                b".." => return Ok((input, expr)),
                // Traversal operator.
                //
                // Note: after the traversal is consumed, the loop is entered again to consume
                // a potentially following conditional or binary operation.
                [b'.' | b'[', _] => {
                    expr.decor_mut().set_suffix(suffix);
                    (input, expr) = apply_traversal(remaining_input, expr)?;
                    continue;
                }
                // Conditional.
                [b'?', _] => {
                    expr.decor_mut().set_suffix(suffix);
                    return apply_conditional(remaining_input, expr);
                }
                // Binary operation.
                //
                // Note: matching a single `=` is ambiguous as it could also be an object
                // key-value separator, so we'll need to match on `==`.
                b"=="
                | [b'!' | b'<' | b'>' | b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|', _] => {
                    expr.decor_mut().set_suffix(suffix);
                    return apply_binary_op(remaining_input, expr);
                }
                // None of the above matched.
                _ => return Ok((input, expr)),
            }
        }

        // We hit the end of input.
        return Ok((input, expr));
    }
}

fn expr_term(input: Input) -> IResult<Input, Expression> {
    dispatch! {peek(any);
        b'"' => alt((
            string.map(|s| Expression::String(s.into())),
            string_template.map(Expression::Template),
        )),
        b'[' => array,
        b'{' => object,
        b'0'..=b'9' => number.map(|n| Expression::Number(n.into())),
        b'<' => cut_err(heredoc.map(|heredoc| Expression::HeredocTemplate(Box::new(heredoc))))
            .context(Context::Expression("heredoc template")),
        b'-' => alt((
            preceded((b'-', sp), number).map(|n| Expression::Number((-n).into())),
            (
                spanned(one_of('-').value(UnaryOperator::Neg).map(Spanned::new)),
                prefix_decorated(sp, expr_term),
            )
                .map(|(operator, expr)| {
                    Expression::UnaryOp(Box::new(UnaryOp::new(operator, expr)))
                }),
        )),
        b'!' => (
            spanned(take(1usize).value(UnaryOperator::Not).map(Spanned::new)),
            prefix_decorated(sp, expr_term),
        )
            .map(|(operator, expr)| Expression::UnaryOp(Box::new(UnaryOp::new(operator, expr)))),
        b'(' => parenthesis.map(|expr| Expression::Parenthesis(Box::new(expr.into()))),
        b'_' | b'a'..=b'z' | b'A'..=b'Z' => identlike,
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

fn apply_traversal(input: Input, expr_term: Expression) -> IResult<Input, Expression> {
    let mut traversal = many1(prefix_decorated(sp, traversal_operator.map(Decorated::new)));

    let (input, operators) = traversal.parse_next(input)?;
    let traversal = Traversal::new(expr_term, operators);
    let expr = Expression::Traversal(Box::new(traversal));
    Ok((input, expr))
}

fn traversal_operator(input: Input) -> IResult<Input, TraversalOperator> {
    dispatch! {any;
        b'.' => prefix_decorated(
            ws,
            cut_err(alt((
                one_of('*').value(TraversalOperator::AttrSplat(Decorated::new(()))),
                ident.map(TraversalOperator::GetAttr),
                dec_uint.map(|index: u64| TraversalOperator::LegacyIndex(index.into())),
            )))
            .context(Context::Expression("traversal operator"))
            .context(Context::Expected(Expected::Char('*')))
            .context(Context::Expected(Expected::Description("identifier")))
            .context(Context::Expected(Expected::Description("unsigned integer"))),
        ),
        b'[' => terminated(
            decorated(
                ws,
                cut_err(alt((
                    one_of('*').value(TraversalOperator::FullSplat(Decorated::new(()))),
                    expr.map(TraversalOperator::Index),
                )))
                .context(Context::Expression("traversal operator"))
                .context(Context::Expected(Expected::Char('*')))
                .context(Context::Expected(Expected::Description("expression"))),
                ws,
            ),
            cut_char(']'),
        ),
        _ => fail,
    }
    .parse_next(input)
}

fn apply_binary_op(input: Input, lhs_expr: Expression) -> IResult<Input, Expression> {
    let mut binary_op = (
        spanned(binary_operator.map(Spanned::new)),
        prefix_decorated(sp, expr),
    );

    let (input, (operator, rhs_expr)) = binary_op.parse_next(input)?;
    let binary_op = BinaryOp::new(lhs_expr, operator, rhs_expr);
    let expr = Expression::BinaryOp(Box::new(binary_op));
    Ok((input, expr))
}

fn binary_operator(input: Input) -> IResult<Input, BinaryOperator> {
    dispatch! {any;
        b'=' => one_of('=').value(BinaryOperator::Eq),
        b'!' => one_of('=').value(BinaryOperator::NotEq),
        b'<' => alt((
            one_of('=').value(BinaryOperator::LessEq),
            success(BinaryOperator::Less),
        )),
        b'>' => alt((
            one_of('=').value(BinaryOperator::GreaterEq),
            success(BinaryOperator::Greater),
        )),
        b'+' => success(BinaryOperator::Plus),
        b'-' => success(BinaryOperator::Minus),
        b'*' => success(BinaryOperator::Mul),
        b'/' => success(BinaryOperator::Div),
        b'%' => success(BinaryOperator::Mod),
        b'&' => one_of('&').value(BinaryOperator::And),
        b'|' => one_of('|').value(BinaryOperator::Or),
        _ => fail,
    }
    .parse_next(input)
}

fn apply_conditional(input: Input, cond_expr: Expression) -> IResult<Input, Expression> {
    let mut conditional = (
        preceded(b'?', decorated(sp, expr, sp)),
        preceded(cut_char(':'), prefix_decorated(sp, expr)),
    );

    let (input, (true_expr, false_expr)) = conditional.parse_next(input)?;
    let conditional = Conditional::new(cond_expr, true_expr, false_expr);
    let expr = Expression::Conditional(Box::new(conditional));
    Ok((input, expr))
}

fn array(input: Input) -> IResult<Input, Expression> {
    delimited(
        b'[',
        alt((
            for_list_expr.map(|expr| Expression::ForExpr(Box::new(expr))),
            array_items.map(|array| Expression::Array(Box::new(array))),
        )),
        cut_char(']'),
    )(input)
}

fn for_list_expr(input: Input) -> IResult<Input, ForExpr> {
    (for_intro, decorated(ws, expr, ws), opt(for_cond))
        .map(|(intro, value_expr, cond)| {
            let mut expr = ForExpr::new(intro, value_expr);

            if let Some(cond) = cond {
                expr.set_cond(cond);
            }

            expr
        })
        .parse_next(input)
}

fn array_items(input: Input) -> IResult<Input, Array> {
    let values = separated1(decorated(ws, preceded(peek(none_of("]")), expr), ws), b',');

    alt((
        (values, opt(preceded(b',', raw_string(ws)))).map(|(values, trailing)| {
            let mut array = Array::new(values);
            if let Some(trailing) = trailing {
                array.set_trailing_comma(true);
                array.set_trailing(trailing);
            }
            array
        }),
        raw_string(ws).map(|trailing| {
            let mut array = Array::default();
            array.set_trailing(trailing);
            array
        }),
    ))(input)
}

fn object(input: Input) -> IResult<Input, Expression> {
    delimited(
        b'{',
        alt((
            for_object_expr.map(|expr| Expression::ForExpr(Box::new(expr))),
            object_items.map(|object| Expression::Object(Box::new(object))),
        )),
        cut_char('}'),
    )(input)
}

fn for_object_expr(input: Input) -> IResult<Input, ForExpr> {
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

            expr
        })
        .parse_next(input)
}

fn object_items(input: Input) -> IResult<Input, Object> {
    let mut remaining_input = input;
    let mut items = Vec::new();

    loop {
        let start = remaining_input.location();

        let (input, trailing) = raw_string(ws).parse_next(remaining_input)?;
        let (input, ch) = peek(any)(input)?;

        let (input, mut item) = if ch == b'}' {
            let mut object = Object::new(items);
            object.set_trailing(trailing);
            return Ok((input, object));
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
                return Ok((input, Object::new(items)));
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

fn object_item(input: Input) -> IResult<Input, ObjectItem> {
    (
        object_key,
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
    suffix_decorated(
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
        }),
        sp,
    )
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

fn parenthesis(input: Input) -> IResult<Input, Expression> {
    delimited(cut_char('('), decorated(ws, expr, ws), cut_char(')'))(input)
}

fn heredoc(input: Input) -> IResult<Input, HeredocTemplate> {
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

    Ok((input, heredoc))
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

fn identlike(input: Input) -> IResult<Input, Expression> {
    (str_ident.with_span(), opt(prefix_decorated(ws, func_sig)))
        .map(|((ident, span), signature)| match signature {
            Some(signature) => {
                let name = Decorated::new(Ident::new_unchecked(ident)).spanned(span);
                let func_call = FuncCall::new(name, signature);
                Expression::FuncCall(Box::new(func_call))
            }
            None => match ident {
                "null" => Expression::Null(().into()),
                "true" => Expression::Bool(true.into()),
                "false" => Expression::Bool(false.into()),
                var => Expression::Variable(Ident::new_unchecked(var).into()),
            },
        })
        .parse_next(input)
}

fn func_sig(input: Input) -> IResult<Input, FuncSig> {
    delimited(
        b'(',
        alt((
            (
                separated1(
                    decorated(ws, preceded(peek(none_of(",.)")), expr), ws),
                    b',',
                ),
                opt((alt((b",", b"...")), raw_string(ws))),
            )
                .map(|(args, trailer)| {
                    let mut sig = FuncSig::new(args);

                    if let Some((sep, trailing)) = trailer {
                        if sep.as_bytes() == b"..." {
                            sig.set_expand_final(true);
                        } else {
                            sig.set_trailing_comma(true);
                        }

                        sig.set_trailing(trailing);
                    }

                    sig
                }),
            raw_string(ws).map(|trailing| {
                let mut sig = FuncSig::new(Vec::new());
                sig.set_trailing(trailing);
                sig
            }),
        )),
        cut_char(')'),
    )(input)
}

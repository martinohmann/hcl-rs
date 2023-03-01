use super::ast::{
    Array, BinaryOp, Conditional, Expression, ForCond, ForExpr, ForIntro, FuncCall, FuncSig,
    HeredocTemplate, Object, ObjectItem, ObjectKey, ObjectKeyValueSeparator, ObjectValueTerminator,
    Traversal, TraversalOperator, UnaryOp,
};
use super::{
    cut_char, cut_ident, cut_tag, decor,
    error::{Context, Expected},
    ident, line_comment, number, prefix_decor, raw,
    repr::{Decorate, Decorated, RawString, SetSpan, Span, Spanned},
    sp, spanned, str_ident, string, suffix_decor,
    template::{heredoc_template, string_template},
    ws, IResult, Input,
};
use crate::expr::{BinaryOperator, UnaryOperator, Variable};
use crate::Identifier;
use winnow::{
    branch::alt,
    bytes::{any, none_of, one_of, tag, take},
    character::{crlf, dec_uint, line_ending, newline, space0},
    combinator::{cut_err, fail, not, opt, peek, success},
    dispatch,
    multi::{many1, separated1},
    sequence::{delimited, preceded, separated_pair, terminated},
    stream::{AsBytes, Location},
    Parser,
};

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

fn array_items(input: Input) -> IResult<Input, Array> {
    let values = separated1(decor(ws, preceded(peek(none_of("]")), expr), ws), b',');

    alt((
        (values, opt(preceded(b',', raw(ws)))).map(|(values, trailing)| {
            let mut array = Array::new(values);
            if let Some(trailing) = trailing {
                array.set_trailing_comma(true);
                array.set_trailing(trailing);
            }
            array
        }),
        raw(ws).map(|trailing| {
            let mut array = Array::default();
            array.set_trailing(trailing);
            array
        }),
    ))(input)
}

fn for_list_expr(input: Input) -> IResult<Input, ForExpr> {
    (for_intro, decor(ws, expr, ws), opt(for_cond))
        .map(|(intro, value_expr, cond)| {
            let mut expr = ForExpr::new(intro, value_expr);

            if let Some(cond) = cond {
                expr.set_cond(cond);
            }

            expr
        })
        .parse_next(input)
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

fn object_items(input: Input) -> IResult<Input, Object> {
    let mut remaining_input = input;
    let mut items = Vec::new();

    loop {
        let start = remaining_input.location();

        let (input, trailing) = raw(ws).parse_next(remaining_input)?;
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

fn object_key(input: Input) -> IResult<Input, ObjectKey> {
    suffix_decor(
        expr.map(|expr| {
            // Variable identifiers without traversal are treated as identifier object keys.
            //
            // Handle this case here by converting the variable into an identifier. This
            // avoids re-parsing the whole key-value pair when an identifier followed by a
            // traversal operator is encountered.
            if let Expression::Variable(variable) = expr {
                ObjectKey::Identifier(Decorated::new(variable.into_inner().into()))
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

fn object_item(input: Input) -> IResult<Input, ObjectItem> {
    (object_key, object_key_value_separator, decor(sp, expr, sp))
        .map(|(key, key_value_separator, value)| {
            let mut item = ObjectItem::new(key, value);
            item.set_key_value_separator(key_value_separator);
            item
        })
        .parse_next(input)
}

fn for_object_expr(input: Input) -> IResult<Input, ForExpr> {
    (
        for_intro,
        separated_pair(decor(ws, expr, ws), cut_tag("=>"), decor(ws, expr, ws)),
        opt(tag("...")),
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

fn for_intro(input: Input) -> IResult<Input, ForIntro> {
    prefix_decor(
        ws,
        delimited(
            // The `for` tag needs to be followed by either a space character or a comment start to
            // disambiguate. Otherwise an identifier like `format` will match both the `for` tag
            // and the following identifier which would fail parsing of arrays with identifier/func
            // call elements and objects with those as keys.
            (tag("for"), peek(one_of(" \t#/"))),
            (
                decor(ws, cut_ident, ws),
                opt(preceded(b',', decor(ws, cut_ident, ws))),
                preceded(cut_tag("in"), decor(ws, expr, ws)),
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
    prefix_decor(
        ws,
        preceded(tag("if"), decor(ws, expr, ws)).map(ForCond::new),
    )
    .parse_next(input)
}

fn parenthesis(input: Input) -> IResult<Input, Expression> {
    delimited(cut_char('('), decor(ws, expr, ws), cut_char(')'))(input)
}

fn heredoc_start(input: Input) -> IResult<Input, (bool, &str)> {
    terminated(
        (
            preceded(tag("<<"), opt(b'-')).map(|indent| indent.is_some()),
            cut_err(str_ident).context(Context::Expected(Expected::Description("identifier"))),
        ),
        cut_err(line_ending).context(Context::Expected(Expected::Char('\n'))),
    )(input)
}

fn heredoc(input: Input) -> IResult<Input, HeredocTemplate> {
    let (input, (indented, delim)) = heredoc_start(input)?;

    let (input, (template, trailing)) = (
        spanned(heredoc_template(delim)),
        terminated(
            raw(space0),
            cut_err(tag(delim)).context(Context::Expected(Expected::Description(
                "heredoc end delimiter",
            ))),
        ),
    )
        .parse_next(input)?;

    let mut heredoc = HeredocTemplate::new(Identifier::unchecked(delim), template);

    if indented {
        heredoc.dedent();
    }

    heredoc.set_trailing(trailing);

    Ok((input, heredoc))
}

fn traversal_operator(input: Input) -> IResult<Input, TraversalOperator> {
    dispatch! {any;
        b'.' => prefix_decor(
            ws,
            preceded(
                // Must not match `for` object value grouping or func call expand final which
                // are both `...`.
                not(b'.'),
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
        ),
        b'[' => terminated(
            decor(
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

fn ident_or_func_call(input: Input) -> IResult<Input, Expression> {
    (str_ident.with_span(), opt(prefix_decor(ws, func_sig)))
        .map(|((ident, span), signature)| match signature {
            Some(signature) => {
                let name = Decorated::new(Identifier::unchecked(ident)).spanned(span);
                let func_call = FuncCall::new(name, signature);
                Expression::FuncCall(Box::new(func_call))
            }
            None => match ident {
                "null" => Expression::Null(().into()),
                "true" => Expression::Bool(true.into()),
                "false" => Expression::Bool(false.into()),
                var => Expression::Variable(Variable::unchecked(var).into()),
            },
        })
        .parse_next(input)
}

fn func_sig(input: Input) -> IResult<Input, FuncSig> {
    delimited(
        b'(',
        alt((
            (
                separated1(decor(ws, preceded(peek(none_of(",.)")), expr), ws), b','),
                opt((alt((tag(","), tag("..."))), raw(ws))),
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
            raw(ws).map(|trailing| {
                let mut sig = FuncSig::new(Vec::new());
                sig.set_trailing(trailing);
                sig
            }),
        )),
        cut_char(')'),
    )(input)
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
                spanned(take(1usize).value(UnaryOperator::Neg).map(Spanned::new)),
                prefix_decor(sp, expr_term),
            )
                .map(|(operator, expr)| {
                    Expression::UnaryOp(Box::new(UnaryOp::new(operator, expr)))
                }),
        )),
        b'!' => (
            spanned(take(1usize).value(UnaryOperator::Not).map(Spanned::new)),
            prefix_decor(sp, expr_term),
        )
            .map(|(operator, expr)| Expression::UnaryOp(Box::new(UnaryOp::new(operator, expr)))),
        b'(' => parenthesis.map(|expr| Expression::Parenthesis(Box::new(expr.into()))),
        b'_' | b'a'..=b'z' | b'A'..=b'Z' => ident_or_func_call,
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

pub fn expr(input: Input) -> IResult<Input, Expression> {
    let mut traversal = many1(prefix_decor(sp, traversal_operator.map(Decorated::new))).with_span();

    let mut conditional = (
        sp.span(),
        preceded(b'?', decor(sp, expr, sp)),
        preceded(cut_char(':'), prefix_decor(sp, expr)),
    );

    let mut binary_op = (
        prefix_decor(sp, binary_operator.map(Decorated::new)),
        prefix_decor(sp, expr),
    );

    let (input, expr) = spanned(expr_term).parse_next(input)?;

    let (input, mut expr) = match traversal.parse_next(input) {
        Ok((input, (operators, span))) => {
            let expr_start = expr.span().map_or(0, |span| span.start);
            let mut expr = Expression::Traversal(Box::new(Traversal::new(expr, operators)));
            expr.set_span(expr_start..span.end);
            (input, expr)
        }
        Err(winnow::error::ErrMode::Cut(e)) => return Err(winnow::error::ErrMode::Cut(e)),
        Err(_) => (input, expr),
    };

    match peek(preceded(sp, any)).parse_next(input) {
        Ok((input, ch)) => {
            match ch {
                b'?' => {
                    let (input, (suffix_span, true_expr, false_expr)) =
                        conditional.parse_next(input)?;

                    // Associate whitespace preceding the `?` with the cond expression, updating
                    // the span if it already has a decor suffix.
                    let suffix_start = match expr.decor().suffix() {
                        Some(suffix) => suffix.span().unwrap().start,
                        None => suffix_span.start,
                    };

                    expr.decor_mut()
                        .set_suffix(RawString::from_span(suffix_start..suffix_span.end));

                    let cond = Conditional::new(expr, true_expr, false_expr);
                    let expr = Expression::Conditional(Box::new(cond));
                    Ok((input, expr))
                }
                b'=' | b'!' | b'<' | b'>' | b'+' | b'-' | b'*' | b'/' | b'%' | b'&' | b'|' => {
                    match binary_op.parse_next(input) {
                        Ok((input, (operator, rhs_expr))) => {
                            let op = BinaryOp::new(expr, operator, rhs_expr);
                            let expr = Expression::BinaryOp(Box::new(op));
                            Ok((input, expr))
                        }
                        Err(winnow::error::ErrMode::Cut(e)) => Err(winnow::error::ErrMode::Cut(e)),
                        Err(_) => Ok((input, expr)),
                    }
                }
                _ => Ok((input, expr)),
            }
        }
        Err(_) => Ok((input, expr)),
    }
}

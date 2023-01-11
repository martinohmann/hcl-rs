use super::{
    boolean, ident, null, number, opt_sep, sp_delimited, str_ident, string, ws_delimited,
    ws_preceded, ws_terminated,
};
use crate::expr::{
    BinaryOp, BinaryOperator, Conditional, Expression, ForExpr, FuncCall, Heredoc,
    HeredocStripMode, Object, ObjectKey, TemplateExpr, Traversal, TraversalOperator, UnaryOp,
    UnaryOperator, Variable,
};
use crate::util::is_templated;
use crate::Identifier;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char, line_ending, one_of, space0, u64},
    combinator::{cut, map, not, opt, recognize, value},
    error::context,
    multi::{many0, many1, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

fn array(input: &str) -> IResult<&str, Vec<Expression>> {
    context(
        "array",
        map(
            delimited(
                ws_terminated(char('[')),
                opt(terminated(
                    separated_list1(ws_delimited(char(',')), expr),
                    ws_terminated(opt_sep(char(','))),
                )),
                char(']'),
            ),
            Option::unwrap_or_default,
        ),
    )(input)
}

fn object(input: &str) -> IResult<&str, Object<ObjectKey, Expression>> {
    context(
        "object",
        map(
            delimited(
                ws_terminated(char('{')),
                opt(many1(terminated(
                    object_key_value,
                    ws_terminated(opt_sep(one_of(",\n"))),
                ))),
                char('}'),
            ),
            |items| Object::from(items.unwrap_or_default()),
        ),
    )(input)
}

fn object_key_value(input: &str) -> IResult<&str, (ObjectKey, Expression)> {
    separated_pair(
        map(expr, |expr| {
            // Variable identifiers without traversal are treated as identifier object keys. This
            // allows us to avoid re-parsing the whole key-value pair when an identifier followed
            // by a traversal operator is encountered.
            if let Expression::Variable(variable) = expr {
                ObjectKey::Identifier(variable.into_inner())
            } else {
                ObjectKey::Expression(expr)
            }
        }),
        sp_delimited(one_of("=:")),
        cut(expr),
    )(input)
}

fn parenthesis(input: &str) -> IResult<&str, Box<Expression>> {
    map(delimited(tag("("), ws_delimited(expr), tag(")")), Box::new)(input)
}

fn string_or_template(input: &str) -> IResult<&str, Expression> {
    map(string, |s| {
        if is_templated(&s) {
            Expression::from(TemplateExpr::QuotedString(s))
        } else {
            Expression::String(s)
        }
    })(input)
}

fn heredoc_start(input: &str) -> IResult<&str, (HeredocStripMode, &str)> {
    terminated(
        pair(
            alt((
                value(HeredocStripMode::Indent, tag("<<-")),
                value(HeredocStripMode::None, tag("<<")),
            )),
            str_ident,
        ),
        pair(space0, line_ending),
    )(input)
}

fn heredoc_end<'a>(delim: &'a str) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str> {
    recognize(tuple((line_ending, space0, tag(delim))))
}

fn heredoc_template<'a>(delim: &'a str) -> impl FnMut(&'a str) -> IResult<&'a str, String> {
    move |input: &'a str| {
        map(
            recognize(many1(preceded(not(heredoc_end(delim)), anychar))),
            |template| {
                // Append the trailing newline here. This is easier than doing this via the parser combinators.
                let mut template = template.to_owned();
                template.push('\n');
                template
            },
        )(input)
    }
}

fn heredoc(input: &str) -> IResult<&str, Heredoc> {
    let (input, (strip, delim)) = heredoc_start(input)?;

    map(
        terminated(heredoc_template(delim), heredoc_end(delim)),
        move |template| Heredoc {
            delimiter: Identifier::unchecked(delim),
            template,
            strip,
        },
    )(input)
}

fn traversal_operator(input: &str) -> IResult<&str, TraversalOperator> {
    alt((
        preceded(
            ws_terminated(char('.')),
            alt((
                value(TraversalOperator::AttrSplat, char('*')),
                map(ident, TraversalOperator::GetAttr),
                map(u64, TraversalOperator::LegacyIndex),
            )),
        ),
        delimited(
            ws_terminated(char('[')),
            alt((
                value(TraversalOperator::FullSplat, char('*')),
                map(expr, TraversalOperator::Index),
            )),
            ws_preceded(char(']')),
        ),
    ))(input)
}

fn variable_or_func_call(input: &str) -> IResult<&str, Expression> {
    map(
        pair(ident, opt(ws_preceded(func_sig))),
        |(name, sig)| match sig {
            Some((args, expand_final)) => Expression::from(FuncCall {
                name,
                args,
                expand_final,
            }),
            None => Expression::from(Variable::from(name)),
        },
    )(input)
}

fn func_sig(input: &str) -> IResult<&str, (Vec<Expression>, bool)> {
    context(
        "func signature",
        map(
            delimited(
                ws_terminated(char('(')),
                opt(pair(
                    separated_list1(ws_delimited(char(',')), expr),
                    ws_terminated(opt_sep(alt((tag(","), tag("..."))))),
                )),
                char(')'),
            ),
            |pair| {
                pair.map(|(args, trailer)| (args, trailer == Some("...")))
                    .unwrap_or_default()
            },
        ),
    )(input)
}

struct ForIntro {
    key_var: Option<Identifier>,
    value_var: Identifier,
    collection_expr: Expression,
}

fn for_intro(input: &str) -> IResult<&str, ForIntro> {
    map(
        delimited(
            ws_terminated(tag("for")),
            tuple((
                ident,
                opt(preceded(ws_delimited(char(',')), ident)),
                preceded(ws_delimited(tag("in")), expr),
            )),
            ws_preceded(char(':')),
        ),
        |(first, second, expr)| match second {
            Some(second) => ForIntro {
                key_var: Some(first),
                value_var: second,
                collection_expr: expr,
            },
            None => ForIntro {
                key_var: None,
                value_var: first,
                collection_expr: expr,
            },
        },
    )(input)
}

fn for_cond_expr(input: &str) -> IResult<&str, Expression> {
    preceded(ws_terminated(tag("if")), expr)(input)
}

fn for_list_expr(input: &str) -> IResult<&str, ForExpr> {
    map(
        tuple((
            ws_terminated(for_intro),
            expr,
            opt(ws_preceded(for_cond_expr)),
        )),
        |(intro, value_expr, cond_expr)| ForExpr {
            key_var: intro.key_var,
            value_var: intro.value_var,
            collection_expr: intro.collection_expr,
            key_expr: None,
            value_expr,
            cond_expr,
            grouping: false,
        },
    )(input)
}

fn for_object_expr(input: &str) -> IResult<&str, ForExpr> {
    map(
        tuple((
            ws_terminated(for_intro),
            separated_pair(expr, ws_delimited(tag("=>")), expr),
            opt(ws_preceded(tag("..."))),
            opt(ws_preceded(for_cond_expr)),
        )),
        |(intro, (key_expr, value_expr), grouping, cond_expr)| ForExpr {
            key_var: intro.key_var,
            value_var: intro.value_var,
            collection_expr: intro.collection_expr,
            key_expr: Some(key_expr),
            value_expr,
            cond_expr,
            grouping: grouping.is_some(),
        },
    )(input)
}

fn for_expr(input: &str) -> IResult<&str, ForExpr> {
    alt((
        delimited(
            ws_terminated(char('[')),
            for_list_expr,
            ws_preceded(char(']')),
        ),
        delimited(
            ws_terminated(char('{')),
            for_object_expr,
            ws_preceded(char('}')),
        ),
    ))(input)
}

fn unary_operator(input: &str) -> IResult<&str, UnaryOperator> {
    context(
        "unary operator",
        alt((
            value(UnaryOperator::Neg, char('-')),
            value(UnaryOperator::Not, char('!')),
        )),
    )(input)
}

fn binary_operator(input: &str) -> IResult<&str, BinaryOperator> {
    context(
        "binary operator",
        alt((
            value(BinaryOperator::Eq, tag("==")),
            value(BinaryOperator::NotEq, tag("!=")),
            value(BinaryOperator::LessEq, tag("<=")),
            value(BinaryOperator::GreaterEq, tag(">=")),
            value(BinaryOperator::Less, char('<')),
            value(BinaryOperator::Greater, char('>')),
            value(BinaryOperator::Plus, char('+')),
            value(BinaryOperator::Minus, char('-')),
            value(BinaryOperator::Mul, char('*')),
            value(BinaryOperator::Div, char('/')),
            value(BinaryOperator::Mod, char('%')),
            value(BinaryOperator::And, tag("&&")),
            value(BinaryOperator::Or, tag("||")),
        )),
    )(input)
}

fn expr_term(input: &str) -> IResult<&str, Expression> {
    map(
        pair(
            alt((
                string_or_template,
                variable_or_func_call,
                map(number, Expression::Number),
                map(boolean, Expression::Bool),
                map(null, |_| Expression::Null),
                map(array, Expression::Array),
                map(object, Expression::Object),
                map(for_expr, Expression::from),
                map(parenthesis, Expression::Parenthesis),
                map(heredoc, Expression::from),
            )),
            many0(ws_preceded(traversal_operator)),
        ),
        |(expr, operators)| {
            if operators.is_empty() {
                expr
            } else {
                Expression::from(Traversal { expr, operators })
            }
        },
    )(input)
}

pub fn expr(input: &str) -> IResult<&str, Expression> {
    let unary_op = ws_terminated(unary_operator);

    let binary_op = pair(ws_delimited(binary_operator), expr);

    let conditional = pair(
        preceded(sp_delimited(char('?')), expr),
        preceded(sp_delimited(char(':')), expr),
    );

    context(
        "expression",
        map(
            tuple((opt(unary_op), expr_term, opt(binary_op), opt(conditional))),
            |(unary_op, expr, binary_op, conditional)| {
                let lhs_expr = match unary_op {
                    Some(operator) => {
                        // Negative numbers are implemented as unary negation operations in the HCL
                        // spec. We'll convert these to negative numbers to make them more
                        // convenient to use.
                        match (operator, expr) {
                            (UnaryOperator::Neg, Expression::Number(num)) => {
                                Expression::Number(-num)
                            }
                            (operator, expr) => Expression::from(UnaryOp { operator, expr }),
                        }
                    }
                    None => expr,
                };

                let expr = match binary_op {
                    Some((operator, rhs_expr)) => Expression::from(BinaryOp {
                        lhs_expr,
                        operator,
                        rhs_expr,
                    }),
                    None => lhs_expr,
                };

                match conditional {
                    Some((true_expr, false_expr)) => Expression::from(Conditional {
                        cond_expr: expr,
                        true_expr,
                        false_expr,
                    }),
                    None => expr,
                }
            },
        ),
    )(input)
}

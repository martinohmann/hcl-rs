use super::*;
use crate::expr::Variable;
use std::fmt;
use std::str::FromStr;

#[track_caller]
fn eval_to_ctx<T, U>(ctx: &Context, value: T, expected: U)
where
    T: Evaluate<Output = U> + fmt::Debug + PartialEq,
    U: fmt::Debug + PartialEq,
{
    assert_eq!(value.evaluate(ctx).unwrap(), expected);
}

#[track_caller]
fn eval_to<T, U>(value: T, expected: U)
where
    T: Evaluate<Output = U> + fmt::Debug + PartialEq,
    U: fmt::Debug + PartialEq,
{
    let ctx = Context::new();
    eval_to_ctx(&ctx, value, expected);
}

#[track_caller]
fn eval_error<T, E>(value: T, expected: E)
where
    T: Evaluate + fmt::Debug + PartialEq,
    <T as Evaluate>::Output: fmt::Debug,
    E: Into<Error>,
{
    let ctx = Context::new();
    let err = value.evaluate(&ctx).unwrap_err();
    let expected = expected.into();
    assert_eq!(err.kind(), expected.kind());
    assert_eq!(err.expr(), expected.expr());
}

#[test]
fn eval_binary_op() {
    use {BinaryOperator::*, Operation::*};

    eval_to(
        BinaryOp::new(
            Binary(BinaryOp::new(1, Div, 2)),
            Mul,
            Binary(BinaryOp::new(3, Plus, Binary(BinaryOp::new(4, Div, 5)))),
        ),
        Value::from(2.3),
    );
    eval_to(BinaryOp::new("foo", Eq, "foo"), Value::from(true));
    eval_to(BinaryOp::new(false, Or, true), Value::from(true));
    eval_to(BinaryOp::new(true, And, true), Value::from(true));
    eval_to(BinaryOp::new(true, And, false), Value::from(false));
    eval_to(BinaryOp::new(1, Less, 2), Value::from(true));
    eval_to(
        BinaryOp::new(
            Binary(BinaryOp::new(1, Greater, 0)),
            And,
            Binary(BinaryOp::new("foo", NotEq, Expression::Null)),
        ),
        Value::from(true),
    );
}

#[test]
fn eval_conditional() {
    eval_to(Conditional::new(true, "yes", "no"), Value::from("yes"));
    eval_to(Conditional::new(false, "yes", "no"), Value::from("no"));
    eval_error(
        Conditional::new("foo", "yes", "no"),
        ErrorKind::Unexpected(Value::from("foo"), "a boolean"),
    );
}

#[test]
fn eval_for_expr() {
    eval_to(
        ForExpr::new(
            Identifier::unchecked("item"),
            Expression::from_iter([1, 2, 3, 4, 5, 6, 7]),
            Operation::Binary(BinaryOp::new(
                Variable::unchecked("item"),
                BinaryOperator::Mul,
                2,
            )),
        )
        .with_cond_expr(Operation::Binary(BinaryOp::new(
            Variable::unchecked("item"),
            BinaryOperator::Less,
            5,
        ))),
        Value::from_iter([2, 4, 6, 8]),
    );

    eval_to(
        ForExpr::new(
            Identifier::unchecked("value"),
            Expression::from_iter([("a", "1"), ("b", "2"), ("c", "3"), ("d", "4")]),
            Variable::unchecked("key"),
        )
        .with_key_var(Identifier::unchecked("key"))
        .with_key_expr(Variable::unchecked("value"))
        .with_cond_expr(Operation::Binary(BinaryOp::new(
            Variable::unchecked("key"),
            BinaryOperator::NotEq,
            Expression::from("d"),
        ))),
        Value::from_iter([("1", "a"), ("2", "b"), ("3", "c")]),
    );

    eval_to(
        ForExpr::new(
            Identifier::unchecked("value"),
            Expression::from_iter(["a", "b", "c", "d"]),
            Variable::unchecked("value"),
        )
        .with_key_var(Identifier::unchecked("index"))
        .with_key_expr(TemplateExpr::QuotedString("${index}".into())),
        Value::from_iter([("0", "a"), ("1", "b"), ("2", "c"), ("3", "d")]),
    );

    eval_to(
        ForExpr::new(
            Identifier::unchecked("value"),
            Expression::from_iter([("a", "1"), ("b", "2"), ("c", "3"), ("d", "4")]),
            Variable::unchecked("key"),
        )
        .with_key_var(Identifier::unchecked("key")),
        Value::from_iter(["a", "b", "c", "d"]),
    );

    eval_to(
        ForExpr::new(
            Identifier::unchecked("value"),
            Expression::from_iter(["a", "b", "c", "d"]),
            Variable::unchecked("value"),
        )
        .with_key_var(Identifier::unchecked("index"))
        .with_key_expr(TemplateExpr::from("${index}")),
        Value::from_iter([("0", "a"), ("1", "b"), ("2", "c"), ("3", "d")]),
    );

    eval_to(
        ForExpr::new(
            Identifier::unchecked("value"),
            Expression::from_iter([("a", 1), ("b", 2), ("c", 3), ("d", 4)]),
            Variable::unchecked("value"),
        )
        .with_key_var(Identifier::unchecked("key"))
        .with_key_expr(Expression::from("foo"))
        .with_cond_expr(Operation::Binary(BinaryOp::new(
            Variable::unchecked("key"),
            BinaryOperator::NotEq,
            Expression::from("d"),
        )))
        .with_grouping(true),
        Value::from_iter([("foo", vec![1, 2, 3])]),
    );
}

#[test]
fn eval_traversal() {
    use TraversalOperator::*;

    // legacy index access: expr.2
    eval_to(
        Traversal::new(vec![1, 2, 3], [LegacyIndex(1)]),
        Value::from(2),
    );

    // legacy index access: expr[2]
    eval_to(
        Traversal::new(vec![1, 2, 3], [Index(Expression::from(2))]),
        Value::from(3),
    );

    // get-attr: expr.foo
    eval_to(
        Traversal::new(
            expression!({"foo" = [1, 2, 3], "bar" = []}),
            [GetAttr(Identifier::unchecked("foo"))],
        ),
        Value::from_iter([1, 2, 3]),
    );

    // chain get-attr -> index: expr.foo[2]
    eval_to(
        Traversal::new(
            Traversal::new(
                expression!({"foo" = [1, 2, 3], "bar" = []}),
                [GetAttr(Identifier::unchecked("foo"))],
            ),
            [Index(Expression::from(2))],
        ),
        Value::from(3),
    );

    // full-splat non-array
    eval_to(
        Traversal::new(
            expression!({"foo" = [1, 2, 3], "bar" = []}),
            [FullSplat, GetAttr(Identifier::unchecked("foo"))],
        ),
        Value::from_iter([vec![1, 2, 3]]),
    );

    // full-splat array
    eval_to(
        Traversal::new(
            expression! {
                [
                    { "foo" = 2 },
                    { "foo" = 1, "bar" = 2 }
                ]
            },
            [FullSplat, GetAttr(Identifier::unchecked("foo"))],
        ),
        Value::from_iter([2, 1]),
    );

    // full-splat null
    eval_to(
        Traversal::new(
            Expression::Null,
            [FullSplat, GetAttr(Identifier::unchecked("foo"))],
        ),
        Value::Array(vec![]),
    );

    // attr-splat non-array
    eval_to(
        Traversal::new(
            expression!({"foo" = [1, 2, 3], "bar" = []}),
            [AttrSplat, GetAttr(Identifier::unchecked("foo"))],
        ),
        Value::from_iter([vec![1, 2, 3]]),
    );

    // attr-splat array
    eval_to(
        Traversal::new(
            expression! {
                [
                    { "foo" = 2 },
                    { "foo" = 1, "bar" = 2 }
                ]
            },
            [AttrSplat, GetAttr(Identifier::unchecked("foo"))],
        ),
        Value::from_iter([2, 1]),
    );

    // attr-splat null
    eval_to(
        Traversal::new(
            Expression::Null,
            [AttrSplat, GetAttr(Identifier::unchecked("foo"))],
        ),
        Value::Array(vec![]),
    );

    // attr-splat followed by non-get-attr
    eval_to(
        Traversal::new(
            expression! {
                [
                    { "foo" = { "bar" = [1, 2, 3] } },
                    { "foo" = { "bar" = [10, 20, 30] } }
                ]
            },
            [
                AttrSplat,
                GetAttr(Identifier::unchecked("foo")),
                GetAttr(Identifier::unchecked("bar")),
                Index(expression!(1)),
            ],
        ),
        Value::from_iter([10, 20, 30]),
    );

    // full-splat followed by non-get-attr
    eval_to(
        Traversal::new(
            expression! {
                [
                    { "foo" = { "bar" = [1, 2, 3] } },
                    { "foo" = { "bar" = [10, 20, 30] } }
                ]
            },
            [
                FullSplat,
                GetAttr(Identifier::unchecked("foo")),
                GetAttr(Identifier::unchecked("bar")),
                Index(expression!(1)),
            ],
        ),
        Value::from_iter([2, 20]),
    );

    // errors
    eval_error(
        Traversal::new(vec![1, 2, 3], [LegacyIndex(5)]),
        ErrorKind::Index(5),
    );
}

#[test]
fn eval_func_call() {
    fn add(args: FuncArgs) -> EvalResult<Value, String> {
        let a = args[0].as_number().unwrap();
        let b = args[1].as_number().unwrap();
        Ok(Value::Number(*a + *b))
    }

    fn strlen(args: FuncArgs) -> EvalResult<Value, String> {
        Ok(Value::from(args[0].as_str().unwrap().len()))
    }

    let mut ctx = Context::new();
    ctx.declare_func(
        "add",
        FuncDef::builder()
            .params([ParamType::Number, ParamType::Number])
            .build(add),
    );
    ctx.declare_func(
        "strlen",
        FuncDef::builder().param(ParamType::String).build(strlen),
    );

    eval_to_ctx(
        &ctx,
        FuncCall::builder("add")
            .arg(FuncCall::builder("strlen").arg("foo").build())
            .arg(2)
            .build(),
        Value::from(5),
    )
}

#[test]
fn eval_template() {
    let mut ctx = Context::new();
    ctx.declare_var("name", "World");

    eval_to_ctx(
        &ctx,
        Template::from_str("Hello, ${name ~} !").unwrap(),
        String::from("Hello, World!"),
    );

    let template_str = r#"Let's ${~ what ~} :
%{ for item in items ~}
- ${item}
%{~ endfor ~}

"#;

    let expected = r#"Let's render a list:
- foo
- bar
- baz"#;

    let mut ctx = Context::new();
    ctx.declare_var("what", " render a list");
    ctx.declare_var("items", vec!["foo", "bar", "baz"]);

    eval_to_ctx(
        &ctx,
        Template::from_str(template_str).unwrap(),
        String::from(expected),
    );
}

#[test]
fn expr_error_context() {
    let input = r#"
        block {
            attr = cond ? "yes" : "no"
        }
    "#;

    // The `cond` variable is not defined which should forcefully fail the evaluation.
    let ctx = Context::new();

    let err = from_str::<Body>(input, &ctx).unwrap_err();

    assert_eq!(
        err.to_string(),
        r#"eval error: undefined variable `cond` in expression `cond ? "yes" : "no"`"#,
    )
}

#[test]
fn param_type() {
    let string = Value::from("a string");
    let number = Value::from(42);
    let boolean = Value::from(true);
    let string_array = Value::from_iter(["foo", "bar"]);
    let number_array = Value::from_iter([1, 2, 3]);
    let object_of_strings = Value::from_iter([("foo", "bar"), ("baz", "qux")]);
    let object_of_numbers = Value::from_iter([("foo", 1), ("bar", 2)]);

    let param = ParamType::String;
    assert!(param.is_satisfied_by(&string));
    assert!(!param.is_satisfied_by(&number));

    let param = ParamType::Any;
    assert!(param.is_satisfied_by(&string));
    assert!(param.is_satisfied_by(&number));

    let param = ParamType::nullable(ParamType::String);
    assert!(param.is_satisfied_by(&string));
    assert!(param.is_satisfied_by(&Value::Null));
    assert!(!param.is_satisfied_by(&number));

    let param = ParamType::one_of([ParamType::String, ParamType::Number]);
    assert!(param.is_satisfied_by(&string));
    assert!(param.is_satisfied_by(&number));
    assert!(!param.is_satisfied_by(&boolean));

    let param = ParamType::array_of(ParamType::String);
    assert!(param.is_satisfied_by(&string_array));
    assert!(!param.is_satisfied_by(&number_array));

    let param = ParamType::object_of(ParamType::String);
    assert!(param.is_satisfied_by(&object_of_strings));
    assert!(!param.is_satisfied_by(&object_of_numbers));
}

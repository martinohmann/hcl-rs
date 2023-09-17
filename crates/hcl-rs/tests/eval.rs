mod common;

use common::{assert_eval, assert_eval_ctx, assert_eval_error};
use hcl::eval::{Context, ErrorKind, EvalResult, Evaluate, FuncArgs, FuncDef, ParamType};
use hcl::expr::{
    BinaryOp, BinaryOperator, Conditional, Expression, ForExpr, FuncCall, TemplateExpr, Traversal,
    TraversalOperator, Variable,
};
use hcl::structure::Body;
use hcl::template::Template;
use hcl::{Attribute, Block, Identifier, Number, Value};
use indoc::indoc;

#[test]
fn eval_binary_op() {
    use BinaryOperator::*;

    assert_eval(
        BinaryOp::new(
            BinaryOp::new(1, Div, 2),
            Mul,
            BinaryOp::new(3, Plus, BinaryOp::new(4, Div, 5)),
        ),
        Value::from(2.3),
    );
    assert_eval(BinaryOp::new("foo", Eq, "foo"), Value::from(true));
    assert_eval(BinaryOp::new(false, Or, true), Value::from(true));
    assert_eval(BinaryOp::new(true, And, true), Value::from(true));
    assert_eval(BinaryOp::new(true, And, false), Value::from(false));
    assert_eval(BinaryOp::new(1, Less, 2), Value::from(true));
    assert_eval(
        BinaryOp::new(
            BinaryOp::new(1, Greater, 0),
            And,
            BinaryOp::new("foo", NotEq, Expression::Null),
        ),
        Value::from(true),
    );
}

#[test]
fn eval_conditional() {
    assert_eval(Conditional::new(true, "yes", "no"), Value::from("yes"));
    assert_eval(Conditional::new(false, "yes", "no"), Value::from("no"));
    assert_eval_error(
        Conditional::new("foo", "yes", "no"),
        ErrorKind::Unexpected(Value::from("foo"), "a boolean"),
    );
}

#[test]
fn eval_for_expr() {
    assert_eval(
        ForExpr::new(
            Identifier::unchecked("item"),
            Expression::from_iter([1, 2, 3, 4, 5, 6, 7]),
            BinaryOp::new(Variable::unchecked("item"), BinaryOperator::Mul, 2),
        )
        .with_cond_expr(BinaryOp::new(
            Variable::unchecked("item"),
            BinaryOperator::Less,
            5,
        )),
        Value::from_iter([2, 4, 6, 8]),
    );

    assert_eval(
        ForExpr::new(
            Identifier::unchecked("value"),
            Expression::from_iter([("a", "1"), ("b", "2"), ("c", "3"), ("d", "4")]),
            Variable::unchecked("key"),
        )
        .with_key_var(Identifier::unchecked("key"))
        .with_key_expr(Variable::unchecked("value"))
        .with_cond_expr(BinaryOp::new(
            Variable::unchecked("key"),
            BinaryOperator::NotEq,
            Expression::from("d"),
        )),
        Value::from_iter([("1", "a"), ("2", "b"), ("3", "c")]),
    );

    assert_eval(
        ForExpr::new(
            Identifier::unchecked("value"),
            Expression::from_iter(["a", "b", "c", "d"]),
            Variable::unchecked("value"),
        )
        .with_key_var(Identifier::unchecked("index"))
        .with_key_expr(TemplateExpr::QuotedString("${index}".into())),
        Value::from_iter([("0", "a"), ("1", "b"), ("2", "c"), ("3", "d")]),
    );

    assert_eval(
        ForExpr::new(
            Identifier::unchecked("value"),
            Expression::from_iter([("a", "1"), ("b", "2"), ("c", "3"), ("d", "4")]),
            Variable::unchecked("key"),
        )
        .with_key_var(Identifier::unchecked("key")),
        Value::from_iter(["a", "b", "c", "d"]),
    );

    assert_eval(
        ForExpr::new(
            Identifier::unchecked("value"),
            Expression::from_iter(["a", "b", "c", "d"]),
            Variable::unchecked("value"),
        )
        .with_key_var(Identifier::unchecked("index"))
        .with_key_expr(TemplateExpr::from("${index}")),
        Value::from_iter([("0", "a"), ("1", "b"), ("2", "c"), ("3", "d")]),
    );

    assert_eval(
        ForExpr::new(
            Identifier::unchecked("value"),
            Expression::from_iter([("a", 1), ("b", 2), ("c", 3), ("d", 4)]),
            Variable::unchecked("value"),
        )
        .with_key_var(Identifier::unchecked("key"))
        .with_key_expr(Expression::from("foo"))
        .with_cond_expr(BinaryOp::new(
            Variable::unchecked("key"),
            BinaryOperator::NotEq,
            Expression::from("d"),
        ))
        .with_grouping(true),
        Value::from_iter([("foo", vec![1, 2, 3])]),
    );

    assert_eval_error(
        ForExpr::new(
            Identifier::unchecked("v"),
            Expression::from_iter(["a"]),
            Expression::Bool(true),
        )
        .with_key_expr(Expression::Null),
        ErrorKind::Unexpected(Value::Null, "a string, boolean or number"),
    );
}

#[test]
fn eval_traversal() {
    use TraversalOperator::*;

    // legacy index access: expr.2
    assert_eval(
        Traversal::new(vec![1, 2, 3], [LegacyIndex(1)]),
        Value::from(2),
    );

    // legacy index access: expr[2]
    assert_eval(
        Traversal::new(vec![1, 2, 3], [Index(Expression::from(2))]),
        Value::from(3),
    );

    // get-attr: expr.foo
    assert_eval(
        Traversal::new(
            hcl::expression!({"foo" = [1, 2, 3], "bar" = []}),
            [GetAttr(Identifier::unchecked("foo"))],
        ),
        Value::from_iter([1, 2, 3]),
    );

    // chain get-attr -> index: expr.foo[2]
    assert_eval(
        Traversal::new(
            Traversal::new(
                hcl::expression!({"foo" = [1, 2, 3], "bar" = []}),
                [GetAttr(Identifier::unchecked("foo"))],
            ),
            [Index(Expression::from(2))],
        ),
        Value::from(3),
    );

    // full-splat non-array
    assert_eval(
        Traversal::new(
            hcl::expression!({"foo" = [1, 2, 3], "bar" = []}),
            [FullSplat, GetAttr(Identifier::unchecked("foo"))],
        ),
        Value::from_iter([vec![1, 2, 3]]),
    );

    // full-splat array
    assert_eval(
        Traversal::new(
            hcl::expression! {
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
    assert_eval(
        Traversal::new(
            Expression::Null,
            [FullSplat, GetAttr(Identifier::unchecked("foo"))],
        ),
        Value::Array(vec![]),
    );

    // attr-splat non-array
    assert_eval(
        Traversal::new(
            hcl::expression!({"foo" = [1, 2, 3], "bar" = []}),
            [AttrSplat, GetAttr(Identifier::unchecked("foo"))],
        ),
        Value::from_iter([vec![1, 2, 3]]),
    );

    // attr-splat array
    assert_eval(
        Traversal::new(
            hcl::expression! {
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
    assert_eval(
        Traversal::new(
            Expression::Null,
            [AttrSplat, GetAttr(Identifier::unchecked("foo"))],
        ),
        Value::Array(vec![]),
    );

    // attr-splat followed by non-get-attr
    assert_eval(
        Traversal::new(
            hcl::expression! {
                [
                    { "foo" = { "bar" = [1, 2, 3] } },
                    { "foo" = { "bar" = [10, 20, 30] } }
                ]
            },
            [
                AttrSplat,
                GetAttr(Identifier::unchecked("foo")),
                GetAttr(Identifier::unchecked("bar")),
                Index(hcl::expression!(1)),
            ],
        ),
        Value::from_iter([10, 20, 30]),
    );

    // full-splat followed by non-get-attr
    assert_eval(
        Traversal::new(
            hcl::expression! {
                [
                    { "foo" = { "bar" = [1, 2, 3] } },
                    { "foo" = { "bar" = [10, 20, 30] } }
                ]
            },
            [
                FullSplat,
                GetAttr(Identifier::unchecked("foo")),
                GetAttr(Identifier::unchecked("bar")),
                Index(hcl::expression!(1)),
            ],
        ),
        Value::from_iter([2, 20]),
    );

    // errors
    assert_eval_error(
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

    assert_eval_ctx(
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
    use std::str::FromStr;

    let mut ctx = Context::new();
    ctx.declare_var("name", "World");

    assert_eval_ctx(
        &ctx,
        Template::from_str("Hello, ${name ~} !").unwrap(),
        String::from("Hello, World!"),
    );

    let template_str = indoc! {r#"
        Let's ${~ what ~} :
        %{ for item in items ~}
        - ${item}

        %{~ endfor ~}

        Yay!

    "#};

    let expected = indoc! {r#"
        Let's render a list:
        - foo
        - bar
        - baz

        Yay!

    "#};

    let mut ctx = Context::new();
    ctx.declare_var("what", " render a list");
    ctx.declare_var("items", vec!["foo", "bar", "baz"]);

    assert_eval_ctx(
        &ctx,
        Template::from_str(template_str).unwrap(),
        expected.to_owned(),
    );
}

#[test]
fn expr_error_context() {
    let input = indoc! {r#"
        block {
            attr = cond ? "yes" : "no"
        }
    "#};

    // The `cond` variable is not defined which should forcefully fail the evaluation.
    let ctx = Context::new();

    let err = hcl::eval::from_str::<Body>(input, &ctx).unwrap_err();

    assert_eq!(
        err.to_string(),
        r#"eval error: undefined variable `cond` in expression `cond ? "yes" : "no"`"#,
    )
}

#[test]
fn eval_in_place() {
    let mut ctx = Context::new();

    let mut body = Body::builder()
        .add_block(
            Block::builder("foo")
                .add_attribute(Attribute::new("bar", FuncCall::new("baz")))
                .add_attribute(Attribute::new(
                    "qux",
                    BinaryOp::new(1, BinaryOperator::Plus, 1),
                ))
                .build(),
        )
        .add_block(
            Block::builder("bar")
                .add_attribute(Attribute::new("baz", Variable::unchecked("qux")))
                .build(),
        )
        .build();

    assert_eq!(body.evaluate_in_place(&ctx).unwrap_err().len(), 2);

    let expected = Body::builder()
        .add_block(
            Block::builder("foo")
                .add_attribute(Attribute::new("bar", FuncCall::new("baz")))
                .add_attribute(Attribute::new("qux", 2))
                .build(),
        )
        .add_block(
            Block::builder("bar")
                .add_attribute(Attribute::new("baz", Variable::unchecked("qux")))
                .build(),
        )
        .build();

    assert_eq!(body, expected);

    ctx.declare_var("qux", "quxval");
    ctx.declare_func("baz", FuncDef::builder().build(|_| Ok(Value::from("baz"))));

    assert!(body.evaluate_in_place(&ctx).is_ok());

    let expected = Body::builder()
        .add_block(
            Block::builder("foo")
                .add_attribute(Attribute::new("bar", "baz"))
                .add_attribute(Attribute::new("qux", 2))
                .build(),
        )
        .add_block(
            Block::builder("bar")
                .add_attribute(Attribute::new("baz", "quxval"))
                .build(),
        )
        .build();

    assert_eq!(body, expected);
}

#[test]
fn eval_in_place_error() {
    let mut body = Body::builder()
        .add_attribute((
            "foo",
            BinaryOp::new(1, BinaryOperator::Plus, Variable::unchecked("bar")),
        ))
        .add_attribute((
            "bar",
            Conditional::new(
                BinaryOp::new(1, BinaryOperator::Less, 2),
                FuncCall::builder("true_action").arg(1).build(),
                FuncCall::new("false_action"),
            ),
        ))
        .build();

    let ctx = Context::new();
    let err = body.evaluate_in_place(&ctx).unwrap_err();

    assert_eq!(
        err.to_string(),
        indoc! {r#"
            2 errors occurred:
            - undefined variable `bar` in expression `1 + bar`
            - undefined function `true_action` in expression `1 < 2 ? true_action(1) : false_action()`
            "#
        }
    )
}

#[test]
fn interpolation_unwrapping() {
    // unwrapping
    assert_eval(TemplateExpr::from("${null}"), Value::Null);
    assert_eval(TemplateExpr::from("${\"foo\"}"), Value::from("foo"));
    assert_eval(TemplateExpr::from("${true}"), Value::Bool(true));
    assert_eval(TemplateExpr::from("${\"${true}\"}"), Value::Bool(true));
    assert_eval(
        TemplateExpr::from("${42}"),
        Value::Number(Number::from(42u64)),
    );
    assert_eval(
        TemplateExpr::from("${1.5}"),
        Value::Number(Number::from_f64(1.5).unwrap()),
    );
    assert_eval(
        TemplateExpr::from("${[1, 2, 3]}"),
        Value::from_iter([1, 2, 3]),
    );
    assert_eval(
        TemplateExpr::from("${{ a = 1, b = 2 }}"),
        Value::from_iter([("a", 1), ("b", 2)]),
    );

    let mut ctx = Context::new();
    ctx.declare_var("var", true);
    assert_eval_ctx(&ctx, TemplateExpr::from("${\"${var}\"}"), Value::Bool(true));

    // no unwrapping
    assert_eval(
        TemplateExpr::from("hello ${true}"),
        Value::from("hello true"),
    );
    assert_eval(TemplateExpr::from("${\"\"}${true}"), Value::from("true"));
    assert_eval(
        TemplateExpr::from("%{ for v in [true] }${v}%{ endfor }"),
        Value::from("true"),
    );
}

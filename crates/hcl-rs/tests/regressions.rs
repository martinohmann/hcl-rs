mod common;

use common::{assert_deserialize, assert_format};
use hcl::{expr::*, Identifier};
use indoc::indoc;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

// https://github.com/martinohmann/hcl-rs/issues/44
#[test]
fn issue_44() {
    #[derive(Deserialize, Debug, PartialEq)]
    pub struct Config {
        #[serde(rename = "project")]
        pub projects: HashMap<String, Project>,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    pub struct Project {
        pub proj_type: String,
        pub spec: Option<PathBuf>,
        pub dockerfile: Option<PathBuf>,
        pub scripts: Option<Vec<Script>>,
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    pub struct Script {
        pub name: String,
        pub command: String,
    }

    let expected = Config {
        projects: {
            let mut map = HashMap::new();
            map.insert(
                "a".into(),
                Project {
                    proj_type: "generic".into(),
                    spec: Some(PathBuf::from("./test.spec")),
                    dockerfile: None,
                    scripts: None,
                },
            );
            map
        },
    };

    let input = indoc! {r#"
        project "a" {
            proj_type = "generic"
            spec = "./test.spec"
        }
    "#};

    assert_deserialize(input, expected);
}

// https://github.com/martinohmann/hcl-rs/issues/66
#[test]
fn issue_66() {
    let expected =
        hcl::body!({ a = (Traversal::new(Variable::unchecked("b"), [Expression::from("c")])) });

    assert_deserialize(r#"a = b["c"]"#, expected);
}

// https://github.com/martinohmann/hcl-rs/issues/81
#[test]
fn issue_81() {
    let var = Variable::unchecked("module");

    let expected = hcl::body!({
        attr_splat = (Traversal::builder(var.clone()).attr("instance").attr_splat().attr("id").build())
        full_splat = (Traversal::builder(var).attr("instance").full_splat().attr("id").build())
    });

    let input = indoc! {r#"
        attr_splat = module.instance.*.id
        full_splat = module.instance[*].id
    "#};

    assert_deserialize(input, expected);
}

// https://github.com/martinohmann/hcl-rs/issues/83
#[test]
fn issue_83() {
    let expected = hcl::body!({
        attr = (Traversal::new(
            Variable::unchecked("module"),
            [
                TraversalOperator::GetAttr("instance".into()),
                TraversalOperator::LegacyIndex(0),
                TraversalOperator::GetAttr("id".into()),
            ],
        ))
    });

    assert_deserialize("attr = module.instance.0.id", expected);
}

// https://github.com/martinohmann/hcl-rs/issues/137
#[test]
fn issue_137() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Config {
        expr: Expression,
        bin_op: BinaryOp,
        op: Operation,
        cond: Conditional,
        cond_string: String,
        nested: Nested,
    }

    #[derive(Deserialize, Debug, PartialEq)]
    struct Nested {
        template_expr: TemplateExpr,
        heredoc: Heredoc,
    }

    let bin_op = BinaryOp::new(1, BinaryOperator::Plus, 1);

    let expected = Config {
        expr: Expression::from(bin_op.clone()),
        bin_op: bin_op.clone(),
        op: Operation::Binary(bin_op),
        cond: Conditional::new(Variable::unchecked("var"), true, false),
        cond_string: String::from("${var ? true : false}"),
        nested: Nested {
            template_expr: TemplateExpr::from("${some_var}"),
            heredoc: Heredoc::new(Identifier::unchecked("EOS"), "${some_var}\n")
                .with_strip_mode(HeredocStripMode::Indent),
        },
    };

    let input = indoc! {r#"
        expr = 1 + 1
        bin_op = 1 + 1
        op = 1 + 1
        cond = var ? true : false
        cond_string = var ? true : false

        nested {
          template_expr = "${some_var}"
          heredoc = <<-EOS
            ${some_var}
          EOS
        }
    "#};

    assert_deserialize(input, expected);
}

// https://github.com/martinohmann/hcl-rs/issues/87
#[test]
fn issue_87() {
    let expr = Expression::from(
        FuncCall::builder("foo")
            .arg(Expression::from_iter([("bar", FuncCall::new("baz"))]))
            .build(),
    );

    assert_format(
        expr,
        indoc! {r#"
            foo({ "bar" = baz() })
        "#}
        .trim_end(),
    );
}

// https://github.com/martinohmann/hcl-rs/issues/91
#[test]
fn issue_91() {
    assert_format(
        hcl::attribute!(_foo = "bar"),
        indoc! {r#"
            _foo = "bar"
        "#},
    );
}

// https://github.com/martinohmann/hcl-rs/issues/131
#[test]
fn issue_131() {
    assert_format(
        hcl::attribute!(a = (TemplateExpr::from(r#"${"b"}"#))),
        indoc! {r#"
            a = "${"b"}"
        "#},
    );

    assert_format(
        hcl::value!({ a = r#"${"b"}"# }),
        indoc! {r#"
            {
              "a" = "${"b"}"
            }
        "#}
        .trim_end(),
    );
}

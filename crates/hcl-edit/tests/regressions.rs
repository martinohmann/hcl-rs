use hcl_edit::expr::Expression;
use hcl_edit::structure::{Attribute, Block, Body, Structure};
use hcl_edit::template::{Element, Interpolation, Template};
use hcl_edit::{Ident, Span};
use pretty_assertions::assert_eq;

// https://github.com/martinohmann/hcl-rs/issues/248
#[test]
fn issue_248() {
    let expr = Expression::from("${foo}");

    let encoded = expr.to_string();
    assert_eq!(encoded, "\"$${foo}\"");

    let parsed: Expression = encoded.parse().unwrap();
    assert_eq!(parsed, expr);
}

// https://github.com/martinohmann/hcl-rs/issues/256
#[test]
fn issue_256() {
    let input = "$${escaped1} ${unescaped} $${escaped2} $$ESCAPED_SHELL_VAR\n$SHELL_VAR";
    let parsed: Template = input.parse().unwrap();
    let expected = Template::from_iter([
        Element::from("${escaped1} "),
        Element::from(Interpolation::new(Ident::new("unescaped"))),
        Element::from(" ${escaped2} $$ESCAPED_SHELL_VAR\n$SHELL_VAR"),
    ]);

    assert_eq!(parsed, expected);
}

// https://github.com/martinohmann/hcl-rs/issues/270
#[test]
fn issue_270() {
    let no_trailing_newline = String::from("block {\nfoo = \"bar\"\n}\nbar = \"baz\"");
    let trailing_newline = format!("{no_trailing_newline}\n");

    // Parsed
    let parsed: Body = no_trailing_newline.parse().unwrap();
    assert_eq!(parsed.to_string(), no_trailing_newline);

    let parsed: Body = trailing_newline.parse().unwrap();
    assert_eq!(parsed.to_string(), trailing_newline);

    // Manually constructed
    let mut body = Body::builder()
        .block(
            Block::builder(Ident::new("block"))
                .attribute(Attribute::new(Ident::new("foo"), "bar"))
                .build(),
        )
        .attribute(Attribute::new(Ident::new("bar"), "baz"))
        .build();
    assert_eq!(body.to_string(), trailing_newline);

    body.set_prefer_omit_trailing_newline(true);
    assert_eq!(body.to_string(), no_trailing_newline);
}

// https://github.com/martinohmann/hcl-rs/issues/284
#[test]
fn issue_284() {
    let input = r#"
      locals {
        test = {
          a = b// this comment breaks the parser
          c = d // but this one doesn't
        }
      }
    "#;

    let res: Result<Body, _> = input.parse();
    assert!(res.is_ok());
}

// https://github.com/martinohmann/hcl-rs/issues/294
#[test]
fn issue_294() {
    let input = r#"
        foo = bar
        block {}
        labeled_block "label" {}
    "#;

    let body: Body = input.parse().unwrap();

    assert_eq!(body.len(), 3);

    for structure in &body {
        let ident = match structure {
            Structure::Attribute(attr) => &attr.key,
            Structure::Block(block) => &block.ident,
        };

        assert!(
            ident.span().is_some(),
            "ident `{ident}` misses span information in {structure:?}"
        );
    }
}

// https://github.com/martinohmann/hcl-rs/issues/319
#[test]
fn issue_319() {
    macro_rules! assert_ok {
        ($input:expr) => {
            assert!($input.parse::<Body>().is_ok());
        };
    }

    macro_rules! assert_err {
        ($input:expr) => {
            assert!($input.parse::<Body>().is_err());
        };
    }

    // single line expressions with parenthesis
    assert_ok! {r#"
        foo = (true ? "bar" : "baz")
    "#};
    assert_ok! {r#"
        foo = (1 > 2)
    "#};
    assert_ok! {r#"
        foo = (var.foo[2])
    "#};

    // multiline expressions with parenthesis
    assert_ok! {r#"
        foo = (true ?
            "bar" :
            "baz"
        )
    "#};
    assert_ok! {r#"
        foo = (
            1
            >
            2
        )
    "#};
    assert_ok! {r#"
        foo = (
            var
                .foo
                [2]
        )
    "#};

    // invalid multiline expressions without parenthesis
    assert_err! {r#"
        foo = true ?
            "bar" :
            "baz"
    "#};
    assert_err! {r#"
        foo = 1
            >
            2
    "#};
    assert_err! {r#"
        foo = var
            .foo
            [2]
    "#};
}

// https://github.com/martinohmann/hcl-rs/issues/350
#[test]
fn issue_350() {
    let unicode_input = r#"
        locals {
            é = 4
        }
        output "ééé" {
            value = local.é
        }
    "#;
    assert!(unicode_input.parse::<Body>().is_ok());
}

// https://github.com/martinohmann/hcl-rs/issues/367
#[test]
fn issue_367() {
    macro_rules! assert_ok {
        ($input:expr) => {
            assert!($input.parse::<Body>().is_ok());
        };
    }

    macro_rules! assert_err {
        ($input:expr) => {
            assert!($input.parse::<Body>().is_err());
        };
    }

    // multiline expressions with function calls
    assert_ok! {r#"
        foo = length(
            true
            ? "bar"
            : "baz"
        )
    "#};
    assert_ok! {r#"
        foo = length(
            1
            >
            2
        )
    "#};
    assert_ok! {r#"
        foo = length(
            var
                .foo
                [2]
        )
    "#};

    // multiline expressions with arrays
    assert_ok! {r#"
        foo = [
            true
            ? "bar"
            : "baz"
        ]
    "#};
    assert_ok! {r#"
        foo = [
            1
            >
            2
        ]
    "#};
    assert_ok! {r#"
        foo = [
            var.foo
            || "yes"
        ]
    "#};
    assert_ok! {r#"
        foo = [
            var
                .foo
                [2]
        ]
    "#};
    assert_ok! {r#"
        foo = [
            for a in range(10) :
            var.foo
            || "yes"
        ]
    "#};
    assert_ok! {r#"
        foo = [
            for a in range(10) :
            var.foo
            if a > 5
            || "yes"
        ]
    "#};
    assert_ok! {r#"
        beep = {
            for num in range(1) :
            num => splatme...
        }
    "#};

    // unsupported multiline expressions with objects
    assert_err! {r#"
        beep = {
            a = true
            ? "bar"
            : "baz"
        }
    "#};
    assert_err! {r#"
        beep = {
            a = 1
            >
            2
        }
    "#};

    // multiline expressions for expressions
    assert_ok! {r#"
        beep = {
            for
            num in [1] :
            num => num
        }
    "#};

    // spread operator with nontrivial expressions
    assert_ok! {r#"
        origins = {
            for behavior in var.behaviors :
            behavior.origin => behavior.path...
        }
    "#};
    assert_ok! {r#"
        origins = {
            for behavior in var.behaviors :
            behavior.origin => behavior.path...
            if behavior.origin != null
            && !startswith(behavior.origin, "forbidden/")
        }
    "#};
}

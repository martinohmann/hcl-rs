use super::expr::expr;
use super::parse;
use crate::expr::{Expression, Variable};
use crate::structure::{Block, Body};
use indoc::indoc;
use pretty_assertions::assert_eq;

#[test]
fn test_parse_string() {
    let data = "\"abc\"";
    let result = expr::<()>(data);
    assert_eq!(result, Ok(("", Expression::from("abc"))));

    let data = "\"tab:\\tafter tab, newline:\\nnew line, quote: \\\", emoji: \\u{1F602}, newline:\\nescaped whitespace: \\    abc\"";
    let result = expr::<()>(data);
    assert_eq!(
    result,
    Ok((
      "",
      Expression::from("tab:\tafter tab, newline:\nnew line, quote: \", emoji: \u{1F602}, newline:\nescaped whitespace: abc")
    ))
  );
}

#[test]
fn test_parse_number() {
    let result = expr::<()>("1.1");
    assert_eq!(result, Ok(("", Expression::from(1.1))));
    let result = expr::<()>("1");
    assert_eq!(result, Ok(("", Expression::from(1u64))));
    // let result = expr::<()>("-1");
    // assert_eq!(result, Ok(("", Expression::from(-1i64))));
    assert_eq!(
        expr::<()>("NaN"),
        Ok(("", Expression::from(Variable::unchecked("NaN"))))
    );
}

#[test]
#[ignore]
fn test_parse_body() {
    let input = indoc! {r#"
        foo "label" {
            bar = "baz"
        }
    "#};

    let expected = Body::builder()
        .add_block(
            Block::builder("foo")
                .add_label("label")
                .add_attribute(("bar", "baz"))
                .build(),
        )
        .build();

    assert_eq!(parse(input).unwrap(), expected);

    let input = r#"value = {"Struct"=1}"#;

    assert_eq!(parse(input).unwrap(), Body::default());
}

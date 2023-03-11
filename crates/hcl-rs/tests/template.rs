use hcl::expr::{TemplateExpr, Variable};
use hcl::template::{IfDirective, Strip, Template};
use pretty_assertions::assert_eq;

#[test]
fn from_expr() {
    let expr = TemplateExpr::from("bar ${baz} %{~ if cond}qux%{ endif ~}");

    let expected = Template::new()
        .add_literal("bar ")
        .add_interpolation(Variable::unchecked("baz"))
        .add_literal(" ")
        .add_directive(
            IfDirective::new(
                Variable::unchecked("cond"),
                Template::new().add_literal("qux"),
            )
            .with_if_strip(Strip::Start)
            .with_endif_strip(Strip::End),
        );

    assert_eq!(Template::from_expr(&expr).unwrap(), expected);
}

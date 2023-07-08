//! This example demonstrates interpolation unwrapping as described in the HCL spec:
//!
//! https://github.com/hashicorp/hcl/blob/main/hclsyntax/spec.md#template-interpolation-unwrapping
//!
//! Templates containing only a single interpolation element can be unwrapped into the expression
//! contained in the interpolation.
//!
//! # Usage
//!
//! ```shell
//! cargo run --example interpolation-unwrapping file.hcl
//! ```
//!
//! # Example
//!
//! In the following attribute, the right-hand-side template interpolation can be unwrapped:
//!
//! ```hcl
//! foo = "${var.bar}"
//! ```
//!
//! After interpolation unwrapping the result will look like this:
//!
//! ```hcl
//! foo = var.bar
//! ```
//!
//! But the following cannot be unwrapped since it consists of two interpolations separated by a
//! literal string (`/`):
//!
//! ```hcl
//! foo = "${var.bar}/${var.baz}"
//! ```
use hcl_edit::expr::Expression;
use hcl_edit::prelude::*;
use hcl_edit::structure::Body;
use hcl_edit::template::{Element, StringTemplate};
use hcl_edit::visit_mut::{visit_expr_mut, VisitMut};

struct InterpolationUnwrapper;

impl VisitMut for InterpolationUnwrapper {
    fn visit_expr_mut(&mut self, expr: &mut Expression) {
        // Only templates containing a single interpolation can be unwrapped.
        if let Some(interpolation) = expr
            .as_string_template()
            .and_then(StringTemplate::as_single_element)
            .and_then(Element::as_interpolation)
        {
            let mut unwrapped_expr = interpolation.expr.clone();

            // Apply the existing decor to the unwrapped expression.
            std::mem::swap(expr.decor_mut(), unwrapped_expr.decor_mut());
            *expr = unwrapped_expr;
        } else {
            // Recurse further down the AST.
            visit_expr_mut(self, expr);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(filename) = std::env::args().into_iter().skip(1).next() else {
        eprintln!("filename argument required");
        std::process::exit(1);
    };

    let input = std::fs::read_to_string(filename)?;
    let mut body: Body = input.parse()?;

    let mut visitor = InterpolationUnwrapper;
    visitor.visit_body_mut(&mut body);

    println!("{body}");

    Ok(())
}

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn interpolations_are_unwrapped() {
    let input = indoc::indoc! {r#"
        // Subscribe the SQS queue to my SNS topic.
        resource "aws_sns_topic_subscription" "my_subscription" {
          topic_arn = "${aws_sns_topic.my_topic.arn}" // This comment will be preserved
          protocol = "sqs"
          endpoint = "${aws_sqs_queue.my_queue.arn}"
        }
    "#};

    let mut body: Body = input.parse().unwrap();

    let mut visitor = InterpolationUnwrapper;
    visitor.visit_body_mut(&mut body);

    let expected = indoc::indoc! {r#"
        // Subscribe the SQS queue to my SNS topic.
        resource "aws_sns_topic_subscription" "my_subscription" {
          topic_arn = aws_sns_topic.my_topic.arn // This comment will be preserved
          protocol = "sqs"
          endpoint = aws_sqs_queue.my_queue.arn
        }
    "#};

    assert_eq!(body.to_string(), expected);
}

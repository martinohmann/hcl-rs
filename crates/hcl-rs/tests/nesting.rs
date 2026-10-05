//! Deeply nested input must result in errors instead of stack overflows.
//!
//! The nesting limits themselves are tested in `hcl-edit`. These tests make sure that the deepest
//! accepted inputs can be handled by `hcl-rs` and that parse errors for deeper inputs are passed
//! through.

use hcl::eval::{Context, Evaluate};
use hcl::{Body, Value};
use testdata::nesting::{binary_inputs, nested_inputs, with_small_stack};

/// Finds the deepest input of `inputs` named `name` that is accepted and makes sure that
/// converting, formatting, evaluating, deserializing and dropping it does not overflow the stack.
///
/// The next deeper input must be rejected with the parse error reported by `hcl-edit`.
fn assert_deepest_accepted(name: &str, inputs: fn(usize) -> Vec<(&'static str, String)>) {
    let input = |depth| {
        inputs(depth)
            .into_iter()
            .find(|(n, _)| *n == name)
            .unwrap()
            .1
    };

    let mut depth = 0;

    loop {
        let input = input(depth + 1);

        match hcl::parse(&input) {
            Ok(body) => {
                let formatted = hcl::to_string(&body).unwrap();
                assert_eq!(hcl::parse(&formatted).unwrap(), body);
                let _ = body.evaluate(&Context::new());
                assert_eq!(hcl::from_str::<Body>(&input).unwrap(), body);
                let _ = hcl::from_str::<Value>(&input);
                drop(body.clone());
                depth += 1;
            }
            Err(hcl::Error::Parse(err)) => {
                let expected = hcl::edit::parser::parse_body(&input).unwrap_err();
                assert_eq!(err, expected, "unexpected error for {name}");
                assert!(
                    matches!(hcl::from_str::<Body>(&input), Err(hcl::Error::Parse(e)) if e == expected),
                    "unexpected error for {name}"
                );
                assert!(depth > 0, "no input accepted for {name}");
                return;
            }
            Err(err) => panic!("unexpected error for {name}: {err}"),
        }
    }
}

#[test]
fn deepest_accepted_input() {
    with_small_stack(|| {
        for (name, _) in nested_inputs(0) {
            assert_deepest_accepted(name, nested_inputs);
        }

        for (name, _) in binary_inputs(0) {
            assert_deepest_accepted(name, binary_inputs);
        }
    });
}

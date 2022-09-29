# hcl-rs

[![Build Status](https://github.com/martinohmann/hcl-rs/workflows/ci/badge.svg)](https://github.com/martinohmann/hcl-rs/actions?query=workflow%3Aci)
[![crates.io](https://img.shields.io/crates/v/hcl-rs)](https://crates.io/crates/hcl-rs)
[![docs.rs](https://img.shields.io/docsrs/hcl-rs)](https://docs.rs/hcl-rs)
![MIT License](https://img.shields.io/github/license/martinohmann/hcl-rs?color=blue)

A rust library for interacting with the Hashicorp Configuration Language (HCL).

## Features

- A parser for the [HCL syntax
  specification](https://github.com/hashicorp/hcl/blob/main/hclsyntax/spec.md)
- Types for all HCL structures, e.g. body, blocks and attributes
- Supporting macros like `body!` for constructing HCL data structures
- Supports the
  [expression](https://github.com/hashicorp/hcl/blob/main/hclsyntax/spec.md#expressions)
  and
  [template](https://github.com/hashicorp/hcl/blob/main/hclsyntax/spec.md#templates)
  sub-languages in attribute values
- Support for deserializing and serializing arbitrary types that
  implement `serde::Deserialize` or `serde::Serialize`

## Planned Features

- Evaluation of HCL expressions and the template sub-language (currently in
  progress)

## Deserialization examples

Deserialize arbitrary HCL according to the [HCL JSON
Specification](https://github.com/hashicorp/hcl/blob/main/json/spec.md):

```rust
use serde_json::{json, Value};

let input = r#"
    some_attr = {
      foo = [1, 2]
      bar = true
    }

    some_block "some_block_label" {
      attr = "value"
    }
"#;

let expected = json!({
    "some_attr": {
        "foo": [1, 2],
        "bar": true
    },
    "some_block": {
        "some_block_label": {
            "attr": "value"
        }
    }
});

let value: Value = hcl::from_str(input).unwrap();

assert_eq!(value, expected);
```

If you need to preserve context about the HCL structure, deserialize into
`hcl::Body` instead:

```rust
use hcl::{Block, Body, Expression};

let input = r#"
    some_attr = {
      "foo" = [1, 2]
      "bar" = true
    }

    some_block "some_block_label" {
      attr = "value"
    }
"#;

let expected = Body::builder()
    .add_attribute((
        "some_attr",
        Expression::from_iter([
            ("foo", Expression::from(vec![1, 2])),
            ("bar", Expression::Bool(true)),
        ]),
    ))
    .add_block(
        Block::builder("some_block")
            .add_label("some_block_label")
            .add_attribute(("attr", "value"))
            .build(),
    )
    .build();

let body: Body = hcl::from_str(input).unwrap();

assert_eq!(body, expected);
```

## Serialization examples

A simple example to serialize some terraform configuration:

```rust
use hcl::{Block, Body, RawExpression};

let body = Body::builder()
    .add_block(
        Block::builder("resource")
            .add_label("aws_sns_topic_subscription")
            .add_label("topic")
            .add_attribute(("topic_arn", RawExpression::new("aws_sns_topic.queue.arn")))
            .add_attribute(("protocol", "sqs"))
            .add_attribute(("endpoint", RawExpression::new("aws_sqs_queue.queue.arn")))
            .build(),
    )
    .build();

let expected = r#"
resource "aws_sns_topic_subscription" "topic" {
  topic_arn = aws_sns_topic.queue.arn
  protocol = "sqs"
  endpoint = aws_sqs_queue.queue.arn
}
"#.trim_start();

let serialized = hcl::to_string(&body).unwrap();

assert_eq!(serialized, expected);
```

Also have a look at the other examples provided in the [documentation of the
`ser` module](https://docs.rs/hcl-rs/latest/hcl/ser/index.html).

## Macros

This crate provides a couple of macros to ease building HCL data structures.
Have a look at [their
documentation](https://docs.rs/hcl-rs/latest/hcl/macro.body.html) for usage
examples.

## Contributing

Contributions are welcome! Please read [`CONTRIBUTING.md`](CONTRIBUTING.md)
before creating a PR.

## License

The source code of hcl-rs is released under the MIT License. See the bundled
LICENSE file for details.

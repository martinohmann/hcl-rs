# hcl-rs

[![Build Status](https://github.com/martinohmann/hcl-rs/workflows/ci/badge.svg)](https://github.com/martinohmann/hcl-rs/actions?query=workflow%3Aci)
[![docs.rs](https://img.shields.io/docsrs/hcl-rs)](https://docs.rs/hcl-rs)
![MIT License](https://img.shields.io/github/license/martinohmann/hcl-rs?color=blue)

This crate provides functionality to deserialize, serialize and manipulate HCL data.

The main types are `Deserializer` for deserializing data, `Serializer` for
serialization. Furthermore the provided `Body` and `Value` types can be used to
construct HCL data or as a deserialization target.

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

Please have a look at examples provided in the [documentation of the `ser` module](https://docs.rs/hcl-rs/latest/hcl/ser/index.html).

## License

The source code of hcl-rs is released under the MIT License. See the bundled
LICENSE file for details.

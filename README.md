# hcl 

[![Build Status](https://github.com/martinohmann/dts/workflows/ci/badge.svg)](https://github.com/martinohmann/dts/actions?query=workflow%3Aci)
![MIT License](https://img.shields.io/github/license/martinohmann/dts?color=blue)

This crate provides functionality to deserialize and manipulate HCL data.

The main types are `Deserializer` for deserializing data and `Value` which can
be used to deserialize arbitrary HCL data.

**Note**: Serializing to HCL is not supported.

## Example

```rust
let input = r#"
    some_attr = {
      foo = [1, 2]
      bar = true
    }

    some_block "some_block_label" {
      attr = "value"
    }
"#;

let v: hcl::Value = hcl::from_str(input).unwrap();
println!("{:#?}", v);
```

## License

The source code of hcl is released under the MIT License. See the bundled
LICENSE file for details.

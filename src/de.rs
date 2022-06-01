//! Deserialize HCL data to a Rust data structure.
//!
//! The `Deserializer` implementation tries to follow the [HCL JSON Specification][hcl-json-spec]
//! as close as possible.
//!
//! [hcl-json-spec]: https://github.com/hashicorp/hcl/blob/main/json/spec.md

use crate::{
    parser, structure::de::BodyDeserializer, value::de::ValueDeserializer, Body, Error, Result,
};
use serde::de;
use serde::forward_to_deserialize_any;

/// A structure that deserializes HCL into Rust values.
pub struct Deserializer {
    body: Body,
}

impl Deserializer {
    /// Creates a HCL deserializer from a `&str`.
    ///
    /// ## Errors
    ///
    /// An [`Error`][Error] is returned when the input is not valid HCL.
    ///
    /// [Error]: ../error/enum.Error.html
    pub fn from_str(input: &str) -> Result<Self> {
        let body = parser::parse(input)?;
        Ok(Deserializer { body })
    }
}

/// Deserialize an instance of type `T` from a string of HCL text.
///
/// By default, the deserialization will follow the [HCL JSON Specification][hcl-json-spec].
///
/// If preserving HCL semantics is required consider deserializing into a [`Body`][Body] instead or
/// use [`hcl::parse`][parse] to directly parse the input into a [`Body`][Body].
///
/// [hcl-json-spec]: https://github.com/hashicorp/hcl/blob/main/json/spec.md
/// [parse]: ../fn.parse.html
/// [Body]: ../struct.Body.html
///
/// ## Example
///
/// ```
/// use serde_json::{json, Value};
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let input = r#"
///     some_attr = {
///       foo = [1, 2]
///       bar = true
///     }
///
///     some_block "some_block_label" {
///       attr = "value"
///     }
/// "#;
///
/// let expected = json!({
///     "some_attr": {
///         "foo": [1, 2],
///         "bar": true
///     },
///     "some_block": {
///         "some_block_label": {
///             "attr": "value"
///         }
///     }
/// });
///
/// let value: Value = hcl::from_str(input)?;
///
/// assert_eq!(value, expected);
/// #   Ok(())
/// # }
/// ```
///
/// ## Errors
///
/// This functions fails with an error if the data does not match the structure of `T`.
pub fn from_str<'de, T>(s: &'de str) -> Result<T>
where
    T: de::Deserialize<'de>,
{
    let deserializer = Deserializer::from_str(s)?;
    T::deserialize(deserializer)
}

/// Deserialize an instance of type `T` from an IO stream of HCL.
///
/// See the documentation of [`from_str`][from_str] for more information.
///
/// ## Example
///
/// ```
/// use serde_json::{json, Value};
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let input = r#"
///     some_attr = {
///       foo = [1, 2]
///       bar = true
///     }
///
///     some_block "some_block_label" {
///       attr = "value"
///     }
/// "#;
///
/// let expected = json!({
///     "some_attr": {
///         "foo": [1, 2],
///         "bar": true
///     },
///     "some_block": {
///         "some_block_label": {
///             "attr": "value"
///         }
///     }
/// });
///
/// let value: Value = hcl::from_reader(input.as_bytes())?;
///
/// assert_eq!(value, expected);
/// #   Ok(())
/// # }
/// ```
///
/// ## Errors
///
/// This functions fails with an error if reading from the reader fails or if the data does not
/// match the structure of `T`.
pub fn from_reader<T, R>(mut reader: R) -> Result<T>
where
    T: de::DeserializeOwned,
    R: std::io::Read,
{
    let mut s = String::new();
    reader.read_to_string(&mut s)?;

    from_str(&s)
}

/// Deserialize an instance of type `T` from a byte slice.
///
/// See the documentation of [`from_str`][from_str] for more information.
///
/// ## Errors
///
/// This functions fails with an error if `buf` does not contain valid UTF-8 or if the data does
/// not match the structure of `T`.
pub fn from_slice<'de, T>(buf: &'de [u8]) -> Result<T>
where
    T: de::Deserialize<'de>,
{
    let s = std::str::from_utf8(buf)?;
    from_str(s)
}

impl<'de, 'a> de::Deserializer<'de> for Deserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let de = ValueDeserializer::new(self.body.into());
        de.deserialize_any(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if name == "$hcl::body" {
            // Specialized handling of `hcl::Body`.
            let de = BodyDeserializer::new(self.body);
            de.deserialize_any(visitor)
        } else {
            // Generic deserialization according to the HCL JSON spec.
            self.deserialize_any(visitor)
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let de = ValueDeserializer::new(self.body.into());
        de.deserialize_enum(name, variants, visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde::Deserialize;
    use serde_json::{json, Value};

    #[test]
    fn test_string_attribute() {
        let h = r#"foo = "bar""#;
        let expected: Value = json!({
            "foo": "bar"
        });
        assert_eq!(expected, from_str::<Value>(h).unwrap());
    }

    #[test]
    fn test_object() {
        let h = r#"foo = { bar = 42, "baz" = true }"#;
        let expected: Value = json!({
            "foo": {"bar": 42, "baz": true}
        });
        assert_eq!(expected, from_str::<Value>(h).unwrap());
    }

    #[test]
    fn test_block() {
        let h = r#"resource "aws_s3_bucket" "mybucket" { name = "mybucket" }"#;
        let expected: Value = json!({
            "resource": {
                "aws_s3_bucket": {
                    "mybucket": {
                        "name": "mybucket"
                    }
                }
            }
        });
        assert_eq!(expected, from_str::<Value>(h).unwrap());

        let h = r#"block { name = "asdf" }"#;
        let expected: Value = json!({
            "block": {
                "name": "asdf"
            }
        });
        assert_eq!(expected, from_str::<Value>(h).unwrap());
    }

    #[test]
    fn test_duplicate_block() {
        let h = r#"
            block {
              foo {
                bar = "baz"
              }

              foo {
                bar = 1
              }
            }

            other "one" "two" {
              foo = "bar"
            }

            other "two" "three" {
              bar = "baz"
            }
        "#;
        let expected = json!({
            "block": {
                "foo": [
                    {
                        "bar": "baz"
                    },
                    {
                        "bar": 1
                    }
                ]
            },
            "other": {
                "one": {
                    "two": {
                        "foo": "bar"
                    }
                },
                "two": {
                    "three": {
                        "bar": "baz"
                    }
                }
            }
        });
        assert_eq!(expected, from_str::<Value>(h).unwrap());

        let h = r#"
            foo { bar = "baz" }
            foo { bar = 1 }
        "#;
        let expected = json!({
            "foo": [
                {
                    "bar": "baz"
                },
                {
                    "bar": 1
                }
            ]
        });
        assert_eq!(expected, from_str::<Value>(h).unwrap());
    }

    #[test]
    fn test_duplicate_attribute() {
        let h = r#"
            foo = ["bar"]
            foo = ["baz"]
        "#;
        let expected = json!({"foo": ["baz"]});
        assert_eq!(expected, from_str::<Value>(h).unwrap());
    }

    #[test]
    fn test_duplicate_attribute_and_block() {
        let h = r#"
            foo = ["bar"]
            foo { bar = "baz" }
        "#;
        let expected = json!({"foo": {"bar": "baz"}});
        assert_eq!(expected, from_str::<Value>(h).unwrap());

        let h = r#"
            foo { bar = "baz" }
            foo = ["bar"]
        "#;
        let expected = json!({"foo": ["bar"]});
        assert_eq!(expected, from_str::<Value>(h).unwrap());
    }

    #[test]
    fn test_tuple() {
        let h = r#"foo = [true, 2, "three", var.enabled]"#;
        let expected: Value = json!({
            "foo": [true, 2, "three", "${var.enabled}"]
        });
        assert_eq!(expected, from_str::<Value>(h).unwrap());
    }

    #[test]
    fn test_struct() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            foo: u32,
        }

        let h = r#"foo = 1"#;
        let expected = Test { foo: 1 };
        assert_eq!(expected, from_str::<Test>(h).unwrap());
    }

    #[test]
    fn test_enum() {
        #[derive(Deserialize, PartialEq, Debug)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            value: E,
        }

        let h = r#"value = "Unit""#;
        let expected = Test { value: E::Unit };
        assert_eq!(expected, from_str::<Test>(h).unwrap());

        let h = r#"Newtype = 1"#;
        let expected = E::Newtype(1);
        assert_eq!(expected, from_str::<E>(h).unwrap());

        let h = r#"Tuple = [1,2]"#;
        let expected = E::Tuple(1, 2);
        assert_eq!(expected, from_str::<E>(h).unwrap());

        let h = r#"value = {"Struct" = {"a" = 1}}"#;
        let expected = Test {
            value: E::Struct { a: 1 },
        };
        assert_eq!(expected, from_str::<Test>(h).unwrap());
    }

    #[test]
    fn test_invalid_hcl() {
        let h = r#"invalid["#;
        assert!(from_str::<Value>(h).is_err());
    }
}

//! Deserialize HCL data to a Rust data structure.
//!
//! The `Deserializer` implementation tries to follow the [HCL JSON Specification][hcl-json-spec]
//! as close as possible.
//!
//! [hcl-json-spec]: https://github.com/hashicorp/hcl/blob/main/json/spec.md

use crate::{parser, Error, Number, Result, Value};
use indexmap::{map, IndexMap};
use serde::de::{
    self, value::StringDeserializer, DeserializeOwned, DeserializeSeed, EnumAccess,
    IntoDeserializer, MapAccess, SeqAccess, VariantAccess, Visitor,
};
use serde::{forward_to_deserialize_any, Deserialize};
use std::vec;

/// A structure that deserializes HCL into Rust values.
pub struct Deserializer {
    value: Value,
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
        Ok(Deserializer::from_value(body.into()))
    }

    fn from_value(value: Value) -> Self {
        Deserializer { value }
    }
}

/// Deserialize an instance of type `T` from a string of HCL text.
///
/// If preserving HCL semantics is required consider using [`hcl::parse`][parse] to parse the
/// input into a [`Body`][Body].
///
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
    T: Deserialize<'de>,
{
    let mut deserializer = Deserializer::from_str(s)?;
    T::deserialize(&mut deserializer)
}

/// Deserialize an instance of type `T` from an IO stream of HCL.
///
/// If preserving HCL semantics is required consider using [`hcl::parse`][parse] to parse the
/// input into a [`Body`][Body].
///
/// [parse]: ../fn.parse.html
/// [Body]: ../struct.Body.html
/////
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
    T: DeserializeOwned,
    R: std::io::Read,
{
    let mut s = String::new();
    reader.read_to_string(&mut s)?;

    from_str(&s)
}

/// Deserialize an instance of type `T` from a byte slice.
///
/// ## Errors
///
/// This functions fails with an error if `buf` does not contain valid UTF-8 or if the data does
/// not match the structure of `T`.
pub fn from_slice<'de, T>(buf: &'de [u8]) -> Result<T>
where
    T: Deserialize<'de>,
{
    let s = std::str::from_utf8(buf)?;
    from_str(s)
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value.take() {
            Value::Null => visitor.visit_unit(),
            Value::Bool(b) => visitor.visit_bool(b),
            Value::Number(n) => match n {
                Number::PosInt(i) => visitor.visit_u64(i),
                Number::NegInt(i) => visitor.visit_i64(i),
                Number::Float(f) => visitor.visit_f64(f),
            },
            Value::String(s) => visitor.visit_string(s),
            Value::Array(array) => visitor.visit_seq(Seq::new(array)),
            Value::Object(object) => visitor.visit_map(Map::new(object)),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value.take() {
            Value::String(s) => visitor.visit_enum(s.into_deserializer()),
            Value::Object(object) => visitor.visit_enum(Enum::new(object)),
            _ => Err(Error::expected("enum")),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

struct Seq {
    iter: vec::IntoIter<Value>,
}

impl Seq {
    fn new(vec: Vec<Value>) -> Self {
        Self {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> SeqAccess<'de> for Seq {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed
                .deserialize(&mut Deserializer::from_value(value))
                .map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.iter.size_hint().1
    }
}

struct Map {
    iter: map::IntoIter<String, Value>,
    value: Value,
}

impl Map {
    fn new(map: IndexMap<String, Value>) -> Self {
        Self {
            iter: map.into_iter(),
            value: Value::Null,
        }
    }
}

impl<'de> MapAccess<'de> for Map {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = value;
                seed.deserialize(key.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut Deserializer::from_value(self.value.take()))
    }

    fn size_hint(&self) -> Option<usize> {
        self.iter.size_hint().1
    }
}

struct Enum {
    iter: map::IntoIter<String, Value>,
}

impl Enum {
    fn new(map: IndexMap<String, Value>) -> Self {
        Self {
            iter: map.into_iter(),
        }
    }
}

impl<'de> EnumAccess<'de> for Enum {
    type Error = Error;
    type Variant = EnumVariant;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((value, variant)) => Ok((
                seed.deserialize::<StringDeserializer<Error>>(value.into_deserializer())?,
                EnumVariant::new(variant),
            )),
            None => Err(Error::expected("variant")),
        }
    }
}

struct EnumVariant {
    value: Value,
}

impl EnumVariant {
    fn new(value: Value) -> Self {
        Self { value }
    }
}

impl<'de> VariantAccess<'de> for EnumVariant {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(Error::expected("string"))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut Deserializer::from_value(self.value))
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(&mut Deserializer::from_value(self.value), visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(&mut Deserializer::from_value(self.value), visitor)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
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
    fn test_terraform() {
        let hcl = std::fs::read_to_string("fixtures/test.tf").unwrap();
        let value: Value = from_str(&hcl).unwrap();
        let expected = json!({
            "resource": {
                "aws_eks_cluster": {
                    "this": {
                        "count": "${var.create_eks ? 1 : 0}",
                        "name": "${var.cluster_name}",
                        "enabled_cluster_log_types": "${var.cluster_enabled_log_types}",
                        "role_arn": "${local.cluster_iam_role_arn}",
                        "version": "${var.cluster_version}",
                        "vpc_config": {
                            "security_group_ids": "${compact([local.cluster_security_group_id])}",
                            "subnet_ids": "${var.subnets}"
                        },
                        "kubernetes_network_config": {
                            "service_ipv4_cidr": "${var.cluster_service_ipv4_cidr}"
                        },
                        "dynamic": {
                            "encryption_config": {
                                "for_each": "${toset(var.cluster_encryption_config)}",
                                "content": {
                                    "provider": {
                                        "key_arn": "${encryption_config.value[\"provider_key_arn\"]}"
                                    },
                                    "resources": "${encryption_config.value[\"resources\"]}"
                                }
                            }
                        },
                        "tags": "${merge(\n    var.tags,\n    var.cluster_tags,\n  )}",
                        "depends_on": ["${aws_cloudwatch_log_group.this}"]
                    }
                },
                "aws_s3_bucket": {
                    "mybucket": {
                        "name": "mybucket"
                    },
                    "otherbucket": {
                        "name": "otherbucket"
                    }
                }
            }
        });
        assert_eq!(expected, value);
    }

    #[test]
    fn test_invalid_hcl() {
        let h = r#"invalid["#;
        assert!(from_str::<Value>(h).is_err());
    }
}

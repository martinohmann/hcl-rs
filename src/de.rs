//! Deserialize HCL data to a Rust data structure.
//!
//! The `Deserializer` implementation tries to follow the [HCL JSON Specification][hcl-json-spec]
//! as close as possible.
//!
//! [hcl-json-spec]: https://github.com/hashicorp/hcl/blob/main/json/spec.md

use crate::parser::{self, interpolate, Node};
use crate::{Error, Result};
use indexmap::{map, IndexMap};
use serde::de::{
    self, DeserializeOwned, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};
use serde::forward_to_deserialize_any;
use serde::Deserialize;
use std::str::FromStr;
use std::vec;

/// A structure that deserializes HCL into Rust values.
pub struct Deserializer<'de> {
    node: Option<Node<'de>>,
}

impl<'de> Deserializer<'de> {
    /// Creates a HCL deserializer from a `&str`.
    ///
    /// ## Errors
    ///
    /// An [`Error`][Error] is returned when the input is not valid HCL.
    ///
    /// [Error]: ../error/enum.Error.html
    pub fn from_str(input: &'de str) -> Result<Self> {
        let root = parser::parse(input)?;
        Ok(Deserializer::from_node(root))
    }

    fn from_node(node: Node<'de>) -> Self {
        Deserializer { node: Some(node) }
    }
}

/// Deserialize an instance of type `T` from a string of HCL text.
pub fn from_str<'de, T>(s: &'de str) -> Result<T>
where
    T: Deserialize<'de>,
{
    let mut deserializer = Deserializer::from_str(s)?;
    T::deserialize(&mut deserializer)
}

/// Deserialize an instance of type `T` from an IO stream of HCL.
pub fn from_reader<T, R>(mut reader: R) -> Result<T>
where
    T: DeserializeOwned,
    R: std::io::Read,
{
    let mut s = String::new();
    reader.read_to_string(&mut s)?;

    from_str(&s)
}

// Utility functions for consuming the input.
impl<'de> Deserializer<'de> {
    fn peek_node(&self) -> Result<&Node<'de>> {
        self.node.as_ref().ok_or(Error::Eof)
    }

    fn take_node(&mut self) -> Result<Node<'de>> {
        self.node.take().ok_or(Error::Eof)
    }

    fn parse_bool(&mut self) -> Result<bool> {
        match self.take_node()? {
            Node::Boolean(pair) => Ok(pair.as_str().parse().unwrap()),
            node => Err(Error::expected_span("boolean", node.as_span())),
        }
    }

    fn parse_int<T>(&mut self) -> Result<T>
    where
        T: FromStr,
    {
        let node = self.take_node()?;
        let span = node.as_span();

        let res = match node {
            Node::Int(pair) => pair.as_str().parse().map_err(|_| Error::new("Invalid int")),
            _ => Err(Error::expected("int")),
        };

        res.map_err(|err| err.with_span(span))
    }

    fn parse_float<T>(&mut self) -> Result<T>
    where
        T: FromStr,
    {
        let node = self.take_node()?;
        let span = node.as_span();

        let res = match node {
            Node::Float(pair) => pair
                .as_str()
                .parse()
                .map_err(|_| Error::new("Invalid float")),
            _ => Err(Error::expected("float")),
        };

        res.map_err(|err| err.with_span(span))
    }

    fn parse_str(&mut self) -> Result<&'de str> {
        match self.take_node()? {
            Node::String(pair) => Ok(pair.as_str()),
            node => Err(Error::expected_span("string", node.as_span())),
        }
    }

    fn parse_char(&mut self) -> Result<char> {
        let node = self.take_node()?;
        let span = node.as_span();

        let res = match node {
            Node::String(pair) => {
                let mut chars = pair.as_str().chars();

                match (chars.next(), chars.next()) {
                    (Some(c), None) => Ok(c),
                    (_, _) => Err(Error::expected("char")),
                }
            }
            _ => Err(Error::expected("string")),
        };

        res.map_err(|err| err.with_span(span))
    }

    fn interpolate_expression(&mut self) -> Result<String> {
        match self.take_node()? {
            Node::Expression(pair) => Ok(interpolate(pair.as_str())),
            node => Err(Error::expected_span("expression", node.as_span())),
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek_node()? {
            Node::Null(_) => self.deserialize_unit(visitor),
            Node::Boolean(_) => self.deserialize_bool(visitor),
            Node::String(_) => self.deserialize_str(visitor),
            Node::Float(_) => self.deserialize_f64(visitor),
            Node::Int(_) => self.deserialize_i64(visitor),
            Node::Seq(_) | Node::BlockBody(_) => self.deserialize_seq(visitor),
            Node::Map(_) | Node::Attribute(_) | Node::Block(_) => self.deserialize_map(visitor),
            // Anthing else is treated as an expression and gets interpolated to distinguish it
            // from normal string values.
            _ => visitor.visit_string(self.interpolate_expression()?),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_int()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_int()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_int()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_int()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_int()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_int()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_int()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_int()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.parse_float()?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.parse_float()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_char(self.parse_char()?)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_str()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bytes(self.parse_str()?.as_bytes())
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_byte_buf(self.parse_str()?.as_bytes().to_vec())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek_node()? {
            Node::Null(_) => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.take_node()? {
            Node::Null(_) => visitor.visit_unit(),
            node => Err(Error::expected_span("null", node.as_span())),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.take_node()? {
            Node::Seq(nodes) | Node::BlockBody(nodes) => visitor.visit_seq(Seq::new(nodes)),
            node => Err(Error::expected_span("sequence", node.as_span())),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.take_node()? {
            Node::Map(map) | Node::Attribute(map) | Node::Block(map) => {
                visitor.visit_map(Map::new(map))
            }
            node => Err(Error::expected_span("map", node.as_span())),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
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
        let node = self.take_node()?;
        let span = node.as_span();

        let res = match node {
            Node::String(pair) => visitor.visit_enum(pair.as_str().into_deserializer()),
            Node::Map(map) | Node::Attribute(map) | Node::Block(map) => {
                visitor.visit_enum(Enum::new(map))
            }
            _ => Err(Error::expected("enum")),
        };

        res.map_err(|err| err.with_span(span))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct StringDeserializer {
    key: String,
}

impl StringDeserializer {
    fn new(key: &str) -> Self {
        Self {
            key: key.to_owned(),
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut StringDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_str(&self.key)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct Seq<'de> {
    iter: vec::IntoIter<Node<'de>>,
}

impl<'de> Seq<'de> {
    fn new(nodes: Vec<Node<'de>>) -> Self {
        Self {
            iter: nodes.into_iter(),
        }
    }
}

impl<'de> SeqAccess<'de> for Seq<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(node) => seed
                .deserialize(&mut Deserializer::from_node(node))
                .map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.iter.size_hint().1
    }
}

struct Map<'de> {
    iter: map::IntoIter<String, Node<'de>>,
    value: Option<Node<'de>>,
}

impl<'de> Map<'de> {
    fn new(map: IndexMap<String, Node<'de>>) -> Self {
        Self {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for Map<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(&mut StringDeserializer::new(&key))
                    .map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut Deserializer::from_node(self.value.take().unwrap()))
    }

    fn size_hint(&self) -> Option<usize> {
        self.iter.size_hint().1
    }
}

struct Enum<'de> {
    iter: map::IntoIter<String, Node<'de>>,
}

impl<'de> Enum<'de> {
    fn new(map: IndexMap<String, Node<'de>>) -> Self {
        Self {
            iter: map.into_iter(),
        }
    }
}

impl<'de, 'a> EnumAccess<'de> for Enum<'de> {
    type Error = Error;
    type Variant = EnumVariant<'de>;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((value, variant)) => Ok((
                seed.deserialize(&mut StringDeserializer::new(&value))?,
                EnumVariant::new(variant),
            )),
            None => Err(Error::expected("variant")),
        }
    }
}

struct EnumVariant<'de> {
    node: Node<'de>,
}

impl<'de> EnumVariant<'de> {
    fn new(node: Node<'de>) -> Self {
        Self { node }
    }
}

impl<'de> VariantAccess<'de> for EnumVariant<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(Error::expected("string"))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut Deserializer::from_node(self.node))
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(&mut Deserializer::from_node(self.node), visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(&mut Deserializer::from_node(self.node), visitor)
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
                    "mybucket": [
                        {
                            "name": "mybucket"
                        }
                    ]
                }
            }
        });
        assert_eq!(expected, from_str::<Value>(h).unwrap());

        let h = r#"block { name = "asdf" }"#;
        let expected: Value = json!({
            "block": [
                {
                    "name": "asdf"
                }
            ]
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
            "block": [
                {
                    "foo": [
                        {
                            "bar": "baz"
                        },
                        {
                            "bar": 1
                        }
                    ]
                }
            ],
            "other": {
                "one": {
                    "two": [
                        {
                            "foo": "bar"
                        }
                    ]
                },
                "two": {
                    "three": [
                        {
                            "bar": "baz"
                        }
                    ]
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
        let expected = json!({"foo": [{"bar": "baz"}]});
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
                    "this": [
                        {
                            "count": "${var.create_eks ? 1 : 0}",
                            "name": "${var.cluster_name}",
                            "enabled_cluster_log_types": "${var.cluster_enabled_log_types}",
                            "role_arn": "${local.cluster_iam_role_arn}",
                            "version": "${var.cluster_version}",
                            "vpc_config": [
                                {
                                    "security_group_ids": "${compact([local.cluster_security_group_id])}",
                                    "subnet_ids": "${var.subnets}"
                                }
                            ],
                            "kubernetes_network_config": [
                                {
                                    "service_ipv4_cidr": "${var.cluster_service_ipv4_cidr}"
                                },
                            ],
                            "dynamic": {
                                "encryption_config": [
                                    {
                                        "for_each": "${toset(var.cluster_encryption_config)}",
                                        "content": [
                                            {
                                                "provider": [
                                                    {
                                                        "key_arn": "${encryption_config.value[\"provider_key_arn\"]}"
                                                    }
                                                ],
                                                "resources": "${encryption_config.value[\"resources\"]}"
                                            }
                                        ]
                                    }
                                ]
                            },
                            "tags": "${merge(\n    var.tags,\n    var.cluster_tags,\n  )}",
                            "depends_on": ["${aws_cloudwatch_log_group.this}"]
                        }
                    ]
                },
                "aws_s3_bucket": {
                    "mybucket": [
                        {
                            "name": "mybucket"
                        }
                    ],
                    "otherbucket": [
                        {
                            "name": "otherbucket"
                        }
                    ]
                }
            }
        });
        assert_eq!(expected, value);
    }
}

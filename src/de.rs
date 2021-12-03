//! Deserialize HCL data to a Rust data structure.

use crate::{
    parser::{HclParser, Rule},
    Error, Result,
};
use pest::iterators::{Pair, Pairs};
use pest::Parser as ParserTrait;
use serde::de::{
    self, DeserializeOwned, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};
use serde::Deserialize;
use std::str::FromStr;

/// A structure that deserializes HCL into Rust values.
pub struct Deserializer<'de> {
    pair: Option<Pair<'de, Rule>>,
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
        let pair = HclParser::parse(Rule::hcl, input)
            .map_err(|e| Error::ParseError(e.to_string()))?
            .next()
            .unwrap();
        Ok(Deserializer::from_pair(pair))
    }

    fn from_pair(pair: Pair<'de, Rule>) -> Self {
        Deserializer { pair: Some(pair) }
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
    fn peek_pair(&mut self) -> Result<&Pair<'de, Rule>> {
        self.pair.as_ref().ok_or(Error::Eof)
    }

    fn peek_rule(&mut self) -> Result<Rule> {
        self.peek_pair().map(Pair::as_rule)
    }

    fn take_pair(&mut self) -> Result<Pair<'de, Rule>> {
        self.pair.take().ok_or(Error::Eof)
    }

    fn parse_bool(&mut self) -> Result<bool> {
        let pair = self.take_pair()?;

        match pair.as_rule() {
            Rule::boolean => Ok(pair.as_str().parse().unwrap()),
            _ => Err(Error::token_expected("boolean")),
        }
    }

    fn parse_int<T>(&mut self) -> Result<T>
    where
        T: FromStr,
    {
        let pair = self.take_pair()?;

        match pair.as_rule() {
            Rule::int => pair.as_str().parse().map_err(|_| Error::Syntax),
            _ => Err(Error::token_expected("int")),
        }
    }

    fn parse_float<T>(&mut self) -> Result<T>
    where
        T: FromStr,
    {
        let pair = self.take_pair()?;

        match pair.as_rule() {
            Rule::float => pair.as_str().parse().map_err(|_| Error::Syntax),
            _ => Err(Error::token_expected("float")),
        }
    }

    fn parse_str(&mut self) -> Result<&'de str> {
        let pair = self.take_pair()?;

        match pair.as_rule() {
            Rule::heredoc => Ok(pair.into_inner().nth(1).unwrap().as_str()),
            Rule::block_identifier | Rule::string_lit => {
                Ok(pair.into_inner().next().unwrap().as_str())
            }
            Rule::identifier => Ok(pair.as_str()),
            _ => Err(Error::token_expected(
                "string, identifier, block identifier, or heredoc",
            )),
        }
    }

    fn parse_char(&mut self) -> Result<char> {
        let s = self.parse_str()?;

        if s.len() == 1 {
            Ok(s.chars().next().unwrap())
        } else {
            Err(Error::token_expected("char"))
        }
    }

    fn interpolate_expression(&mut self) -> Result<String> {
        Ok(format!("${{{}}}", self.take_pair()?.as_str()))
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek_rule()? {
            Rule::null => self.deserialize_unit(visitor),
            Rule::boolean => self.deserialize_bool(visitor),
            // Strings
            Rule::string_lit => self.deserialize_string(visitor),
            Rule::identifier => self.deserialize_string(visitor),
            Rule::block_identifier => self.deserialize_string(visitor),
            Rule::heredoc => self.deserialize_string(visitor),
            // Numbers
            Rule::float => self.deserialize_f64(visitor),
            Rule::int => self.deserialize_i64(visitor),
            // Seqs
            Rule::config_file => self.deserialize_seq(visitor),
            Rule::block_keys => self.deserialize_seq(visitor),
            Rule::block_body => self.deserialize_seq(visitor),
            Rule::tuple => self.deserialize_seq(visitor),
            // Maps
            Rule::attribute => self.deserialize_map(visitor),
            Rule::block => self.deserialize_map(visitor),
            Rule::object => self.deserialize_map(visitor),
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
        if self.peek_rule()? == Rule::null {
            self.take_pair()?; // consume `null`
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.take_pair()?.as_rule() {
            Rule::null => visitor.visit_unit(),
            _ => Err(Error::token_expected("null")),
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
        let pair = self.take_pair()?;

        match pair.as_rule() {
            Rule::config_file | Rule::block_keys | Rule::block_body | Rule::tuple => {
                visitor.visit_seq(Seq::new(pair.into_inner()))
            }
            _ => Err(Error::token_expected(
                "config file, block, block keys, block body or tuple",
            )),
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
        let pair = self.take_pair()?;

        match pair.as_rule() {
            Rule::attribute => visitor.visit_map(Structure::new(
                "attribute",
                &["kind", "key", "value"],
                pair.into_inner(),
            )),
            Rule::block => visitor.visit_map(Structure::new(
                "block",
                &["kind", "ident", "keys", "body"],
                pair.into_inner(),
            )),
            Rule::object => visitor.visit_map(Map::new(pair.into_inner())),
            _ => Err(Error::token_expected("attribute or object")),
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
        match self.peek_rule()? {
            Rule::string_lit | Rule::block_identifier | Rule::identifier | Rule::heredoc => {
                visitor.visit_enum(self.parse_str()?.into_deserializer())
            }
            Rule::attribute | Rule::object => {
                visitor.visit_enum(Enum::new(self.take_pair()?.into_inner()))
            }
            _ => Err(Error::token_expected("enum")),
        }
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

struct Seq<'de> {
    pairs: Pairs<'de, Rule>,
}

impl<'de> Seq<'de> {
    fn new(pairs: Pairs<'de, Rule>) -> Self {
        Self { pairs }
    }
}

impl<'de> SeqAccess<'de> for Seq<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        match self.pairs.next() {
            Some(pair) => seed
                .deserialize(&mut Deserializer::from_pair(pair))
                .map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.pairs.size_hint().1
    }
}

struct Structure<'de> {
    kind: Option<&'static str>,
    keys: std::slice::Iter<'static, &'static str>,
    pairs: Pairs<'de, Rule>,
}

impl<'de> Structure<'de> {
    fn new(kind: &'static str, keys: &'static [&'static str], pairs: Pairs<'de, Rule>) -> Self {
        Self {
            kind: Some(kind),
            keys: keys.iter(),
            pairs,
        }
    }
}

impl<'de> MapAccess<'de> for Structure<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match self.keys.next() {
            Some(key) => seed.deserialize(key.into_deserializer()).map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        match self.kind.take() {
            Some(kind) => seed.deserialize(kind.into_deserializer()),
            None => match self.pairs.next() {
                Some(pair) => seed.deserialize(&mut Deserializer::from_pair(pair)),
                None => Err(Error::token_expected("structure")),
            },
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.keys.size_hint().1.map(|hint| hint / 2)
    }
}

struct Map<'de> {
    pairs: Pairs<'de, Rule>,
}

impl<'de> Map<'de> {
    fn new(pairs: Pairs<'de, Rule>) -> Self {
        Self { pairs }
    }
}

impl<'de> MapAccess<'de> for Map<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        match self.pairs.next() {
            Some(pair) => seed
                .deserialize(&mut Deserializer::from_pair(pair))
                .map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        match self.pairs.next() {
            Some(pair) => seed.deserialize(&mut Deserializer::from_pair(pair)),
            None => Err(Error::token_expected("map value")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        self.pairs.size_hint().1.map(|hint| hint / 2)
    }
}

struct Enum<'de> {
    pairs: Pairs<'de, Rule>,
}

impl<'de> Enum<'de> {
    fn new(pairs: Pairs<'de, Rule>) -> Self {
        Self { pairs }
    }
}

impl<'de> EnumAccess<'de> for Enum<'de> {
    type Error = Error;
    type Variant = EnumVariant<'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let mut pairs = self.pairs;

        match pairs.next() {
            Some(pair) => {
                let val = seed.deserialize(&mut Deserializer::from_pair(pair))?;

                match pairs.next() {
                    Some(pair) => Ok((val, EnumVariant::new(pair))),
                    None => Err(Error::token_expected("variant")),
                }
            }
            None => Err(Error::token_expected("variant seed")),
        }
    }
}

struct EnumVariant<'de> {
    pair: Pair<'de, Rule>,
}

impl<'de> EnumVariant<'de> {
    fn new(pair: Pair<'de, Rule>) -> Self {
        Self { pair }
    }
}

impl<'de> VariantAccess<'de> for EnumVariant<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Err(Error::token_expected("string"))
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut Deserializer::from_pair(self.pair))
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(&mut Deserializer::from_pair(self.pair), visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(&mut Deserializer::from_pair(self.pair), visitor)
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
        let expected: Value = json!([{
            "kind": "attribute",
            "key": "foo",
            "value": "bar"
        }]);
        assert_eq!(expected, from_str::<Value>(h).unwrap());
    }

    #[test]
    fn test_object() {
        let h = r#"foo = { bar = 42, "baz" = true }"#;
        let expected: Value = json!([{
            "kind": "attribute",
            "key": "foo",
            "value": {"bar": 42, "baz": true}
        }]);
        assert_eq!(expected, from_str::<Value>(h).unwrap());
    }

    #[test]
    fn test_block() {
        let h = r#"resource "aws_s3_bucket" "mybucket" { name = "mybucket" }"#;
        let expected: Value = json!([{
            "kind": "block",
            "ident": "resource",
            "keys": ["aws_s3_bucket", "mybucket"],
            "body": [{
                "kind": "attribute",
                "key": "name",
                "value": "mybucket"
            }],
        }]);
        assert_eq!(expected, from_str::<Value>(h).unwrap());

        let h = r#"block { name = "asdf" }"#;
        let expected: Value = json!([{
            "kind": "block",
            "ident": "block",
            "keys": [],
            "body": [{
                "kind": "attribute",
                "key": "name",
                "value": "asdf"
            }],
        }]);
        assert_eq!(expected, from_str::<Value>(h).unwrap());
    }

    #[test]
    fn test_tuple() {
        let h = r#"foo = [true, 2, "three", var.enabled]"#;
        let expected: Value = json!([{
            "kind": "attribute",
            "key": "foo",
            "value": [true, 2, "three", "${var.enabled}"]
        }]);
        assert_eq!(expected, from_str::<Value>(h).unwrap());
    }

    #[test]
    fn test_struct() {
        #[derive(Deserialize, PartialEq, Debug)]
        struct Test {
            kind: String,
            key: String,
            value: u32,
        }

        let h = r#"foo = 1"#;
        let expected = vec![Test {
            kind: "attribute".into(),
            key: "foo".into(),
            value: 1,
        }];
        assert_eq!(expected, from_str::<Vec<Test>>(h).unwrap());
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
            kind: String,
            key: String,
            value: E,
        }

        let h = r#"foo = "Unit""#;
        let expected = vec![Test {
            kind: "attribute".into(),
            key: "foo".into(),
            value: E::Unit,
        }];
        assert_eq!(expected, from_str::<Vec<Test>>(h).unwrap());

        let h = r#"Newtype = 1"#;
        let expected = vec![E::Newtype(1)];
        assert_eq!(expected, from_str::<Vec<E>>(h).unwrap());

        let h = r#"Tuple = [1,2]"#;
        let expected = vec![E::Tuple(1, 2)];
        assert_eq!(expected, from_str::<Vec<E>>(h).unwrap());

        let h = r#"foo = {"Struct" = {"a" = 1}}"#;
        let expected = vec![Test {
            kind: "attribute".into(),
            key: "foo".into(),
            value: E::Struct { a: 1 },
        }];
        assert_eq!(expected, from_str::<Vec<Test>>(h).unwrap());
    }

    #[test]
    fn test_terraform() {
        let hcl = std::fs::read_to_string("fixtures/test.tf").unwrap();
        let value: Value = from_str(&hcl).unwrap();
        let expected = json!([
            {
                "kind": "block",
                "ident": "resource",
                "keys": ["aws_eks_cluster", "this"],
                "body": [
                    {
                        "kind": "attribute",
                        "key": "count",
                        "value": "${var.create_eks ? 1 : 0}"
                    },
                    {
                        "kind": "attribute",
                        "key": "name",
                        "value": "${var.cluster_name}"
                    },
                    {
                        "kind": "attribute",
                        "key": "enabled_cluster_log_types",
                        "value": "${var.cluster_enabled_log_types}"
                    },
                    {
                        "kind": "attribute",
                        "key": "role_arn",
                        "value": "${local.cluster_iam_role_arn}"
                    },
                    {
                        "kind": "attribute",
                        "key": "version",
                        "value": "${var.cluster_version}"
                    },
                    {
                        "kind": "block",
                        "ident": "vpc_config",
                        "keys": [],
                        "body": [
                            {
                                "kind": "attribute",
                                "key": "security_group_ids",
                                "value": "${compact([local.cluster_security_group_id])}"
                            },
                            {
                                "kind": "attribute",
                                "key": "subnet_ids",
                                "value": "${var.subnets}"
                            },
                        ]
                    },
                    {
                        "kind": "block",
                        "ident": "kubernetes_network_config",
                        "keys": [],
                        "body": [
                            {
                                "kind": "attribute",
                                "key": "service_ipv4_cidr",
                                "value": "${var.cluster_service_ipv4_cidr}"
                            },
                        ]
                    },
                    {
                        "kind": "block",
                        "ident": "dynamic",
                        "keys": ["encryption_config"],
                        "body": [
                            {
                                "kind": "attribute",
                                "key": "for_each",
                                "value": "${toset(var.cluster_encryption_config)}"
                            },
                            {
                                "kind": "block",
                                "ident": "content",
                                "keys": [],
                                "body": [
                                    {
                                        "kind": "block",
                                        "ident": "provider",
                                        "keys": [],
                                        "body": [
                                            {
                                                "kind": "attribute",
                                                "key": "key_arn",
                                                "value": "${encryption_config.value[\"provider_key_arn\"]}"
                                            }
                                        ]
                                    },
                                    {
                                        "kind": "attribute",
                                        "key": "resources",
                                        "value": "${encryption_config.value[\"resources\"]}"
                                    }
                                ]
                            }
                        ]
                    },
                    {
                        "kind": "attribute",
                        "key": "tags",
                        "value": "${merge(\n    var.tags,\n    var.cluster_tags,\n  )}"
                    },
                    {
                        "kind": "attribute",
                        "key": "depends_on",
                        "value": ["${aws_cloudwatch_log_group.this}"]
                    }
                ]
            }
        ]);
        assert_eq!(expected, value);
    }
}

//! Deserialize impls for HCL structure types.

use super::{
    marker, Attribute, Block, BlockLabel, Body, Expression, ObjectKey, RawExpression, Structure,
};
use crate::{Error, Number, OptionExt, Result};
use serde::de::{
    self,
    value::{BorrowedStrDeserializer, MapDeserializer, SeqAccessDeserializer, SeqDeserializer},
    IntoDeserializer,
};
use serde::forward_to_deserialize_any;
use std::fmt;

impl<'de> de::Deserialize<'de> for Body {
    fn deserialize<D>(deserializer: D) -> Result<Body, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct BodyVisitor;

        impl<'de> de::Visitor<'de> for BodyVisitor {
            type Value = Body;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an HCL body")
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                deserializer.deserialize_seq(self)
            }

            fn visit_seq<V>(self, visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                de::Deserialize::deserialize(SeqAccessDeserializer::new(visitor)).map(Body)
            }
        }

        deserializer.deserialize_newtype_struct(marker::BODY, BodyVisitor)
    }
}

pub struct BodyDeserializer {
    value: Body,
}

impl BodyDeserializer {
    pub fn new(value: Body) -> BodyDeserializer {
        BodyDeserializer { value }
    }
}

impl<'de, 'a> de::Deserializer<'de> for BodyDeserializer {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(SeqDeserializer::new(self.value.into_inner().into_iter()))
    }
}

pub struct StructureDeserializer {
    value: Structure,
}

impl<'de> IntoDeserializer<'de, Error> for Structure {
    type Deserializer = StructureDeserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        StructureDeserializer { value: self }
    }
}

impl<'de> de::Deserializer<'de> for StructureDeserializer {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }
}

impl<'de> de::EnumAccess<'de> for StructureDeserializer {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let variant = match self.value {
            Structure::Attribute(_) => "Attribute",
            Structure::Block(_) => "Block",
        };

        seed.deserialize(BorrowedStrDeserializer::new(variant))
            .map(|value| (value, self))
    }
}

impl<'de> de::VariantAccess<'de> for StructureDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        de::Deserialize::deserialize(self)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.value {
            Structure::Attribute(attribute) => {
                seed.deserialize(AttributeDeserializer::new(attribute))
            }
            Structure::Block(block) => seed.deserialize(BlockDeserializer::new(block)),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self, visitor)
    }
}

struct AttributeDeserializer {
    value: Attribute,
}

impl AttributeDeserializer {
    fn new(value: Attribute) -> Self {
        AttributeDeserializer { value }
    }
}

impl<'de> de::Deserializer<'de> for AttributeDeserializer {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(AttributeAccess::new(self.value))
    }
}

struct AttributeAccess {
    key: Option<String>,
    expr: Option<Expression>,
}

impl AttributeAccess {
    fn new(attr: Attribute) -> Self {
        AttributeAccess {
            key: Some(attr.key),
            expr: Some(attr.expr),
        }
    }
}

impl<'de> de::MapAccess<'de> for AttributeAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.key.is_some() {
            seed.deserialize("key".into_deserializer()).map(Some)
        } else if self.expr.is_some() {
            seed.deserialize("expr".into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if self.key.is_some() {
            seed.deserialize(self.key.consume().into_deserializer())
        } else if self.expr.is_some() {
            seed.deserialize(self.expr.consume().into_deserializer())
        } else {
            Err(de::Error::custom("invalid HCL attribute"))
        }
    }
}

struct BlockDeserializer {
    value: Block,
}

impl BlockDeserializer {
    fn new(value: Block) -> Self {
        BlockDeserializer { value }
    }
}

impl<'de> de::Deserializer<'de> for BlockDeserializer {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(BlockAccess::new(self.value))
    }
}

struct BlockAccess {
    identifier: Option<String>,
    labels: Option<Vec<BlockLabel>>,
    body: Option<Body>,
}

impl BlockAccess {
    fn new(block: Block) -> Self {
        BlockAccess {
            identifier: Some(block.identifier),
            labels: Some(block.labels),
            body: Some(block.body),
        }
    }
}

impl<'de> de::MapAccess<'de> for BlockAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.identifier.is_some() {
            seed.deserialize("identifier".into_deserializer()).map(Some)
        } else if self.labels.is_some() {
            seed.deserialize("labels".into_deserializer()).map(Some)
        } else if self.body.is_some() {
            seed.deserialize("body".into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if self.identifier.is_some() {
            seed.deserialize(self.identifier.consume().into_deserializer())
        } else if self.labels.is_some() {
            seed.deserialize(SeqDeserializer::new(self.labels.consume().into_iter()))
        } else if self.body.is_some() {
            seed.deserialize(BodyDeserializer::new(self.body.consume()))
        } else {
            Err(de::Error::custom("invalid HCL block"))
        }
    }
}

pub struct BlockLabelDeserializer {
    value: BlockLabel,
}

impl<'de> IntoDeserializer<'de, Error> for BlockLabel {
    type Deserializer = BlockLabelDeserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        BlockLabelDeserializer { value: self }
    }
}

impl<'de> de::Deserializer<'de> for BlockLabelDeserializer {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            BlockLabel::String(string) => visitor.visit_string(string),
            BlockLabel::Identifier(ident) => visitor.visit_string(ident),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }
}

impl<'de> de::EnumAccess<'de> for BlockLabelDeserializer {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let variant = match self.value {
            BlockLabel::String(_) => "String",
            BlockLabel::Identifier(_) => "Identifier",
        };

        seed.deserialize(BorrowedStrDeserializer::new(variant))
            .map(|value| (value, self))
    }
}

impl<'de> de::VariantAccess<'de> for BlockLabelDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        de::Deserialize::deserialize(self)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self, visitor)
    }
}

pub struct ExpressionDeserializer {
    value: Expression,
}

impl<'de> IntoDeserializer<'de, Error> for Expression {
    type Deserializer = ExpressionDeserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        ExpressionDeserializer { value: self }
    }
}

impl<'de, 'a> de::Deserializer<'de> for ExpressionDeserializer {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Expression::Null => visitor.visit_unit(),
            Expression::Bool(b) => visitor.visit_bool(b),
            Expression::Number(n) => match n {
                Number::PosInt(i) => visitor.visit_u64(i),
                Number::NegInt(i) => visitor.visit_i64(i),
                Number::Float(f) => visitor.visit_f64(f),
            },
            Expression::String(s) => visitor.visit_string(s),
            Expression::Array(array) => visitor.visit_seq(SeqDeserializer::new(array.into_iter())),
            Expression::Object(object) => {
                visitor.visit_map(MapDeserializer::new(object.into_iter()))
            }
            Expression::Raw(expr) => expr.into_deserializer().deserialize_any(visitor),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }
}

impl<'de> de::EnumAccess<'de> for ExpressionDeserializer {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let variant = match self.value {
            Expression::Null => "Null",
            Expression::Bool(_) => "Bool",
            Expression::Number(_) => "Number",
            Expression::String(_) => "String",
            Expression::Array(_) => "Array",
            Expression::Object(_) => "Object",
            Expression::Raw(_) => "Raw",
        };

        seed.deserialize(BorrowedStrDeserializer::new(variant))
            .map(|value| (value, self))
    }
}

impl<'de> de::VariantAccess<'de> for ExpressionDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        de::Deserialize::deserialize(self)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self, visitor)
    }
}

pub struct ObjectKeyDeserializer {
    value: ObjectKey,
}

impl<'de> IntoDeserializer<'de, Error> for ObjectKey {
    type Deserializer = ObjectKeyDeserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        ObjectKeyDeserializer { value: self }
    }
}

impl<'de, 'a> de::Deserializer<'de> for ObjectKeyDeserializer {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            ObjectKey::String(string) => visitor.visit_string(string),
            ObjectKey::Identifier(ident) => visitor.visit_string(ident),
            ObjectKey::RawExpression(expr) => expr.into_deserializer().deserialize_any(visitor),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }
}

impl<'de> de::EnumAccess<'de> for ObjectKeyDeserializer {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let variant = match self.value {
            ObjectKey::String(_) => "String",
            ObjectKey::Identifier(_) => "Identifier",
            ObjectKey::RawExpression(_) => "RawExpression",
        };

        seed.deserialize(BorrowedStrDeserializer::new(variant))
            .map(|value| (value, self))
    }
}

impl<'de> de::VariantAccess<'de> for ObjectKeyDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        de::Deserialize::deserialize(self)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self, visitor)
    }
}

pub struct RawExpressionDeserializer {
    value: RawExpression,
}

impl<'de> IntoDeserializer<'de, Error> for RawExpression {
    type Deserializer = RawExpressionDeserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        RawExpressionDeserializer { value: self }
    }
}

impl<'de, 'a> de::Deserializer<'de> for RawExpressionDeserializer {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self.value.into_inner().into_deserializer())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_terraform() {
        let input = r#"
            resource "aws_s3_bucket" "mybucket" {
              bucket        = "mybucket"
              force_destroy = true

              server_side_encryption_configuration {
                rule {
                  apply_server_side_encryption_by_default {
                    kms_master_key_id = aws_kms_key.mykey.arn
                    sse_algorithm     = "aws:kms"
                  }
                }
              }

              tags = {
                var.dynamic   = null
                "application" = "myapp"
                team          = "bar"
              }
            }"#;

        let expected = Body::builder()
            .add_block(
                Block::builder("resource")
                    .add_label("aws_s3_bucket")
                    .add_label("mybucket")
                    .add_attribute(("bucket", "mybucket"))
                    .add_attribute(("force_destroy", true))
                    .add_block(
                        Block::builder("server_side_encryption_configuration")
                            .add_block(
                                Block::builder("rule")
                                    .add_block(
                                        Block::builder("apply_server_side_encryption_by_default")
                                            .add_attribute((
                                                "kms_master_key_id",
                                                RawExpression::new("aws_kms_key.mykey.arn"),
                                            ))
                                            .add_attribute(("sse_algorithm", "aws:kms"))
                                            .build(),
                                    )
                                    .build(),
                            )
                            .build(),
                    )
                    .add_attribute((
                        "tags",
                        Expression::from_iter([
                            (
                                ObjectKey::RawExpression("var.dynamic".into()),
                                Expression::Null,
                            ),
                            (
                                ObjectKey::String("application".into()),
                                Expression::String("myapp".into()),
                            ),
                            (
                                ObjectKey::Identifier("team".into()),
                                Expression::String("bar".into()),
                            ),
                        ]),
                    ))
                    .build(),
            )
            .build();

        let body: Body = crate::from_str(input).unwrap();

        assert_eq!(expected, body);
    }
}

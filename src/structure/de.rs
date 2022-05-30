//! Deserialize impls for HCL structure types.

use super::{
    marker, Attribute, Block, BlockLabel, Body, Expression, ObjectKey, RawExpression, Structure,
};
use crate::{Error, Number, OptionExt, Result};
use indexmap::{map, IndexMap};
use serde::de::{
    self,
    value::{MapAccessDeserializer, SeqAccessDeserializer},
    IntoDeserializer,
};
use serde::forward_to_deserialize_any;
use std::fmt::{self, Display};
use std::vec;

struct Fields(&'static [&'static str]);

impl Display for Fields {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0.len() {
            0 => unreachable!(),
            1 => write!(f, "`{}`", self.0[0]),
            2 => write!(f, "`{}` or `{}`", self.0[0], self.0[1]),
            _ => {
                for (i, alt) in self.0.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "`{}`", alt)?;
                }
                Ok(())
            }
        }
    }
}

fn expected_one_of<E>(fields: &'static [&'static str]) -> E
where
    E: de::Error,
{
    de::Error::custom(format_args!(
        "missing fields, expected one of {}",
        Fields(fields)
    ))
}

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

            fn visit_seq<V>(self, visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                de::Deserialize::deserialize(SeqAccessDeserializer::new(visitor)).map(Body)
            }
        }

        deserializer.deserialize_newtype_struct(marker::BODY_NAME, BodyVisitor)
    }
}

impl<'de> de::Deserialize<'de> for Structure {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct StructureVisitor;

        impl<'de> de::Visitor<'de> for StructureVisitor {
            type Value = Structure;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an HCL structure")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                match visitor.next_key()? {
                    Some(marker::ATTRIBUTE_FIELD) => {
                        Ok(Structure::Attribute(visitor.next_value()?))
                    }
                    Some(marker::BLOCK_FIELD) => Ok(Structure::Block(visitor.next_value()?)),
                    _ => Err(expected_one_of(&[
                        marker::ATTRIBUTE_FIELD,
                        marker::BLOCK_FIELD,
                    ])),
                }
            }
        }

        deserializer.deserialize_map(StructureVisitor)
    }
}

impl<'de> de::Deserialize<'de> for BlockLabel {
    fn deserialize<D>(deserializer: D) -> Result<BlockLabel, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct BlockLabelVisitor;

        impl<'de> de::Visitor<'de> for BlockLabelVisitor {
            type Value = BlockLabel;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an HCL block label")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                match visitor.next_key()? {
                    Some(marker::IDENT_FIELD) => Ok(BlockLabel::Identifier(visitor.next_value()?)),
                    Some(marker::STRING_FIELD) => Ok(BlockLabel::String(visitor.next_value()?)),
                    _ => Err(expected_one_of(&[
                        marker::IDENT_FIELD,
                        marker::STRING_FIELD,
                    ])),
                }
            }
        }

        deserializer.deserialize_map(BlockLabelVisitor)
    }
}

impl<'de> de::Deserialize<'de> for Expression {
    fn deserialize<D>(deserializer: D) -> Result<Expression, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ExpressionVisitor;

        impl<'de> de::Visitor<'de> for ExpressionVisitor {
            type Value = Expression;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an HCL expression")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                match visitor.next_key()? {
                    Some(marker::VALUE_FIELD) => {
                        let value: ValueExpression = visitor.next_value()?;
                        Ok(value.expr)
                    }
                    Some(marker::RAW_FIELD) => Ok(Expression::Raw(visitor.next_value()?)),
                    _ => Err(expected_one_of(&[marker::VALUE_FIELD, marker::RAW_FIELD])),
                }
            }
        }

        deserializer.deserialize_map(ExpressionVisitor)
    }
}

struct ValueExpression {
    expr: Expression,
}

impl<'de> de::Deserialize<'de> for ValueExpression {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ExpressionVisitor;

        impl<'de> de::Visitor<'de> for ExpressionVisitor {
            type Value = Expression;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an HCL value expression")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Expression, E> {
                Ok(Expression::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Expression, E> {
                Ok(Expression::Number(value.into()))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Expression, E> {
                Ok(Expression::Number(value.into()))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Expression, E> {
                Ok(Expression::Number(value.into()))
            }

            fn visit_str<E>(self, value: &str) -> Result<Expression, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(value.to_owned())
            }

            fn visit_string<E>(self, value: String) -> Result<Expression, E> {
                Ok(Expression::String(value))
            }

            fn visit_none<E>(self) -> Result<Expression, E> {
                Ok(Expression::Null)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Expression, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                de::Deserialize::deserialize(deserializer)
            }

            fn visit_unit<E>(self) -> Result<Expression, E> {
                Ok(Expression::Null)
            }

            fn visit_seq<V>(self, visitor: V) -> Result<Expression, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                de::Deserialize::deserialize(SeqAccessDeserializer::new(visitor))
                    .map(Expression::Array)
            }

            fn visit_map<V>(self, visitor: V) -> Result<Expression, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                de::Deserialize::deserialize(MapAccessDeserializer::new(visitor))
                    .map(Expression::Object)
            }
        }

        let expr = deserializer.deserialize_any(ExpressionVisitor)?;
        Ok(ValueExpression { expr })
    }
}

impl<'de> de::Deserialize<'de> for ObjectKey {
    fn deserialize<D>(deserializer: D) -> Result<ObjectKey, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ObjectKeyVisitor;

        impl<'de> de::Visitor<'de> for ObjectKeyVisitor {
            type Value = ObjectKey;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an HCL object key")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                match visitor.next_key()? {
                    Some(marker::IDENT_FIELD) => Ok(ObjectKey::Identifier(visitor.next_value()?)),
                    Some(marker::STRING_FIELD) => Ok(ObjectKey::String(visitor.next_value()?)),
                    Some(marker::RAW_FIELD) => Ok(ObjectKey::RawExpression(visitor.next_value()?)),
                    _ => Err(expected_one_of(&[
                        marker::IDENT_FIELD,
                        marker::STRING_FIELD,
                        marker::RAW_FIELD,
                    ])),
                }
            }
        }

        deserializer.deserialize_map(ObjectKeyVisitor)
    }
}

impl<'de> de::Deserialize<'de> for RawExpression {
    fn deserialize<D>(deserializer: D) -> Result<RawExpression, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct RawExpressionVisitor;

        impl<'de> de::Visitor<'de> for RawExpressionVisitor {
            type Value = RawExpression;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an HCL raw expression")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(s.into())
            }
        }

        deserializer.deserialize_str(RawExpressionVisitor)
    }
}

pub(crate) struct BodyDeserializer {
    value: Option<Body>,
}

impl BodyDeserializer {
    pub(crate) fn new(value: Body) -> BodyDeserializer {
        BodyDeserializer { value: Some(value) }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut BodyDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(StructureSeqAccess::new(self.value.consume().into_inner()))
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string seq
        bytes byte_buf map struct option unit newtype_struct
        ignored_any unit_struct tuple_struct tuple enum identifier
    }
}

struct ExpressionSeqAccess {
    iter: vec::IntoIter<Expression>,
}

impl ExpressionSeqAccess {
    fn new(vec: Vec<Expression>) -> Self {
        ExpressionSeqAccess {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> de::SeqAccess<'de> for ExpressionSeqAccess {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.iter
            .next()
            .map(|expr| seed.deserialize(&mut ExpressionDeserializer::new(expr)))
            .transpose()
    }

    fn size_hint(&self) -> Option<usize> {
        self.iter.size_hint().1
    }
}

struct ExpressionMapAccess {
    iter: map::IntoIter<ObjectKey, Expression>,
    value: Option<Expression>,
}

impl ExpressionMapAccess {
    fn new(map: IndexMap<ObjectKey, Expression>) -> Self {
        ExpressionMapAccess {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> de::MapAccess<'de> for ExpressionMapAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        self.iter
            .next()
            .map(|(key, value)| {
                self.value = Some(value);

                match key {
                    ObjectKey::Identifier(identifier) => (marker::IDENT_FIELD, identifier),
                    ObjectKey::String(string) => (marker::STRING_FIELD, string),
                    ObjectKey::RawExpression(expr) => (marker::RAW_FIELD, expr.into_inner()),
                }
            })
            .map(|(field, value)| {
                seed.deserialize(MapAccessDeserializer::new(StringFieldAccess::new(
                    field, value,
                )))
            })
            .transpose()
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut ExpressionDeserializer::new(self.value.consume()))
    }

    fn size_hint(&self) -> Option<usize> {
        self.iter.size_hint().1
    }
}

struct StructureSeqAccess {
    iter: vec::IntoIter<Structure>,
}

impl StructureSeqAccess {
    fn new(vec: Vec<Structure>) -> Self {
        StructureSeqAccess {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> de::SeqAccess<'de> for StructureSeqAccess {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.iter
            .next()
            .map(|value| seed.deserialize(MapAccessDeserializer::new(StructureAccess::new(value))))
            .transpose()
    }

    fn size_hint(&self) -> Option<usize> {
        self.iter.size_hint().1
    }
}

struct FieldDeserializer(&'static str);

impl<'de> de::Deserializer<'de> for FieldDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.0)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string seq
        bytes byte_buf map struct option unit newtype_struct
        ignored_any unit_struct tuple_struct tuple enum identifier
    }
}

struct StructureAccess {
    value: Option<Structure>,
}

impl StructureAccess {
    fn new(value: Structure) -> Self {
        StructureAccess { value: Some(value) }
    }
}

impl<'de> de::MapAccess<'de> for StructureAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        self.value
            .as_ref()
            .map(|value| match value {
                Structure::Attribute(_) => marker::ATTRIBUTE_FIELD,
                Structure::Block(_) => marker::BLOCK_FIELD,
            })
            .map(|field| seed.deserialize(FieldDeserializer(field)))
            .transpose()
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.value.consume() {
            Structure::Attribute(attribute) => {
                seed.deserialize(MapAccessDeserializer::new(AttributeAccess::new(attribute)))
            }
            Structure::Block(block) => {
                seed.deserialize(MapAccessDeserializer::new(BlockAccess::new(block)))
            }
        }
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
            seed.deserialize(FieldDeserializer("key")).map(Some)
        } else if self.expr.is_some() {
            seed.deserialize(FieldDeserializer("expr")).map(Some)
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
            seed.deserialize(&mut ExpressionDeserializer::new(self.expr.consume()))
        } else {
            Err(de::Error::custom("invalid HCL attribute"))
        }
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
            seed.deserialize(FieldDeserializer("identifier")).map(Some)
        } else if self.labels.is_some() {
            seed.deserialize(FieldDeserializer("labels")).map(Some)
        } else if self.body.is_some() {
            seed.deserialize(FieldDeserializer("body")).map(Some)
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
            seed.deserialize(SeqAccessDeserializer::new(BlockLabelSeqAccess::new(
                self.labels.consume(),
            )))
        } else if self.body.is_some() {
            seed.deserialize(&mut BodyDeserializer::new(self.body.consume()))
        } else {
            Err(de::Error::custom("invalid HCL block"))
        }
    }
}

struct StringFieldAccess {
    field: &'static str,
    value: Option<String>,
}

impl StringFieldAccess {
    fn new(field: &'static str, value: String) -> Self {
        StringFieldAccess {
            field,
            value: Some(value),
        }
    }
}

impl<'de> de::MapAccess<'de> for StringFieldAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        seed.deserialize(FieldDeserializer(self.field)).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.value.consume().into_deserializer())
    }
}

struct BlockLabelSeqAccess {
    iter: vec::IntoIter<BlockLabel>,
}

impl BlockLabelSeqAccess {
    fn new(vec: Vec<BlockLabel>) -> Self {
        Self {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> de::SeqAccess<'de> for BlockLabelSeqAccess {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.iter
            .next()
            .map(|value| match value {
                BlockLabel::Identifier(identifier) => (marker::IDENT_FIELD, identifier),
                BlockLabel::String(string) => (marker::STRING_FIELD, string),
            })
            .map(|(field, value)| {
                seed.deserialize(MapAccessDeserializer::new(StringFieldAccess::new(
                    field, value,
                )))
            })
            .transpose()
    }

    fn size_hint(&self) -> Option<usize> {
        self.iter.size_hint().1
    }
}

struct ExpressionDeserializer {
    value: Option<Expression>,
}

impl ExpressionDeserializer {
    fn new(value: Expression) -> ExpressionDeserializer {
        ExpressionDeserializer { value: Some(value) }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut ExpressionDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value.consume() {
            Expression::Raw(expr) => {
                visitor.visit_map(StringFieldAccess::new(marker::RAW_FIELD, expr.into_inner()))
            }
            value => visitor.visit_map(ExpressionValueAccess::new(value)),
        }
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string seq
        bytes byte_buf map option unit struct newtype_struct
        ignored_any unit_struct tuple_struct tuple enum identifier
    }
}

struct ExpressionValueAccess {
    value: Option<Expression>,
}

impl ExpressionValueAccess {
    fn new(value: Expression) -> Self {
        ExpressionValueAccess { value: Some(value) }
    }
}

impl<'de> de::MapAccess<'de> for ExpressionValueAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        seed.deserialize(FieldDeserializer(marker::VALUE_FIELD))
            .map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut ExpressionValueDeserializer::new(self.value.consume()))
    }
}

struct ExpressionValueDeserializer {
    value: Option<Expression>,
}

impl ExpressionValueDeserializer {
    fn new(value: Expression) -> ExpressionValueDeserializer {
        ExpressionValueDeserializer { value: Some(value) }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut ExpressionValueDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.value.consume() {
            Expression::Null => visitor.visit_unit(),
            Expression::Bool(b) => visitor.visit_bool(b),
            Expression::Number(n) => match n {
                Number::PosInt(i) => visitor.visit_u64(i),
                Number::NegInt(i) => visitor.visit_i64(i),
                Number::Float(f) => visitor.visit_f64(f),
            },
            Expression::String(s) => visitor.visit_string(s),
            Expression::Array(array) => visitor.visit_seq(ExpressionSeqAccess::new(array)),
            Expression::Object(object) => visitor.visit_map(ExpressionMapAccess::new(object)),
            Expression::Raw(_) => unreachable!(),
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option enum unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
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
                var.dynamic   = true
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
                                Expression::Bool(true),
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

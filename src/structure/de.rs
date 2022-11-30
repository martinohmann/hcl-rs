//! Deserialize impls for HCL structure types.

use super::{Attribute, Block, BlockLabel, Body, Structure};
use crate::de::NewtypeStructDeserializer;
use crate::expr::Expression;
use crate::{Error, Identifier, Result};
use serde::de::{self, IntoDeserializer};
use serde::forward_to_deserialize_any;

impl<'de> IntoDeserializer<'de, Error> for Body {
    type Deserializer = NewtypeStructDeserializer<Vec<Structure>>;

    fn into_deserializer(self) -> Self::Deserializer {
        NewtypeStructDeserializer::new(self.into_inner())
    }
}

impl<'de> IntoDeserializer<'de, Error> for Structure {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for Structure {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Structure::Attribute(attribute) => visitor.visit_map(AttributeAccess::new(attribute)),
            Structure::Block(block) => visitor.visit_map(BlockAccess::new(block)),
        }
    }

    impl_deserialize_enum!();

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
    }
}

pub struct AttributeAccess {
    key: Option<Identifier>,
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
        if let Some(key) = self.key.take() {
            seed.deserialize(key.into_deserializer())
        } else if let Some(expr) = self.expr.take() {
            seed.deserialize(expr.into_deserializer())
        } else {
            Err(de::Error::custom("invalid HCL attribute"))
        }
    }
}

pub struct BlockAccess {
    identifier: Option<Identifier>,
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
        if let Some(identifier) = self.identifier.take() {
            seed.deserialize(identifier.into_deserializer())
        } else if let Some(labels) = self.labels.take() {
            seed.deserialize(labels.into_deserializer())
        } else if let Some(body) = self.body.take() {
            seed.deserialize(body.into_deserializer())
        } else {
            Err(de::Error::custom("invalid HCL block"))
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for BlockLabel {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for BlockLabel {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            BlockLabel::String(string) => visitor.visit_string(string),
            BlockLabel::Identifier(ident) => visitor.visit_string(ident.into_inner()),
        }
    }

    impl_deserialize_enum!();

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
    }
}

impl_variant_name! {
    BlockLabel => { String, Identifier },
    Structure => { Attribute, Block }
}

impl_into_map_access_deserializer! {
    Attribute => AttributeAccess,
    Block => BlockAccess
}

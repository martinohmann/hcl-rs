//! Deserialize impls for HCL structure types.

use super::{
    Attribute, Block, BlockLabel, Body, Expression, Heredoc, HeredocStripMode, Identifier,
    ObjectKey, RawExpression, Structure, TemplateExpr,
};
use crate::{Error, Number, Result};
use serde::de::{self, value::BorrowedStrDeserializer, IntoDeserializer};
use serde::forward_to_deserialize_any;

pub struct BodyDeserializer {
    value: Body,
}

impl BodyDeserializer {
    pub fn new(value: Body) -> BodyDeserializer {
        BodyDeserializer { value }
    }
}

impl<'de> IntoDeserializer<'de, Error> for Body {
    type Deserializer = BodyDeserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        BodyDeserializer { value: self }
    }
}

impl<'de> de::Deserializer<'de> for BodyDeserializer {
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
        tuple tuple_struct map struct identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Structure::Attribute(attribute) => {
                attribute.into_deserializer().deserialize_any(visitor)
            }
            Structure::Block(block) => block.into_deserializer().deserialize_any(visitor),
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

impl<'de> de::EnumAccess<'de> for StructureDeserializer {
    type Error = Error;
    type Variant = AnyVariantAccess<Self>;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let variant = match self.value {
            Structure::Attribute(_) => "Attribute",
            Structure::Block(_) => "Block",
        };

        seed.deserialize(BorrowedStrDeserializer::new(variant))
            .map(|value| (value, AnyVariantAccess::new(self)))
    }
}

pub struct AttributeDeserializer {
    value: Attribute,
}

impl<'de> IntoDeserializer<'de, Error> for Attribute {
    type Deserializer = AttributeDeserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        AttributeDeserializer { value: self }
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
        if let Some(key) = self.key.take() {
            seed.deserialize(key.into_deserializer())
        } else if let Some(expr) = self.expr.take() {
            seed.deserialize(expr.into_deserializer())
        } else {
            Err(de::Error::custom("invalid HCL attribute"))
        }
    }
}

pub struct BlockDeserializer {
    value: Block,
}

impl<'de> IntoDeserializer<'de, Error> for Block {
    type Deserializer = BlockDeserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        BlockDeserializer { value: self }
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
            BlockLabel::Identifier(ident) => ident.into_deserializer().deserialize_any(visitor),
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
    type Variant = AnyVariantAccess<Self>;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let variant = match self.value {
            BlockLabel::String(_) => "String",
            BlockLabel::Identifier(_) => "Identifier",
        };

        seed.deserialize(BorrowedStrDeserializer::new(variant))
            .map(|value| (value, AnyVariantAccess::new(self)))
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

impl<'de> de::Deserializer<'de> for ExpressionDeserializer {
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
            Expression::Array(array) => visitor.visit_seq(array.into_deserializer()),
            Expression::Object(object) => visitor.visit_map(object.into_deserializer()),
            Expression::Raw(expr) => expr.into_deserializer().deserialize_any(visitor),
            Expression::TemplateExpr(expr) => expr.into_deserializer().deserialize_any(visitor),
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
    type Variant = AnyVariantAccess<Self>;

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
            Expression::TemplateExpr(_) => "TemplateExpr",
        };

        seed.deserialize(BorrowedStrDeserializer::new(variant))
            .map(|value| (value, AnyVariantAccess::new(self)))
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

impl<'de> de::Deserializer<'de> for ObjectKeyDeserializer {
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
            ObjectKey::Identifier(ident) => ident.into_deserializer().deserialize_any(visitor),
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
    type Variant = AnyVariantAccess<Self>;

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
            .map(|value| (value, AnyVariantAccess::new(self)))
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

impl<'de> de::Deserializer<'de> for RawExpressionDeserializer {
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

pub struct TemplateExprDeserializer {
    value: TemplateExpr,
}

impl<'de> IntoDeserializer<'de, Error> for TemplateExpr {
    type Deserializer = TemplateExprDeserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        TemplateExprDeserializer { value: self }
    }
}

impl<'de> de::Deserializer<'de> for TemplateExprDeserializer {
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
            TemplateExpr::QuotedString(string) => visitor.visit_string(string),
            TemplateExpr::Heredoc(heredoc) => heredoc.into_deserializer().deserialize_any(visitor),
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

impl<'de> de::EnumAccess<'de> for TemplateExprDeserializer {
    type Error = Error;
    type Variant = AnyVariantAccess<Self>;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let variant = match self.value {
            TemplateExpr::QuotedString(_) => "QuotedString",
            TemplateExpr::Heredoc(_) => "Heredoc",
        };

        seed.deserialize(BorrowedStrDeserializer::new(variant))
            .map(|value| (value, AnyVariantAccess::new(self)))
    }
}

pub struct HeredocDeserializer {
    value: Heredoc,
}

impl<'de> IntoDeserializer<'de, Error> for Heredoc {
    type Deserializer = HeredocDeserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        HeredocDeserializer { value: self }
    }
}

impl<'de> de::Deserializer<'de> for HeredocDeserializer {
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
        visitor.visit_map(HeredocAccess::new(self.value))
    }
}

struct HeredocAccess {
    delimiter: Option<Identifier>,
    template: Option<String>,
    strip: Option<HeredocStripMode>,
}

impl HeredocAccess {
    fn new(value: Heredoc) -> Self {
        HeredocAccess {
            delimiter: Some(value.delimiter),
            template: Some(value.template),
            strip: Some(value.strip),
        }
    }
}

impl<'de> de::MapAccess<'de> for HeredocAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.delimiter.is_some() {
            seed.deserialize("delimiter".into_deserializer()).map(Some)
        } else if self.template.is_some() {
            seed.deserialize("template".into_deserializer()).map(Some)
        } else if self.strip.is_some() {
            seed.deserialize("strip".into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(delimiter) = self.delimiter.take() {
            seed.deserialize(delimiter.into_deserializer())
        } else if let Some(template) = self.template.take() {
            seed.deserialize(template.into_deserializer())
        } else if let Some(strip) = self.strip.take() {
            seed.deserialize(strip.into_deserializer())
        } else {
            Err(de::Error::custom("invalid HCL heredoc"))
        }
    }
}

pub struct HeredocStripModeDeserializer {
    value: HeredocStripMode,
}

impl<'de> IntoDeserializer<'de, Error> for HeredocStripMode {
    type Deserializer = HeredocStripModeDeserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        HeredocStripModeDeserializer { value: self }
    }
}

impl<'de> de::Deserializer<'de> for HeredocStripModeDeserializer {
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
            HeredocStripMode::None => visitor.visit_str("None"),
            HeredocStripMode::Indent => visitor.visit_str("Indent"),
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

impl<'de> de::EnumAccess<'de> for HeredocStripModeDeserializer {
    type Error = Error;
    type Variant = AnyVariantAccess<Self>;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let variant = match self.value {
            HeredocStripMode::None => "None",
            HeredocStripMode::Indent => "Indent",
        };

        seed.deserialize(BorrowedStrDeserializer::new(variant))
            .map(|value| (value, AnyVariantAccess::new(self)))
    }
}

pub struct IdentifierDeserializer {
    value: Identifier,
}

impl<'de> IntoDeserializer<'de, Error> for Identifier {
    type Deserializer = IdentifierDeserializer;

    fn into_deserializer(self) -> Self::Deserializer {
        IdentifierDeserializer { value: self }
    }
}

impl<'de> de::Deserializer<'de> for IdentifierDeserializer {
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

pub struct AnyVariantAccess<D> {
    de: D,
}

impl<D> AnyVariantAccess<D> {
    fn new(de: D) -> Self {
        AnyVariantAccess { de }
    }
}

impl<'de, D> de::VariantAccess<'de> for AnyVariantAccess<D>
where
    D: de::Deserializer<'de>,
{
    type Error = D::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        de::Deserialize::deserialize(self.de)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}

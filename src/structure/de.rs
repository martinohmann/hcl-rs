//! Deserialize impls for HCL structure types.

use super::{
    Attribute, Block, BlockLabel, Body, ElementAccess, ElementAccessOperator, Expression, FuncCall,
    Heredoc, HeredocStripMode, Identifier, ObjectKey, RawExpression, Structure, TemplateExpr,
};
use crate::{Error, Number, Result};
use serde::de::{self, value::MapAccessDeserializer, IntoDeserializer};
use serde::{forward_to_deserialize_any, Deserializer};
use std::marker::PhantomData;

// A trait that allows enum types to report the name of their variant.
trait VariantName {
    fn variant_name(&self) -> &'static str;
}

macro_rules! impl_deserialize_enum {
    () => {
        fn deserialize_enum<V>(
            self,
            _name: &'static str,
            _variants: &'static [&'static str],
            visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            visitor.visit_enum(EnumAccess::new(self))
        }
    };
}

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

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
    }
    impl_deserialize_enum!();

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Structure::Attribute(attribute) => {
                attribute.into_deserializer().deserialize_any(visitor)
            }
            Structure::Block(block) => block.into_deserializer().deserialize_any(visitor),
        }
    }
}

impl VariantName for Structure {
    fn variant_name(&self) -> &'static str {
        match self {
            Structure::Attribute(_) => "Attribute",
            Structure::Block(_) => "Block",
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for Attribute {
    type Deserializer = MapAccessDeserializer<AttributeAccess>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapAccessDeserializer::new(AttributeAccess::new(self))
    }
}

pub struct AttributeAccess {
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

impl<'de> IntoDeserializer<'de, Error> for Block {
    type Deserializer = MapAccessDeserializer<BlockAccess>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapAccessDeserializer::new(BlockAccess::new(self))
    }
}

pub struct BlockAccess {
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

impl<'de> IntoDeserializer<'de, Error> for BlockLabel {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for BlockLabel {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
    }
    impl_deserialize_enum!();

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            BlockLabel::String(string) => visitor.visit_string(string),
            BlockLabel::Identifier(ident) => ident.into_deserializer().deserialize_any(visitor),
        }
    }
}

impl VariantName for BlockLabel {
    fn variant_name(&self) -> &'static str {
        match self {
            BlockLabel::String(_) => "String",
            BlockLabel::Identifier(_) => "Identifier",
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for Expression {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for Expression {
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
        match self {
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
            Expression::VariableExpr(expr) => expr.into_deserializer().deserialize_any(visitor),
            Expression::ElementAccess(access) => {
                access.into_deserializer().deserialize_any(visitor)
            }
            Expression::FuncCall(func_call) => {
                func_call.into_deserializer().deserialize_any(visitor)
            }
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

impl VariantName for Expression {
    fn variant_name(&self) -> &'static str {
        match self {
            Expression::Null => "Null",
            Expression::Bool(_) => "Bool",
            Expression::Number(_) => "Number",
            Expression::String(_) => "String",
            Expression::Array(_) => "Array",
            Expression::Object(_) => "Object",
            Expression::Raw(_) => "Raw",
            Expression::TemplateExpr(_) => "TemplateExpr",
            Expression::VariableExpr(_) => "VariableExpr",
            Expression::ElementAccess(_) => "ElementAccess",
            Expression::FuncCall(_) => "FuncCall",
        }
    }
}

impl<'de> de::EnumAccess<'de> for Expression {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let variant_name = self.variant_name();

        seed.deserialize(variant_name.into_deserializer())
            .map(|variant| (variant, self))
    }
}

impl<'de> de::VariantAccess<'de> for Expression {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        de::Deserialize::deserialize(self)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self {
            Expression::TemplateExpr(expr) => seed.deserialize(expr.into_deserializer()),
            value => seed.deserialize(value.into_deserializer()),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.into_deserializer().deserialize_seq(visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.into_deserializer().deserialize_map(visitor)
    }
}

impl<'de> IntoDeserializer<'de, Error> for ElementAccess {
    type Deserializer = MapAccessDeserializer<ElementAccessAccess>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapAccessDeserializer::new(ElementAccessAccess::new(self))
    }
}

pub struct ElementAccessAccess {
    expr: Option<Expression>,
    operator: Option<ElementAccessOperator>,
}

impl ElementAccessAccess {
    fn new(access: ElementAccess) -> Self {
        ElementAccessAccess {
            expr: Some(access.expr),
            operator: Some(access.operator),
        }
    }
}

impl<'de> de::MapAccess<'de> for ElementAccessAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.expr.is_some() {
            seed.deserialize("expr".into_deserializer()).map(Some)
        } else if self.operator.is_some() {
            seed.deserialize("operator".into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(expr) = self.expr.take() {
            seed.deserialize(expr.into_deserializer())
        } else if let Some(operator) = self.operator.take() {
            seed.deserialize(operator.into_deserializer())
        } else {
            Err(de::Error::custom("invalid HCL element access"))
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for ElementAccessOperator {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for ElementAccessOperator {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
    }
    impl_deserialize_enum!();

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            ElementAccessOperator::AttrSplat => visitor.visit_str(".*"),
            ElementAccessOperator::FullSplat => visitor.visit_str("[*]"),
            ElementAccessOperator::GetAttr(ident) => {
                ident.into_deserializer().deserialize_any(visitor)
            }
            ElementAccessOperator::Index(expr) => expr.into_deserializer().deserialize_any(visitor),
            ElementAccessOperator::LegacyIndex(index) => visitor.visit_u64(index),
        }
    }
}

impl VariantName for ElementAccessOperator {
    fn variant_name(&self) -> &'static str {
        match self {
            ElementAccessOperator::AttrSplat => "AttrSplat",
            ElementAccessOperator::FullSplat => "FullSplat",
            ElementAccessOperator::GetAttr(_) => "GetAttr",
            ElementAccessOperator::Index(_) => "Index",
            ElementAccessOperator::LegacyIndex(_) => "LegacyIndex",
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for FuncCall {
    type Deserializer = MapAccessDeserializer<FuncCallAccess>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapAccessDeserializer::new(FuncCallAccess::new(self))
    }
}

pub struct FuncCallAccess {
    name: Option<Identifier>,
    args: Option<Vec<Expression>>,
    variadic: Option<bool>,
}

impl FuncCallAccess {
    fn new(func_call: FuncCall) -> Self {
        FuncCallAccess {
            name: Some(func_call.name),
            args: Some(func_call.args),
            variadic: Some(func_call.variadic),
        }
    }
}

impl<'de> de::MapAccess<'de> for FuncCallAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.name.is_some() {
            seed.deserialize("name".into_deserializer()).map(Some)
        } else if self.args.is_some() {
            seed.deserialize("args".into_deserializer()).map(Some)
        } else if self.variadic.is_some() {
            seed.deserialize("variadic".into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(name) = self.name.take() {
            seed.deserialize(name.into_deserializer())
        } else if let Some(args) = self.args.take() {
            seed.deserialize(args.into_deserializer())
        } else if let Some(variadic) = self.variadic.take() {
            seed.deserialize(variadic.into_deserializer())
        } else {
            Err(de::Error::custom("invalid HCL function call"))
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for ObjectKey {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for ObjectKey {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
    }
    impl_deserialize_enum!();

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            ObjectKey::String(string) => visitor.visit_string(string),
            ObjectKey::Identifier(ident) => ident.into_deserializer().deserialize_any(visitor),
            ObjectKey::RawExpression(expr) => expr.into_deserializer().deserialize_any(visitor),
        }
    }
}

impl VariantName for ObjectKey {
    fn variant_name(&self) -> &'static str {
        match self {
            ObjectKey::String(_) => "String",
            ObjectKey::Identifier(_) => "Identifier",
            ObjectKey::RawExpression(_) => "RawExpression",
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for RawExpression {
    type Deserializer = NewtypeStructDeserializer<String>;

    fn into_deserializer(self) -> Self::Deserializer {
        NewtypeStructDeserializer::new(self.into_inner())
    }
}

impl<'de> IntoDeserializer<'de, Error> for TemplateExpr {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for TemplateExpr {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
    }
    impl_deserialize_enum!();

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            TemplateExpr::QuotedString(string) => visitor.visit_string(string),
            TemplateExpr::Heredoc(heredoc) => heredoc.into_deserializer().deserialize_any(visitor),
        }
    }
}

impl VariantName for TemplateExpr {
    fn variant_name(&self) -> &'static str {
        match self {
            TemplateExpr::QuotedString(_) => "QuotedString",
            TemplateExpr::Heredoc(_) => "Heredoc",
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for Heredoc {
    type Deserializer = MapAccessDeserializer<HeredocAccess>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapAccessDeserializer::new(HeredocAccess::new(self))
    }
}

pub struct HeredocAccess {
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

impl<'de> IntoDeserializer<'de, Error> for HeredocStripMode {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for HeredocStripMode {
    type Error = Error;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
    }
    impl_deserialize_enum!();

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            HeredocStripMode::None => visitor.visit_str("None"),
            HeredocStripMode::Indent => visitor.visit_str("Indent"),
        }
    }
}

impl VariantName for HeredocStripMode {
    fn variant_name(&self) -> &'static str {
        match self {
            HeredocStripMode::None => "None",
            HeredocStripMode::Indent => "Indent",
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for Identifier {
    type Deserializer = NewtypeStructDeserializer<String>;

    fn into_deserializer(self) -> Self::Deserializer {
        NewtypeStructDeserializer::new(self.into_inner())
    }
}

pub struct NewtypeStructDeserializer<T, E = Error> {
    value: T,
    marker: PhantomData<E>,
}

impl<T, E> NewtypeStructDeserializer<T, E> {
    fn new(value: T) -> Self {
        NewtypeStructDeserializer {
            value,
            marker: PhantomData,
        }
    }
}

impl<'de, T, E> de::Deserializer<'de> for NewtypeStructDeserializer<T, E>
where
    T: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self.value.into_deserializer())
    }
}

pub struct EnumAccess<T, E = Error> {
    value: T,
    marker: PhantomData<E>,
}

impl<T, E> EnumAccess<T, E> {
    fn new(value: T) -> Self {
        EnumAccess {
            value,
            marker: PhantomData,
        }
    }
}

impl<'de, T, E> de::EnumAccess<'de> for EnumAccess<T, E>
where
    T: IntoDeserializer<'de, E> + VariantName,
    E: de::Error,
{
    type Error = E;
    type Variant = VariantAccess<T, E>;

    fn variant_seed<S>(self, seed: S) -> Result<(S::Value, Self::Variant), Self::Error>
    where
        S: de::DeserializeSeed<'de>,
    {
        let variant_name = self.value.variant_name();

        seed.deserialize(variant_name.into_deserializer())
            .map(|variant| (variant, VariantAccess::new(self.value)))
    }
}

pub struct VariantAccess<T, E = Error> {
    value: T,
    marker: PhantomData<E>,
}

impl<T, E> VariantAccess<T, E> {
    fn new(value: T) -> Self {
        VariantAccess {
            value,
            marker: PhantomData,
        }
    }
}

impl<'de, T, E> de::VariantAccess<'de> for VariantAccess<T, E>
where
    T: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    fn unit_variant(self) -> Result<(), Self::Error> {
        de::Deserialize::deserialize(self.value.into_deserializer())
    }

    fn newtype_variant_seed<S>(self, seed: S) -> Result<S::Value, Self::Error>
    where
        S: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.value.into_deserializer())
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.value.into_deserializer().deserialize_seq(visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.value.into_deserializer().deserialize_map(visitor)
    }
}

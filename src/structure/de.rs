//! Deserialize impls for HCL structure types.

use super::*;
use crate::{Error, Result};
use serde::de::value::{MapAccessDeserializer, StrDeserializer};
use serde::de::{self, IntoDeserializer};
use serde::{forward_to_deserialize_any, Deserializer};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

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
            Expression::Number(n) => n.deserialize_any(visitor),
            Expression::String(s) => visitor.visit_string(s),
            Expression::Array(array) => visitor.visit_seq(array.into_deserializer()),
            Expression::Object(object) => visitor.visit_map(object.into_deserializer()),
            Expression::Raw(expr) => expr.into_deserializer().deserialize_any(visitor),
            Expression::TemplateExpr(expr) => expr.into_deserializer().deserialize_any(visitor),
            Expression::VariableExpr(expr) => expr.into_deserializer().deserialize_any(visitor),
            Expression::Traversal(traversal) => {
                traversal.into_deserializer().deserialize_any(visitor)
            }
            Expression::FuncCall(func_call) => {
                func_call.into_deserializer().deserialize_any(visitor)
            }
            Expression::SubExpr(expr) => expr.into_deserializer().deserialize_any(visitor),
            Expression::Conditional(cond) => cond.into_deserializer().deserialize_any(visitor),
            Expression::Operation(op) => op.into_deserializer().deserialize_any(visitor),
            Expression::ForExpr(expr) => expr.into_deserializer().deserialize_any(visitor),
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
            Expression::Traversal(_) => "Traversal",
            Expression::FuncCall(_) => "FuncCall",
            Expression::SubExpr(_) => "SubExpr",
            Expression::Conditional(_) => "Conditional",
            Expression::Operation(_) => "Operation",
            Expression::ForExpr(_) => "ForExpr",
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
            Expression::SubExpr(expr) => seed.deserialize(expr.into_deserializer()),
            Expression::Operation(op) => seed.deserialize(op.into_deserializer()),
            Expression::ForExpr(expr) => seed.deserialize(expr.into_deserializer()),
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

impl<'de> IntoDeserializer<'de, Error> for Traversal {
    type Deserializer = MapAccessDeserializer<TraversalAccess>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapAccessDeserializer::new(TraversalAccess::new(self))
    }
}

pub struct TraversalAccess {
    expr: Option<Expression>,
    operators: Option<Vec<TraversalOperator>>,
}

impl TraversalAccess {
    fn new(traversal: Traversal) -> Self {
        TraversalAccess {
            expr: Some(traversal.expr),
            operators: Some(traversal.operators),
        }
    }
}

impl<'de> de::MapAccess<'de> for TraversalAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.expr.is_some() {
            seed.deserialize("expr".into_deserializer()).map(Some)
        } else if self.operators.is_some() {
            seed.deserialize("operators".into_deserializer()).map(Some)
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
        } else if let Some(operators) = self.operators.take() {
            seed.deserialize(operators.into_deserializer())
        } else {
            Err(de::Error::custom("invalid HCL element access"))
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for TraversalOperator {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for TraversalOperator {
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
            TraversalOperator::AttrSplat => visitor.visit_str(".*"),
            TraversalOperator::FullSplat => visitor.visit_str("[*]"),
            TraversalOperator::GetAttr(ident) => ident.into_deserializer().deserialize_any(visitor),
            TraversalOperator::Index(expr) => expr.into_deserializer().deserialize_any(visitor),
            TraversalOperator::LegacyIndex(index) => visitor.visit_u64(index),
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

impl VariantName for TraversalOperator {
    fn variant_name(&self) -> &'static str {
        match self {
            TraversalOperator::AttrSplat => "AttrSplat",
            TraversalOperator::FullSplat => "FullSplat",
            TraversalOperator::GetAttr(_) => "GetAttr",
            TraversalOperator::Index(_) => "Index",
            TraversalOperator::LegacyIndex(_) => "LegacyIndex",
        }
    }
}

impl<'de> de::EnumAccess<'de> for TraversalOperator {
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

impl<'de> de::VariantAccess<'de> for TraversalOperator {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        de::Deserialize::deserialize(self)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self {
            TraversalOperator::Index(expr) => seed.deserialize(expr.into_deserializer()),
            TraversalOperator::GetAttr(ident) => seed.deserialize(ident.into_deserializer()),
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

impl<'de> IntoDeserializer<'de, Error> for FuncCall {
    type Deserializer = MapAccessDeserializer<FuncCallAccess>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapAccessDeserializer::new(FuncCallAccess::new(self))
    }
}

pub struct FuncCallAccess {
    name: Option<Identifier>,
    args: Option<Vec<Expression>>,
    expand_final: Option<bool>,
}

impl FuncCallAccess {
    fn new(func_call: FuncCall) -> Self {
        FuncCallAccess {
            name: Some(func_call.name),
            args: Some(func_call.args),
            expand_final: Some(func_call.expand_final),
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
        } else if self.expand_final.is_some() {
            seed.deserialize("expand_final".into_deserializer())
                .map(Some)
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
        } else if let Some(expand_final) = self.expand_final.take() {
            seed.deserialize(expand_final.into_deserializer())
        } else {
            Err(de::Error::custom("invalid HCL function call"))
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for Conditional {
    type Deserializer = MapAccessDeserializer<ConditionalAccess>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapAccessDeserializer::new(ConditionalAccess::new(self))
    }
}

pub struct ConditionalAccess {
    cond_expr: Option<Expression>,
    true_expr: Option<Expression>,
    false_expr: Option<Expression>,
}

impl ConditionalAccess {
    fn new(cond: Conditional) -> Self {
        ConditionalAccess {
            cond_expr: Some(cond.cond_expr),
            true_expr: Some(cond.true_expr),
            false_expr: Some(cond.false_expr),
        }
    }
}

impl<'de> de::MapAccess<'de> for ConditionalAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.cond_expr.is_some() {
            seed.deserialize("cond_expr".into_deserializer()).map(Some)
        } else if self.true_expr.is_some() {
            seed.deserialize("true_expr".into_deserializer()).map(Some)
        } else if self.false_expr.is_some() {
            seed.deserialize("false_expr".into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(cond_expr) = self.cond_expr.take() {
            seed.deserialize(cond_expr.into_deserializer())
        } else if let Some(true_expr) = self.true_expr.take() {
            seed.deserialize(true_expr.into_deserializer())
        } else if let Some(false_expr) = self.false_expr.take() {
            seed.deserialize(false_expr.into_deserializer())
        } else {
            Err(de::Error::custom("invalid HCL condition"))
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for Operation {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for Operation {
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
            Operation::Unary(op) => op.into_deserializer().deserialize_any(visitor),
            Operation::Binary(op) => op.into_deserializer().deserialize_any(visitor),
        }
    }
}

impl VariantName for Operation {
    fn variant_name(&self) -> &'static str {
        match self {
            Operation::Unary(_) => "Unary",
            Operation::Binary(_) => "Binary",
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for UnaryOp {
    type Deserializer = MapAccessDeserializer<UnaryOpAccess>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapAccessDeserializer::new(UnaryOpAccess::new(self))
    }
}

pub struct UnaryOpAccess {
    operator: Option<UnaryOperator>,
    expr: Option<Expression>,
}

impl UnaryOpAccess {
    fn new(op: UnaryOp) -> Self {
        UnaryOpAccess {
            operator: Some(op.operator),
            expr: Some(op.expr),
        }
    }
}

impl<'de> de::MapAccess<'de> for UnaryOpAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.operator.is_some() {
            seed.deserialize("operator".into_deserializer()).map(Some)
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
        if let Some(operator) = self.operator.take() {
            seed.deserialize(operator.into_deserializer())
        } else if let Some(expr) = self.expr.take() {
            seed.deserialize(expr.into_deserializer())
        } else {
            Err(de::Error::custom("invalid HCL unary operation"))
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for UnaryOperator {
    type Deserializer = StrDeserializer<'static, Error>;

    fn into_deserializer(self) -> Self::Deserializer {
        self.as_str().into_deserializer()
    }
}

impl<'de> IntoDeserializer<'de, Error> for BinaryOp {
    type Deserializer = MapAccessDeserializer<BinaryOpAccess>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapAccessDeserializer::new(BinaryOpAccess::new(self))
    }
}

pub struct BinaryOpAccess {
    lhs_expr: Option<Expression>,
    operator: Option<BinaryOperator>,
    rhs_expr: Option<Expression>,
}

impl BinaryOpAccess {
    fn new(op: BinaryOp) -> Self {
        BinaryOpAccess {
            lhs_expr: Some(op.lhs_expr),
            operator: Some(op.operator),
            rhs_expr: Some(op.rhs_expr),
        }
    }
}

impl<'de> de::MapAccess<'de> for BinaryOpAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.lhs_expr.is_some() {
            seed.deserialize("lhs_expr".into_deserializer()).map(Some)
        } else if self.operator.is_some() {
            seed.deserialize("operator".into_deserializer()).map(Some)
        } else if self.rhs_expr.is_some() {
            seed.deserialize("rhs_expr".into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(lhs_expr) = self.lhs_expr.take() {
            seed.deserialize(lhs_expr.into_deserializer())
        } else if let Some(operator) = self.operator.take() {
            seed.deserialize(operator.into_deserializer())
        } else if let Some(rhs_expr) = self.rhs_expr.take() {
            seed.deserialize(rhs_expr.into_deserializer())
        } else {
            Err(de::Error::custom("invalid HCL binary operation"))
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for BinaryOperator {
    type Deserializer = StrDeserializer<'static, Error>;

    fn into_deserializer(self) -> Self::Deserializer {
        self.as_str().into_deserializer()
    }
}

impl<'de> IntoDeserializer<'de, Error> for ForExpr {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for ForExpr {
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
            ForExpr::List(expr) => expr.into_deserializer().deserialize_any(visitor),
            ForExpr::Object(expr) => expr.into_deserializer().deserialize_any(visitor),
        }
    }
}

impl VariantName for ForExpr {
    fn variant_name(&self) -> &'static str {
        match self {
            ForExpr::List(_) => "List",
            ForExpr::Object(_) => "Object",
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for ForListExpr {
    type Deserializer = MapAccessDeserializer<ForListExprAccess>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapAccessDeserializer::new(ForListExprAccess::new(self))
    }
}

pub struct ForListExprAccess {
    index_var: Option<Option<Identifier>>,
    value_var: Option<Identifier>,
    collection_expr: Option<Expression>,
    element_expr: Option<Expression>,
    cond_expr: Option<Option<Expression>>,
}

impl ForListExprAccess {
    fn new(expr: ForListExpr) -> Self {
        ForListExprAccess {
            index_var: Some(expr.index_var),
            value_var: Some(expr.value_var),
            collection_expr: Some(expr.collection_expr),
            element_expr: Some(expr.element_expr),
            cond_expr: Some(expr.cond_expr),
        }
    }
}

impl<'de> de::MapAccess<'de> for ForListExprAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.index_var.is_some() {
            seed.deserialize("key_var".into_deserializer()).map(Some)
        } else if self.value_var.is_some() {
            seed.deserialize("value_var".into_deserializer()).map(Some)
        } else if self.collection_expr.is_some() {
            seed.deserialize("collection_expr".into_deserializer())
                .map(Some)
        } else if self.element_expr.is_some() {
            seed.deserialize("element_expr".into_deserializer())
                .map(Some)
        } else if self.cond_expr.is_some() {
            seed.deserialize("cond_expr".into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(index_var) = self.index_var.take() {
            seed.deserialize(OptionDeserializer::new(index_var))
        } else if let Some(value_var) = self.value_var.take() {
            seed.deserialize(value_var.into_deserializer())
        } else if let Some(collection_expr) = self.collection_expr.take() {
            seed.deserialize(collection_expr.into_deserializer())
        } else if let Some(element_expr) = self.element_expr.take() {
            seed.deserialize(element_expr.into_deserializer())
        } else if let Some(cond_expr) = self.cond_expr.take() {
            seed.deserialize(OptionDeserializer::new(cond_expr))
        } else {
            Err(de::Error::custom("invalid HCL for list expression"))
        }
    }
}

impl<'de> IntoDeserializer<'de, Error> for ForObjectExpr {
    type Deserializer = MapAccessDeserializer<ForObjectExprAccess>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapAccessDeserializer::new(ForObjectExprAccess::new(self))
    }
}

pub struct ForObjectExprAccess {
    key_var: Option<Option<Identifier>>,
    value_var: Option<Identifier>,
    collection_expr: Option<Expression>,
    key_expr: Option<Expression>,
    value_expr: Option<Expression>,
    grouping: Option<bool>,
    cond_expr: Option<Option<Expression>>,
}

impl ForObjectExprAccess {
    fn new(expr: ForObjectExpr) -> Self {
        ForObjectExprAccess {
            key_var: Some(expr.key_var),
            value_var: Some(expr.value_var),
            collection_expr: Some(expr.collection_expr),
            key_expr: Some(expr.key_expr),
            value_expr: Some(expr.value_expr),
            grouping: Some(expr.grouping),
            cond_expr: Some(expr.cond_expr),
        }
    }
}

impl<'de> de::MapAccess<'de> for ForObjectExprAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.key_var.is_some() {
            seed.deserialize("key_var".into_deserializer()).map(Some)
        } else if self.value_var.is_some() {
            seed.deserialize("value_var".into_deserializer()).map(Some)
        } else if self.collection_expr.is_some() {
            seed.deserialize("collection_expr".into_deserializer())
                .map(Some)
        } else if self.key_expr.is_some() {
            seed.deserialize("key_expr".into_deserializer()).map(Some)
        } else if self.value_expr.is_some() {
            seed.deserialize("value_expr".into_deserializer()).map(Some)
        } else if self.grouping.is_some() {
            seed.deserialize("grouping".into_deserializer()).map(Some)
        } else if self.cond_expr.is_some() {
            seed.deserialize("cond_expr".into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(key_var) = self.key_var.take() {
            seed.deserialize(OptionDeserializer::new(key_var))
        } else if let Some(value_var) = self.value_var.take() {
            seed.deserialize(value_var.into_deserializer())
        } else if let Some(collection_expr) = self.collection_expr.take() {
            seed.deserialize(collection_expr.into_deserializer())
        } else if let Some(key_expr) = self.key_expr.take() {
            seed.deserialize(key_expr.into_deserializer())
        } else if let Some(value_expr) = self.value_expr.take() {
            seed.deserialize(value_expr.into_deserializer())
        } else if let Some(grouping) = self.grouping.take() {
            seed.deserialize(grouping.into_deserializer())
        } else if let Some(cond_expr) = self.cond_expr.take() {
            seed.deserialize(OptionDeserializer::new(cond_expr))
        } else {
            Err(de::Error::custom("invalid HCL for object expression"))
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

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            ObjectKey::Identifier(ident) => ident.into_deserializer().deserialize_any(visitor),
            ObjectKey::Expression(expr) => expr.into_deserializer().deserialize_any(visitor),
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

impl VariantName for ObjectKey {
    fn variant_name(&self) -> &'static str {
        match self {
            ObjectKey::Identifier(_) => "Identifier",
            ObjectKey::Expression(_) => "Expression",
        }
    }
}

impl<'de> de::EnumAccess<'de> for ObjectKey {
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

impl<'de> de::VariantAccess<'de> for ObjectKey {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        de::Deserialize::deserialize(self)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self {
            ObjectKey::Expression(expr) => seed.deserialize(expr.into_deserializer()),
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
    type Deserializer = StrDeserializer<'static, Error>;

    fn into_deserializer(self) -> Self::Deserializer {
        self.as_str().into_deserializer()
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

pub struct OptionDeserializer<T, E = Error> {
    value: Option<T>,
    marker: PhantomData<E>,
}

impl<T, E> OptionDeserializer<T, E> {
    fn new(value: Option<T>) -> Self {
        OptionDeserializer {
            value,
            marker: PhantomData,
        }
    }
}

impl<'de, T, E> de::Deserializer<'de> for OptionDeserializer<T, E>
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
        match self.value {
            Some(value) => visitor.visit_some(value.into_deserializer()),
            None => visitor.visit_none(),
        }
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

pub struct FromStrVisitor<T> {
    expecting: &'static str,
    marker: PhantomData<T>,
}

impl<T> FromStrVisitor<T> {
    pub fn new(expecting: &'static str) -> FromStrVisitor<T> {
        FromStrVisitor {
            expecting,
            marker: PhantomData,
        }
    }
}

impl<'de, T> de::Visitor<'de> for FromStrVisitor<T>
where
    T: FromStr,
    T::Err: de::Error,
{
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.expecting)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        FromStr::from_str(value).map_err(de::Error::custom)
    }
}

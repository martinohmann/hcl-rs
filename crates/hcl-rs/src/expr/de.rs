//! Deserialize impls for HCL structure types.

use super::*;
use crate::de::{EnumAccess, FromStrVisitor, OptionDeserializer, VariantName};
use crate::Error;
use serde::de::value::{MapAccessDeserializer, StrDeserializer, StringDeserializer};
use serde::de::{self, Expected, IntoDeserializer, Unexpected, VariantAccess};
use serde::{forward_to_deserialize_any, Deserializer};

impl Expression {
    #[cold]
    fn invalid_type<E>(&self, exp: &dyn Expected) -> E
    where
        E: de::Error,
    {
        de::Error::invalid_type(self.unexpected(), exp)
    }

    #[cold]
    fn unexpected(&self) -> Unexpected {
        match self {
            Expression::Null => Unexpected::Unit,
            Expression::Bool(b) => Unexpected::Bool(*b),
            Expression::Number(n) => n.unexpected(),
            Expression::String(s) => Unexpected::Str(s),
            Expression::Array(_) => Unexpected::Seq,
            Expression::Object(_) => Unexpected::Map,
            other => Unexpected::Other(other.variant_name()),
        }
    }
}

impl<'de> de::Deserialize<'de> for Expression {
    #[allow(clippy::too_many_lines)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        enum Field {
            Null,
            Bool,
            Number,
            String,
            Array,
            Object,
            TemplateExpr,
            Variable,
            Traversal,
            FuncCall,
            Parenthesis,
            Conditional,
            Operation,
            ForExpr,
            Raw,
        }

        struct FieldVisitor;

        impl<'de> de::Visitor<'de> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an HCL expression variant identifier")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    0u64 => Ok(Field::Null),
                    1u64 => Ok(Field::Bool),
                    2u64 => Ok(Field::Number),
                    3u64 => Ok(Field::String),
                    4u64 => Ok(Field::Array),
                    5u64 => Ok(Field::Object),
                    6u64 => Ok(Field::TemplateExpr),
                    7u64 => Ok(Field::Variable),
                    8u64 => Ok(Field::Traversal),
                    9u64 => Ok(Field::FuncCall),
                    10u64 => Ok(Field::Parenthesis),
                    11u64 => Ok(Field::Conditional),
                    12u64 => Ok(Field::Operation),
                    13u64 => Ok(Field::ForExpr),
                    14u64 => Ok(Field::Raw),
                    _ => Err(de::Error::invalid_value(
                        Unexpected::Unsigned(value),
                        &"variant index 0 <= i < 15",
                    )),
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "Null" => Ok(Field::Null),
                    "Bool" => Ok(Field::Bool),
                    "Number" => Ok(Field::Number),
                    "String" => Ok(Field::String),
                    "Array" => Ok(Field::Array),
                    "Object" => Ok(Field::Object),
                    "TemplateExpr" => Ok(Field::TemplateExpr),
                    "Variable" => Ok(Field::Variable),
                    "Traversal" => Ok(Field::Traversal),
                    "FuncCall" => Ok(Field::FuncCall),
                    "Parenthesis" => Ok(Field::Parenthesis),
                    "Conditional" => Ok(Field::Conditional),
                    "Operation" => Ok(Field::Operation),
                    "ForExpr" => Ok(Field::ForExpr),
                    "Raw" => Ok(Field::Raw),
                    _ => Err(de::Error::unknown_variant(value, VARIANTS)),
                }
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    b"Null" => Ok(Field::Null),
                    b"Bool" => Ok(Field::Bool),
                    b"Number" => Ok(Field::Number),
                    b"String" => Ok(Field::String),
                    b"Array" => Ok(Field::Array),
                    b"Object" => Ok(Field::Object),
                    b"TemplateExpr" => Ok(Field::TemplateExpr),
                    b"Variable" => Ok(Field::Variable),
                    b"Traversal" => Ok(Field::Traversal),
                    b"FuncCall" => Ok(Field::FuncCall),
                    b"Parenthesis" => Ok(Field::Parenthesis),
                    b"Conditional" => Ok(Field::Conditional),
                    b"Operation" => Ok(Field::Operation),
                    b"ForExpr" => Ok(Field::ForExpr),
                    b"Raw" => Ok(Field::Raw),
                    _ => {
                        let value = &String::from_utf8_lossy(value);
                        Err(de::Error::unknown_variant(value, VARIANTS))
                    }
                }
            }
        }

        impl<'de> de::Deserialize<'de> for Field {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ExpressionVisitor;

        impl<'de> de::Visitor<'de> for ExpressionVisitor {
            type Value = Expression;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an HCL expression")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                Ok(Expression::Bool(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
                Ok(Expression::Number(value.into()))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
                Ok(Expression::Number(value.into()))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E> {
                Ok(Number::from_f64(value).map_or(Expression::Null, Expression::Number))
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(value.to_owned())
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                Ok(Expression::String(value))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> {
                Ok(Expression::Null)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> {
                Ok(Expression::Null)
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let mut vec = Vec::with_capacity(visitor.size_hint().unwrap_or(0));

                while let Some(elem) = visitor.next_element()? {
                    vec.push(elem);
                }

                Ok(Expression::Array(vec))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut map = Object::with_capacity(visitor.size_hint().unwrap_or(0));

                while let Some((key, value)) = visitor.next_entry()? {
                    map.insert(key, value);
                }

                Ok(Expression::Object(map))
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: de::EnumAccess<'de>,
            {
                match data.variant()? {
                    (Field::Null, v) => v.unit_variant().map(|()| Expression::Null),
                    (Field::Bool, v) => v.newtype_variant().map(Expression::Bool),
                    (Field::Number, v) => v.newtype_variant().map(Expression::Number),
                    (Field::String, v) => v.newtype_variant().map(Expression::String),
                    (Field::Array, v) => v.newtype_variant().map(Expression::Array),
                    (Field::Object, v) => v.newtype_variant().map(Expression::Object),
                    (Field::TemplateExpr, v) => v.newtype_variant().map(Expression::TemplateExpr),
                    (Field::Variable, v) => v.newtype_variant().map(Expression::Variable),
                    (Field::Traversal, v) => v.newtype_variant().map(Expression::Traversal),
                    (Field::FuncCall, v) => v.newtype_variant().map(Expression::FuncCall),
                    (Field::Parenthesis, v) => v.newtype_variant().map(Expression::Parenthesis),
                    (Field::Conditional, v) => v.newtype_variant().map(Expression::Conditional),
                    (Field::Operation, v) => v.newtype_variant().map(Expression::Operation),
                    (Field::ForExpr, v) => v.newtype_variant().map(Expression::ForExpr),
                    (Field::Raw, v) => v.newtype_variant().map(Expression::Raw),
                }
            }
        }

        const VARIANTS: &[&str] = &[
            "Null",
            "Bool",
            "Number",
            "String",
            "Array",
            "Object",
            "TemplateExpr",
            "Variable",
            "Traversal",
            "FuncCall",
            "Parenthesis",
            "Conditional",
            "Operation",
            "ForExpr",
            "Raw",
        ];

        deserializer.deserialize_enum("$hcl::Expression", VARIANTS, ExpressionVisitor)
    }
}

impl<'de> IntoDeserializer<'de, Error> for Expression {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

macro_rules! impl_deserialize_number {
    ($($method:ident)*) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: de::Visitor<'de>,
            {
                match self {
                    Expression::Number(n) => n.deserialize_any(visitor).map_err(de::Error::custom),
                    _ => Err(self.invalid_type(&visitor)),
                }
            }
        )*
    };
}

impl<'de> de::Deserializer<'de> for Expression {
    type Error = Error;

    impl_deserialize_number! { deserialize_i8 deserialize_i16 deserialize_i32 deserialize_i64 deserialize_i128 }
    impl_deserialize_number! { deserialize_u8 deserialize_u16 deserialize_u32 deserialize_u64 deserialize_u128 }
    impl_deserialize_number! { deserialize_f32 deserialize_f64 }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Expression::Null => visitor.visit_unit(),
            Expression::Bool(b) => visitor.visit_bool(b),
            Expression::Number(v) => v.deserialize_any(visitor).map_err(de::Error::custom),
            Expression::String(s) => visitor.visit_string(s),
            Expression::Array(v) => visitor.visit_seq(v.into_deserializer()),
            Expression::Object(v) => visitor.visit_map(v.into_deserializer()),
            Expression::TemplateExpr(v) => visitor.visit_string(v.to_string()),
            Expression::Parenthesis(v) => v.deserialize_any(visitor),
            other => other.deserialize_string(visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Expression::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if name == "$hcl::Expression" {
            return visitor.visit_enum(self);
        }

        match self {
            Expression::String(v) => visitor.visit_enum(v.into_deserializer()),
            Expression::Object(v) => {
                visitor.visit_enum(MapAccessDeserializer::new(v.into_deserializer()))
            }
            Expression::Operation(v) => visitor.visit_enum(EnumAccess::new(*v)),
            Expression::TemplateExpr(v) => visitor.visit_enum(EnumAccess::new(*v)),
            _ => Err(self.invalid_type(&"string, object, operation or template expression")),
        }
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Expression::Bool(v) => visitor.visit_bool(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Expression::String(v) => visitor.visit_string(v),
            Expression::TemplateExpr(v) => visitor.visit_string(v.to_string()),
            other => {
                let formatted = format::to_interpolated_string(&other)?;
                visitor.visit_string(formatted)
            }
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Expression::String(v) => visitor.visit_string(v),
            Expression::Array(v) => visitor.visit_seq(v.into_deserializer()),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Expression::Null => visitor.visit_unit(),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Expression::Array(v) => visitor.visit_seq(v.into_deserializer()),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Expression::Object(v) => visitor.visit_map(v.into_deserializer()),
            Expression::Conditional(v) => visitor.visit_map(ConditionalAccess::new(*v)),
            Expression::FuncCall(v) => visitor.visit_map(FuncCallAccess::new(*v)),
            Expression::ForExpr(v) => visitor.visit_map(ForExprAccess::new(*v)),
            Expression::Traversal(v) => visitor.visit_map(TraversalAccess::new(*v)),
            Expression::Operation(v) => v.deserialize_any(visitor),
            Expression::TemplateExpr(v) => v.deserialize_any(visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Expression::Array(v) => visitor.visit_seq(v.into_deserializer()),
            other => other.deserialize_map(visitor),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
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
        if self == Expression::Null {
            Ok(())
        } else {
            Err(self.invalid_type(&"unit variant"))
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self {
            Expression::Bool(v) => seed.deserialize(v.into_deserializer()),
            Expression::Number(v) => seed.deserialize(v).map_err(de::Error::custom),
            Expression::String(v) => seed.deserialize(v.into_deserializer()),
            Expression::Array(v) => seed.deserialize(v.into_deserializer()),
            Expression::Object(v) => seed.deserialize(v.into_deserializer()),
            Expression::Raw(v) => seed.deserialize(v.into_deserializer()),
            Expression::TemplateExpr(v) => seed.deserialize(*v),
            Expression::Variable(v) => seed.deserialize(v.into_deserializer()),
            Expression::Traversal(v) => seed.deserialize(v.into_deserializer()),
            Expression::FuncCall(v) => seed.deserialize(v.into_deserializer()),
            Expression::Parenthesis(v) => seed.deserialize(*v),
            Expression::Conditional(v) => seed.deserialize(v.into_deserializer()),
            Expression::Operation(v) => seed.deserialize(*v),
            Expression::ForExpr(v) => seed.deserialize(v.into_deserializer()),
            _ => Err(self.invalid_type(&"newtype variant")),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
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

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
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

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
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

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            TraversalOperator::AttrSplat | TraversalOperator::FullSplat => visitor.visit_unit(),
            TraversalOperator::GetAttr(ident) => visitor.visit_string(ident.into_inner()),
            TraversalOperator::Index(expr) => expr.deserialize_any(visitor),
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

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
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
            TraversalOperator::Index(expr) => seed.deserialize(expr),
            TraversalOperator::GetAttr(ident) => seed.deserialize(ident.into_deserializer()),
            value => seed.deserialize(value),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }
}

pub struct FuncCallAccess {
    name: Option<FuncName>,
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

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
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

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
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

pub struct FuncNameAccess {
    namespace: Option<Vec<Identifier>>,
    name: Option<Identifier>,
}

impl FuncNameAccess {
    fn new(func_name: FuncName) -> Self {
        FuncNameAccess {
            namespace: Some(func_name.namespace),
            name: Some(func_name.name),
        }
    }
}

impl<'de> de::MapAccess<'de> for FuncNameAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.namespace.is_some() {
            seed.deserialize("namespace".into_deserializer()).map(Some)
        } else if self.name.is_some() {
            seed.deserialize("name".into_deserializer()).map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(namespace) = self.namespace.take() {
            seed.deserialize(namespace.into_deserializer())
        } else if let Some(name) = self.name.take() {
            seed.deserialize(name.into_deserializer())
        } else {
            Err(de::Error::custom("invalid HCL function name"))
        }
    }
}

#[allow(clippy::struct_field_names)]
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

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
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

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
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

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Operation::Unary(op) => visitor.visit_map(UnaryOpAccess::new(op)),
            Operation::Binary(op) => visitor.visit_map(BinaryOpAccess::new(op)),
        }
    }

    impl_deserialize_enum!();

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
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

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
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

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(operator) = self.operator.take() {
            seed.deserialize(operator.into_deserializer())
                .map_err(de::Error::custom)
        } else if let Some(expr) = self.expr.take() {
            seed.deserialize(expr.into_deserializer())
        } else {
            Err(de::Error::custom("invalid HCL unary operation"))
        }
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

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
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

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        if let Some(lhs_expr) = self.lhs_expr.take() {
            seed.deserialize(lhs_expr.into_deserializer())
        } else if let Some(operator) = self.operator.take() {
            seed.deserialize(operator.into_deserializer())
                .map_err(de::Error::custom)
        } else if let Some(rhs_expr) = self.rhs_expr.take() {
            seed.deserialize(rhs_expr.into_deserializer())
        } else {
            Err(de::Error::custom("invalid HCL binary operation"))
        }
    }
}

pub struct ForExprAccess {
    key_var: Option<Option<Identifier>>,
    value_var: Option<Identifier>,
    collection_expr: Option<Expression>,
    key_expr: Option<Option<Expression>>,
    value_expr: Option<Expression>,
    grouping: Option<bool>,
    cond_expr: Option<Option<Expression>>,
}

impl ForExprAccess {
    fn new(expr: ForExpr) -> Self {
        ForExprAccess {
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

impl<'de> de::MapAccess<'de> for ForExprAccess {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
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

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
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
            seed.deserialize(OptionDeserializer::new(key_expr))
        } else if let Some(value_expr) = self.value_expr.take() {
            seed.deserialize(value_expr.into_deserializer())
        } else if let Some(grouping) = self.grouping.take() {
            seed.deserialize(grouping.into_deserializer())
        } else if let Some(cond_expr) = self.cond_expr.take() {
            seed.deserialize(OptionDeserializer::new(cond_expr))
        } else {
            Err(de::Error::custom("invalid HCL `for` expression"))
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

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            ObjectKey::Identifier(ident) => visitor.visit_string(ident.into_inner()),
            ObjectKey::Expression(expr) => expr.deserialize_any(visitor),
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

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
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
            ObjectKey::Expression(expr) => seed.deserialize(expr),
            value => seed.deserialize(value),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }
}

impl<'de> IntoDeserializer<'de, Error> for RawExpression {
    type Deserializer = StringDeserializer<Error>;

    fn into_deserializer(self) -> Self::Deserializer {
        self.into_inner().into_deserializer()
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

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            TemplateExpr::QuotedString(string) => visitor.visit_string(string),
            TemplateExpr::Heredoc(heredoc) => visitor.visit_map(HeredocAccess::new(heredoc)),
        }
    }

    impl_deserialize_enum!();

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct identifier ignored_any
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

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
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

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
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

impl<'de> IntoDeserializer<'de, Error> for Variable {
    type Deserializer = StringDeserializer<Error>;

    fn into_deserializer(self) -> Self::Deserializer {
        self.into_inner().into_deserializer()
    }
}

impl<'de> de::Deserialize<'de> for HeredocStripMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_any(FromStrVisitor::<Self>::new("a heredoc strip mode"))
    }
}

impl<'de> IntoDeserializer<'de, Error> for HeredocStripMode {
    type Deserializer = StrDeserializer<'static, Error>;

    fn into_deserializer(self) -> Self::Deserializer {
        self.as_str().into_deserializer()
    }
}

impl_variant_name! {
    Expression => {
        Null, Bool, Number, String, Array, Object, Raw, TemplateExpr, Variable,
        Traversal, FuncCall, Parenthesis, Conditional, Operation, ForExpr
    },
    ObjectKey => { Identifier, Expression },
    Operation => { Unary, Binary },
    TemplateExpr => { QuotedString, Heredoc },
    TraversalOperator => { AttrSplat, FullSplat, GetAttr, Index, LegacyIndex }
}

impl_into_map_access_deserializer! {
    BinaryOp => BinaryOpAccess,
    Conditional => ConditionalAccess,
    ForExpr => ForExprAccess,
    FuncCall => FuncCallAccess,
    FuncName => FuncNameAccess,
    Heredoc => HeredocAccess,
    Traversal => TraversalAccess,
    UnaryOp => UnaryOpAccess
}

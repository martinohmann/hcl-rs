//! Serializer impls for HCL expression types.

#[cfg(test)]
mod tests;

use super::*;
use crate::ser::{in_internal_serialization, InternalHandles, SerializeInternalHandleStruct};
use crate::{format, Error, Identifier, Number, Result};
use serde::ser::{self, Impossible, SerializeMap};
use std::fmt;

const EXPR_HANDLE_MARKER: &str = "\x00$hcl::ExprHandle";

thread_local! {
    static EXPR_HANDLES: InternalHandles<Expression> = InternalHandles::new(EXPR_HANDLE_MARKER);
}

macro_rules! impl_serialize_for_expr {
    ($($ty:ty)*) => {
        $(
            impl ser::Serialize for $ty {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: ser::Serializer,
                {
                    if in_internal_serialization() {
                        EXPR_HANDLES.with(|eh| eh.serialize(self.clone(), serializer))
                    } else {
                        let s = format::to_interpolated_string(self).map_err(ser::Error::custom)?;
                        serializer.serialize_str(&s)
                    }
                }
            }
        )*
    };
}

impl_serialize_for_expr! {
    Conditional ForExpr FuncCall Operation UnaryOp BinaryOp
    TemplateExpr Heredoc RawExpression Traversal Variable
}

impl ser::Serialize for HeredocStripMode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl ser::Serialize for Expression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if in_internal_serialization() {
            return EXPR_HANDLES.with(|eh| eh.serialize(self.clone(), serializer));
        }

        match self {
            Expression::Null => serializer.serialize_unit(),
            Expression::Bool(b) => serializer.serialize_bool(*b),
            Expression::Number(num) => num.serialize(serializer),
            Expression::String(s) => serializer.serialize_str(s),
            Expression::Array(array) => array.serialize(serializer),
            Expression::Object(object) => object.serialize(serializer),
            Expression::Parenthesis(expr) => expr.serialize(serializer),
            Expression::TemplateExpr(expr) => expr.serialize(serializer),
            Expression::FuncCall(func_call) => func_call.serialize(serializer),
            Expression::Variable(var) => var.serialize(serializer),
            Expression::Traversal(traversal) => traversal.serialize(serializer),
            Expression::Conditional(cond) => cond.serialize(serializer),
            Expression::Operation(op) => op.serialize(serializer),
            Expression::ForExpr(expr) => expr.serialize(serializer),
            Expression::Raw(raw) => raw.serialize(serializer),
        }
    }
}

pub(crate) struct ExpressionSerializer;

impl ser::Serializer for ExpressionSerializer {
    type Ok = Expression;
    type Error = Error;

    type SerializeSeq = SerializeExpressionSeq;
    type SerializeTuple = SerializeExpressionSeq;
    type SerializeTupleStruct = SerializeExpressionSeq;
    type SerializeTupleVariant = SerializeExpressionTupleVariant;
    type SerializeMap = SerializeExpressionMap;
    type SerializeStruct = SerializeExpressionStruct;
    type SerializeStructVariant = SerializeExpressionStructVariant;

    serialize_self! { some newtype_struct }
    forward_to_serialize_seq! { tuple tuple_struct }

    fn serialize_bool(self, value: bool) -> Result<Self::Ok> {
        Ok(Expression::Bool(value))
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok> {
        Ok(Expression::Number(value.into()))
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok> {
        self.serialize_u64(value as u64)
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok> {
        self.serialize_u64(value as u64)
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok> {
        self.serialize_u64(value as u64)
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        Ok(Expression::Number(value.into()))
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok> {
        self.serialize_f64(value as f64)
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok> {
        Ok(Number::from_f64(value).map_or(Expression::Null, Expression::Number))
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        Ok(Expression::String(value.to_string()))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        Ok(Expression::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        let vec = value
            .iter()
            .map(|&b| Expression::Number(b.into()))
            .collect();
        Ok(Expression::Array(vec))
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        Ok(Expression::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        let mut object = Object::with_capacity(1);
        object.insert(
            ObjectKey::Identifier(Identifier::new(variant)?),
            value.serialize(self)?,
        );
        Ok(Expression::Object(object))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeExpressionSeq::new(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SerializeExpressionTupleVariant::new(variant, len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeExpressionMap::new(len))
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeExpressionStruct::new(name, len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeExpressionStructVariant::new(variant, len))
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + fmt::Display,
    {
        Ok(Expression::String(value.to_string()))
    }
}

pub(crate) struct SerializeExpressionSeq {
    vec: Vec<Expression>,
}

impl SerializeExpressionSeq {
    fn new(len: Option<usize>) -> Self {
        SerializeExpressionSeq {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        }
    }
}

impl ser::SerializeSeq for SerializeExpressionSeq {
    type Ok = Expression;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.vec.push(value.serialize(ExpressionSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Expression::Array(self.vec))
    }
}

impl ser::SerializeTuple for SerializeExpressionSeq {
    impl_forward_to_serialize_seq!(serialize_element, Expression);
}

impl ser::SerializeTupleStruct for SerializeExpressionSeq {
    impl_forward_to_serialize_seq!(serialize_field, Expression);
}

pub(crate) struct SerializeExpressionTupleVariant {
    name: ObjectKey,
    vec: Vec<Expression>,
}

impl SerializeExpressionTupleVariant {
    pub(crate) fn new(variant: &'static str, len: usize) -> Self {
        SerializeExpressionTupleVariant {
            name: ObjectKey::from(variant),
            vec: Vec::with_capacity(len),
        }
    }
}

impl ser::SerializeTupleVariant for SerializeExpressionTupleVariant {
    type Ok = Expression;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.vec.push(value.serialize(ExpressionSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        let mut object = Object::with_capacity(1);
        object.insert(self.name, self.vec.into());
        Ok(Expression::Object(object))
    }
}

pub(crate) struct SerializeExpressionMap {
    map: Object<ObjectKey, Expression>,
    next_key: Option<ObjectKey>,
}

impl SerializeExpressionMap {
    pub(crate) fn new(len: Option<usize>) -> Self {
        SerializeExpressionMap {
            map: Object::with_capacity(len.unwrap_or(0)),
            next_key: None,
        }
    }
}

impl ser::SerializeMap for SerializeExpressionMap {
    type Ok = Expression;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.next_key = Some(key.serialize(ObjectKeySerializer)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        let key = self.next_key.take();
        let key = key.expect("serialize_value called before serialize_key");
        let expr = value.serialize(ExpressionSerializer)?;
        self.map.insert(key, expr);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Expression::Object(self.map))
    }
}

pub(crate) enum SerializeExpressionStruct {
    InternalHandle(SerializeInternalHandleStruct),
    Map(SerializeExpressionMap),
}

impl SerializeExpressionStruct {
    pub(crate) fn new(name: &'static str, len: usize) -> Self {
        if name == EXPR_HANDLE_MARKER {
            SerializeExpressionStruct::InternalHandle(SerializeInternalHandleStruct::new())
        } else {
            SerializeExpressionStruct::Map(SerializeExpressionMap::new(Some(len)))
        }
    }
}

impl ser::SerializeStruct for SerializeExpressionStruct {
    type Ok = Expression;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match self {
            SerializeExpressionStruct::InternalHandle(ser) => ser.serialize_field(key, value),
            SerializeExpressionStruct::Map(ser) => ser.serialize_entry(key, value),
        }
    }

    fn end(self) -> Result<Self::Ok> {
        match self {
            SerializeExpressionStruct::InternalHandle(ser) => ser
                .end()
                .map(|handle| EXPR_HANDLES.with(|eh| eh.remove(handle))),
            SerializeExpressionStruct::Map(ser) => ser.end(),
        }
    }
}

pub(crate) struct SerializeExpressionStructVariant {
    name: ObjectKey,
    map: Object<ObjectKey, Expression>,
}

impl SerializeExpressionStructVariant {
    pub(crate) fn new(variant: &'static str, len: usize) -> Self {
        SerializeExpressionStructVariant {
            name: ObjectKey::from(variant),
            map: Object::with_capacity(len),
        }
    }
}

impl ser::SerializeStructVariant for SerializeExpressionStructVariant {
    type Ok = Expression;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        let expr = value.serialize(ExpressionSerializer)?;
        self.map.insert(ObjectKey::from(key), expr);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        let mut object = Object::with_capacity(1);
        object.insert(self.name, self.map.into());
        Ok(Expression::Object(object))
    }
}

pub(crate) struct ObjectKeySerializer;

impl ser::Serializer for ObjectKeySerializer {
    type Ok = ObjectKey;
    type Error = Error;

    type SerializeSeq = Impossible<ObjectKey, Error>;
    type SerializeTuple = Impossible<ObjectKey, Error>;
    type SerializeTupleStruct = Impossible<ObjectKey, Error>;
    type SerializeTupleVariant = Impossible<ObjectKey, Error>;
    type SerializeMap = Impossible<ObjectKey, Error>;
    type SerializeStruct = Impossible<ObjectKey, Error>;
    type SerializeStructVariant = Impossible<ObjectKey, Error>;

    serialize_unsupported! {
        bool f32 f64 bytes unit unit_struct none
        seq tuple tuple_struct tuple_variant map struct struct_variant
    }

    serialize_self! { some newtype_struct newtype_variant }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok> {
        Ok(ObjectKey::from(value))
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok> {
        Ok(ObjectKey::from(value))
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok> {
        Ok(ObjectKey::from(value))
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok> {
        Ok(ObjectKey::from(value))
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok> {
        Ok(ObjectKey::from(value))
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok> {
        Ok(ObjectKey::from(value))
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok> {
        Ok(ObjectKey::from(value))
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        Ok(ObjectKey::from(value))
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        Ok(ObjectKey::from(value.to_string()))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        Ok(ObjectKey::from(value))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        Identifier::new(variant).map(ObjectKey::Identifier)
    }
}

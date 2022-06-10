use crate::{serialize_unsupported, Error, Expression, Object, ObjectKey, Result};
use serde::ser::{self, Impossible};
use std::fmt::Display;

pub struct ExpressionSerializer;

impl ser::Serializer for ExpressionSerializer {
    type Ok = Expression;
    type Error = Error;

    type SerializeSeq = SerializeExpressionSeq;
    type SerializeTuple = SerializeExpressionSeq;
    type SerializeTupleStruct = SerializeExpressionSeq;
    type SerializeTupleVariant = SerializeExpressionTupleVariant;
    type SerializeMap = SerializeExpressionMap;
    type SerializeStruct = SerializeExpressionMap;
    type SerializeStructVariant = SerializeExpressionStructVariant;

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
        Ok(Expression::Number(value.into()))
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

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
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
        let mut object = Object::new();
        object.insert(ObjectKey::string(variant), to_expression(&value)?);
        Ok(Expression::Object(object))
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeExpressionSeq {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SerializeExpressionTupleVariant {
            name: variant.to_owned(),
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeExpressionMap {
            map: Object::new(),
            next_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeExpressionStructVariant {
            name: variant.to_owned(),
            map: Object::new(),
        })
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Display,
    {
        Ok(Expression::String(value.to_string()))
    }
}
pub struct SerializeExpressionSeq {
    vec: Vec<Expression>,
}

impl ser::SerializeSeq for SerializeExpressionSeq {
    type Ok = Expression;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.vec.push(to_expression(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Expression::Array(self.vec))
    }
}

impl ser::SerializeTuple for SerializeExpressionSeq {
    type Ok = Expression;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeExpressionSeq {
    type Ok = Expression;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeSeq::end(self)
    }
}

pub struct SerializeExpressionTupleVariant {
    name: String,
    vec: Vec<Expression>,
}

impl ser::SerializeTupleVariant for SerializeExpressionTupleVariant {
    type Ok = Expression;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.vec.push(to_expression(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        let mut object = Object::new();
        object.insert(ObjectKey::String(self.name), Expression::Array(self.vec));
        Ok(Expression::Object(object))
    }
}

pub struct SerializeExpressionMap {
    map: Object<ObjectKey, Expression>,
    next_key: Option<ObjectKey>,
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
        self.map.insert(key, to_expression(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Expression::Object(self.map))
    }
}

impl ser::SerializeStruct for SerializeExpressionMap {
    type Ok = Expression;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeMap::end(self)
    }
}

pub struct SerializeExpressionStructVariant {
    name: String,
    map: Object<ObjectKey, Expression>,
}

impl ser::SerializeStructVariant for SerializeExpressionStructVariant {
    type Ok = Expression;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.map
            .insert(ObjectKey::string(key), to_expression(&value)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        let mut object = Object::new();
        object.insert(ObjectKey::String(self.name), Expression::Object(self.map));
        Ok(Expression::Object(object))
    }
}

struct ObjectKeySerializer;

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
        bool f32 f64 bytes unit unit_struct newtype_variant none
        some seq tuple tuple_struct tuple_variant map struct struct_variant
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok> {
        Ok(ObjectKey::String(value.to_string()))
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok> {
        Ok(ObjectKey::String(value.to_string()))
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok> {
        Ok(ObjectKey::String(value.to_string()))
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok> {
        Ok(ObjectKey::String(value.to_string()))
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok> {
        Ok(ObjectKey::String(value.to_string()))
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok> {
        Ok(ObjectKey::String(value.to_string()))
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok> {
        Ok(ObjectKey::String(value.to_string()))
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        Ok(ObjectKey::String(value.to_string()))
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        Ok(ObjectKey::String(value.to_string()))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        Ok(ObjectKey::string(value))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        Ok(ObjectKey::identifier(variant))
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        value.serialize(self)
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Display,
    {
        Ok(ObjectKey::String(value.to_string()))
    }
}

/// Convert a `T` into `hcl::Expression` which is an enum that can represent any valid HCL
/// attribute value expression.
///
/// # Errors
///
/// This conversion can fail if `T`'s implementation of `Serialize` decides to
/// fail, or if `T` contains a map with non-string keys.
pub fn to_expression<T>(value: T) -> Result<Expression>
where
    T: ser::Serialize,
{
    value.serialize(ExpressionSerializer)
}

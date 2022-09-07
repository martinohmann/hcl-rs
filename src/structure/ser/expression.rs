use super::{template::TemplateExprSerializer, StringSerializer};
use crate::{serialize_unsupported, Error, Expression, Object, ObjectKey, RawExpression, Result};
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

    serialize_self! { some }
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

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        if name == "$hcl::raw_expression" {
            Ok(Expression::Raw(RawExpression::from(
                value.serialize(StringSerializer)?,
            )))
        } else {
            value.serialize(self)
        }
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        if name == "$hcl::expression" {
            value.serialize(self)
        } else if name == "$hcl::template_expr" {
            Ok(Expression::TemplateExpr(Box::new(
                TemplateExprSerializer.serialize_newtype_variant(
                    name,
                    variant_index,
                    variant,
                    value,
                )?,
            )))
        } else {
            let mut object = Object::new();
            object.insert(ObjectKey::identifier(variant), value.serialize(self)?);
            Ok(Expression::Object(object))
        }
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

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
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
        T: ?Sized + Display,
    {
        Ok(Expression::String(value.to_string()))
    }
}

pub struct SerializeExpressionSeq {
    vec: Vec<Expression>,
}

impl SerializeExpressionSeq {
    pub fn new(len: Option<usize>) -> Self {
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

pub struct SerializeExpressionTupleVariant {
    name: ObjectKey,
    vec: Vec<Expression>,
}

impl SerializeExpressionTupleVariant {
    pub fn new(variant: &'static str, len: usize) -> Self {
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
        let mut object = Object::new();
        object.insert(self.name, self.vec.into());
        Ok(Expression::Object(object))
    }
}

pub struct SerializeExpressionMap {
    map: Object<ObjectKey, Expression>,
    next_key: Option<ObjectKey>,
}

impl SerializeExpressionMap {
    pub fn new(len: Option<usize>) -> Self {
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
    name: ObjectKey,
    map: Object<ObjectKey, Expression>,
}

impl SerializeExpressionStructVariant {
    pub fn new(variant: &'static str, len: usize) -> Self {
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
        let mut object = Object::new();
        object.insert(self.name, self.map.into());
        Ok(Expression::Object(object))
    }
}

pub struct ObjectKeySerializer;

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

    serialize_self! { some newtype_struct }

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

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        // Specialization for the `ObjectKey` type itself.
        match (name, variant) {
            ("$hcl::object_key", "Identifier") => {
                Ok(ObjectKey::identifier(value.serialize(StringSerializer)?))
            }
            ("$hcl::object_key", "RawExpression") => Ok(ObjectKey::raw_expression(
                value.serialize(StringSerializer)?,
            )),
            (_, _) => value.serialize(self),
        }
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

use super::{ExpressionSerializer, StringSerializer};
use crate::{serialize_unsupported, Attribute, Error, Expression, Object, ObjectKey, Result};
use serde::ser::{self, Serialize};

pub struct AttributeSerializer;

impl ser::Serializer for AttributeSerializer {
    type Ok = Attribute;
    type Error = Error;

    type SerializeSeq = SerializeAttributeSeq;
    type SerializeTuple = SerializeAttributeSeq;
    type SerializeTupleStruct = SerializeAttributeSeq;
    type SerializeTupleVariant = SerializeAttributeTupleVariant;
    type SerializeMap = SerializeAttributeMap;
    type SerializeStruct = SerializeAttributeStruct;
    type SerializeStructVariant = SerializeAttributeStructVariant;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        char str bytes none unit unit_struct unit_variant
    }

    fn serialize_some<T>(self, value: &T) -> Result<Attribute>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Attribute>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Attribute>
    where
        T: ?Sized + Serialize,
    {
        Ok(Attribute::new(
            variant,
            value.serialize(ExpressionSerializer)?,
        ))
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeAttributeSeq::new())
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
        Ok(SerializeAttributeTupleVariant::new(variant, len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeAttributeMap::new())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeAttributeStruct::new())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeAttributeStructVariant::new(variant, len))
    }
}

pub struct SerializeAttributeSeq {
    key: Option<String>,
    expr: Option<Expression>,
}

impl SerializeAttributeSeq {
    pub fn new() -> Self {
        SerializeAttributeSeq {
            key: None,
            expr: None,
        }
    }
}

impl ser::SerializeSeq for SerializeAttributeSeq {
    type Ok = Attribute;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.key.is_none() {
            self.key = Some(value.serialize(StringSerializer)?);
        } else if self.expr.is_none() {
            self.expr = Some(value.serialize(ExpressionSerializer)?);
        } else {
            return Err(ser::Error::custom("expected sequence with 2 elements"));
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.key, self.expr) {
            (Some(key), Some(expr)) => Ok(Attribute::new(key, expr)),
            (_, _) => Err(ser::Error::custom("expected sequence with 2 elements")),
        }
    }
}

impl ser::SerializeTuple for SerializeAttributeSeq {
    type Ok = Attribute;
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

impl serde::ser::SerializeTupleStruct for SerializeAttributeSeq {
    type Ok = Attribute;
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

pub struct SerializeAttributeTupleVariant {
    key: String,
    vec: Vec<Expression>,
}

impl SerializeAttributeTupleVariant {
    pub fn new(variant: &'static str, len: usize) -> Self {
        SerializeAttributeTupleVariant {
            key: variant.to_owned(),
            vec: Vec::with_capacity(len),
        }
    }
}

impl ser::SerializeTupleVariant for SerializeAttributeTupleVariant {
    type Ok = Attribute;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.vec.push(value.serialize(ExpressionSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Attribute::new(self.key, self.vec))
    }
}

pub struct SerializeAttributeMap {
    key: Option<String>,
    expr: Option<Expression>,
}

impl SerializeAttributeMap {
    pub fn new() -> Self {
        SerializeAttributeMap {
            key: None,
            expr: None,
        }
    }
}

impl ser::SerializeMap for SerializeAttributeMap {
    type Ok = Attribute;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.key.is_none() {
            self.key = Some(key.serialize(StringSerializer)?);
            Ok(())
        } else {
            Err(ser::Error::custom("expected map with 1 entry"))
        }
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.key.is_none() {
            panic!("serialize_value called before serialize_key");
        }

        self.expr = Some(value.serialize(ExpressionSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.key, self.expr) {
            (Some(key), Some(expr)) => Ok(Attribute::new(key, expr)),
            (Some(_), None) => Err(ser::Error::custom("attribute value missing")),
            (_, _) => Err(ser::Error::custom("expected map with 1 entry")),
        }
    }
}

pub struct SerializeAttributeStruct {
    key: Option<String>,
    expr: Option<Expression>,
}

impl SerializeAttributeStruct {
    pub fn new() -> Self {
        SerializeAttributeStruct {
            key: None,
            expr: None,
        }
    }
}

impl ser::SerializeStruct for SerializeAttributeStruct {
    type Ok = Attribute;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match key {
            "key" => self.key = Some(value.serialize(StringSerializer)?),
            "expr" => self.expr = Some(value.serialize(ExpressionSerializer)?),
            _ => {
                return Err(ser::Error::custom(
                    "expected struct with fields `key` and `expr`",
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.key, self.expr) {
            (Some(key), Some(expr)) => Ok(Attribute::new(key, expr)),
            (Some(_), None) => Err(ser::Error::custom("`expr` field missing")),
            (_, _) => Err(ser::Error::custom(
                "expected struct with fields `key` and `expr`",
            )),
        }
    }
}

pub struct SerializeAttributeStructVariant {
    key: String,
    map: Object<ObjectKey, Expression>,
}

impl SerializeAttributeStructVariant {
    pub fn new(variant: &'static str, len: usize) -> Self {
        SerializeAttributeStructVariant {
            key: variant.to_owned(),
            map: Object::with_capacity(len),
        }
    }
}

impl ser::SerializeStructVariant for SerializeAttributeStructVariant {
    type Ok = Attribute;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        let expr = value.serialize(ExpressionSerializer)?;
        self.map.insert(ObjectKey::identifier(key), expr);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Attribute::new(self.key, self.map))
    }
}

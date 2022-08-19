use super::{
    attribute::{
        SerializeAttributeStruct, SerializeAttributeStructVariant, SerializeAttributeTupleVariant,
    },
    block::SerializeBlockStruct,
    expression::ExpressionSerializer,
    structure::StructureSerializer,
    IdentifierSerializer,
};
use crate::{serialize_unsupported, Attribute, Body, Error, Result, Structure};
use serde::ser::{self, Serialize};

pub struct BodySerializer;

impl ser::Serializer for BodySerializer {
    type Ok = Body;
    type Error = Error;

    type SerializeSeq = SerializeBodySeq;
    type SerializeTuple = SerializeBodySeq;
    type SerializeTupleStruct = SerializeBodySeq;
    type SerializeTupleVariant = SerializeBodyTupleVariant;
    type SerializeMap = SerializeBodyMap;
    type SerializeStruct = SerializeBodyStruct;
    type SerializeStructVariant = SerializeBodyStructVariant;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        char str bytes none unit unit_struct unit_variant
    }

    fn serialize_some<T>(self, value: &T) -> Result<Body>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Body>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Body>
    where
        T: ?Sized + Serialize,
    {
        match name {
            "$hcl::body" => value.serialize(self),
            _ => StructureSerializer
                .serialize_newtype_variant(name, variant_index, variant, value)
                .map(Into::into),
        }
    }

    /// A sequence of HCL attributes and blocks.
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeBodySeq::new(len))
    }

    /// A tuple of HCL attributes and blocks.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    /// A tuple of HCL attributes and blocks.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    /// Tuple variants are serialized as HCL attributes with an array value (`VARIANT = [...]`).
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SerializeBodyTupleVariant::new(variant, len))
    }

    /// Maps are serialized as sequences of HCL attributes (`KEY1 = VALUE1`).
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeBodyMap::new(len))
    }

    /// Structs have special handling for `hcl::Attribute` and `hcl::Block`. Attributes are
    /// serialized as key-expression pairs (`KEY = EXPR`), whereas blocks are serialized as block
    /// identifier, block labels (if any) and block body.
    ///
    /// Any other struct is serialized as a sequence of HCL attributes.
    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeBodyStruct::new(name, len))
    }

    /// Struct variants are serialized as HCL attributes with object value (`VARIANT = {...}`).
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeBodyStructVariant::new(variant, len))
    }
}

pub struct SerializeBodySeq {
    vec: Vec<Structure>,
}

impl SerializeBodySeq {
    pub fn new(len: Option<usize>) -> Self {
        SerializeBodySeq {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        }
    }
}

impl ser::SerializeSeq for SerializeBodySeq {
    type Ok = Body;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.vec.push(value.serialize(StructureSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Body(self.vec))
    }
}

impl ser::SerializeTuple for SerializeBodySeq {
    type Ok = Body;
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

impl serde::ser::SerializeTupleStruct for SerializeBodySeq {
    type Ok = Body;
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

pub struct SerializeBodyTupleVariant {
    inner: SerializeAttributeTupleVariant,
}

impl SerializeBodyTupleVariant {
    pub fn new(variant: &'static str, len: usize) -> Self {
        SerializeBodyTupleVariant {
            inner: SerializeAttributeTupleVariant::new(variant, len),
        }
    }
}

impl ser::SerializeTupleVariant for SerializeBodyTupleVariant {
    type Ok = Body;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.inner.serialize_field(value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.inner.end().map(Into::into)
    }
}

pub struct SerializeBodyMap {
    structures: Vec<Structure>,
    next_key: Option<String>,
}

impl SerializeBodyMap {
    pub fn new(len: Option<usize>) -> Self {
        SerializeBodyMap {
            structures: Vec::with_capacity(len.unwrap_or(0)),
            next_key: None,
        }
    }
}

impl ser::SerializeMap for SerializeBodyMap {
    type Ok = Body;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.next_key = Some(key.serialize(IdentifierSerializer)?.into_inner());
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        let key = self.next_key.take();
        let key = key.expect("serialize_value called before serialize_key");
        let expr = value.serialize(ExpressionSerializer)?;
        self.structures.push(Attribute::new(key, expr).into());
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Body(self.structures))
    }
}

impl ser::SerializeStruct for SerializeBodyMap {
    type Ok = Body;
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

pub enum SerializeBodyStruct {
    Attribute(SerializeAttributeStruct),
    Block(SerializeBlockStruct),
    Other(SerializeBodyMap),
}

impl SerializeBodyStruct {
    pub fn new(name: &'static str, len: usize) -> Self {
        match name {
            "$hcl::attribute" => SerializeBodyStruct::Attribute(SerializeAttributeStruct::new()),
            "$hcl::block" => SerializeBodyStruct::Block(SerializeBlockStruct::new()),
            _ => SerializeBodyStruct::Other(SerializeBodyMap::new(Some(len))),
        }
    }
}

impl ser::SerializeStruct for SerializeBodyStruct {
    type Ok = Body;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self {
            SerializeBodyStruct::Attribute(ser) => ser.serialize_field(key, value),
            SerializeBodyStruct::Block(ser) => ser.serialize_field(key, value),
            SerializeBodyStruct::Other(ser) => ser.serialize_field(key, value),
        }
    }

    fn end(self) -> Result<Body> {
        match self {
            SerializeBodyStruct::Attribute(ser) => ser.end().map(Into::into),
            SerializeBodyStruct::Block(ser) => ser.end().map(Into::into),
            SerializeBodyStruct::Other(ser) => ser.end(),
        }
    }
}

pub struct SerializeBodyStructVariant {
    inner: SerializeAttributeStructVariant,
}

impl SerializeBodyStructVariant {
    pub fn new(variant: &'static str, len: usize) -> Self {
        SerializeBodyStructVariant {
            inner: SerializeAttributeStructVariant::new(variant, len),
        }
    }
}

impl ser::SerializeStructVariant for SerializeBodyStructVariant {
    type Ok = Body;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.inner.serialize_field(key, value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.inner.end().map(Into::into)
    }
}

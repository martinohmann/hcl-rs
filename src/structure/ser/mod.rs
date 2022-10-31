//! Serializer impls for HCL structure types.

mod attribute;
mod block;
mod structure;
#[cfg(test)]
mod tests;

use self::{
    attribute::{
        AttributeSerializer, SerializeAttributeMap, SerializeAttributeSeq,
        SerializeAttributeStruct, SerializeAttributeStructVariant, SerializeAttributeTupleVariant,
    },
    block::{BlockSerializer, SerializeBlockSeq, SerializeBlockStruct},
    structure::{
        SerializeStructureStructVariant, SerializeStructureTupleVariant, StructureSerializer,
    },
};
use crate::expr::ser::ExpressionSerializer;
use crate::ser::IdentifierSerializer;
use crate::{Attribute, Body, Error, Identifier, Result, Structure};
use serde::ser::{self, Serialize, SerializeMap};

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
    serialize_self! { some newtype_struct }
    forward_to_serialize_seq! { tuple tuple_struct }

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

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeBodySeq::new(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SerializeBodyTupleVariant::new(
            Identifier::new(variant)?,
            len,
        ))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeBodyMap::new(len))
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeBodyStruct::new(name, len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeBodyStructVariant::new(
            Identifier::new(variant)?,
            len,
        ))
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
    impl_forward_to_serialize_seq!(serialize_element, Body);
}

impl ser::SerializeTupleStruct for SerializeBodySeq {
    impl_forward_to_serialize_seq!(serialize_field, Body);
}

pub struct SerializeBodyTupleVariant {
    inner: SerializeStructureTupleVariant,
}

impl SerializeBodyTupleVariant {
    pub fn new(key: Identifier, len: usize) -> Self {
        SerializeBodyTupleVariant {
            inner: SerializeStructureTupleVariant::new(key, len),
        }
    }
}

impl ser::SerializeTupleVariant for SerializeBodyTupleVariant {
    impl_forward_to_inner!(Body, serialize_field);
}

pub struct SerializeBodyMap {
    structures: Vec<Structure>,
    next_key: Option<Identifier>,
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
        self.next_key = Some(key.serialize(IdentifierSerializer)?);
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
            SerializeBodyStruct::Other(ser) => ser.serialize_entry(key, value),
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
    inner: SerializeStructureStructVariant,
}

impl SerializeBodyStructVariant {
    pub fn new(key: Identifier, len: usize) -> Self {
        SerializeBodyStructVariant {
            inner: SerializeStructureStructVariant::new(key, len),
        }
    }
}

impl ser::SerializeStructVariant for SerializeBodyStructVariant {
    impl_forward_to_inner!(Body, serialize_field(key: &'static str));
}

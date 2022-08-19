//! Serializer impls for HCL structure types.

mod attribute;
mod block;
mod body;
mod expression;
#[cfg(test)]
mod tests;

pub use self::{
    attribute::{
        AttributeSerializer, SerializeAttributeStruct, SerializeAttributeStructVariant,
        SerializeAttributeTupleVariant,
    },
    block::{BlockSerializer, SerializeBlockStruct},
    body::{
        BodySerializer, SerializeBodyMap, SerializeBodySeq, SerializeBodyStruct,
        SerializeBodyStructVariant, SerializeBodyTupleVariant,
    },
    expression::{to_expression, ExpressionSerializer},
};
use self::{
    attribute::{SerializeAttributeMap, SerializeAttributeSeq},
    block::SerializeBlockSeq,
};
use super::{Identifier, Structure};
use crate::{serialize_unsupported, Error, Result};
use serde::ser::{self, Impossible, Serialize, SerializeMap};
use std::fmt::Display;

pub struct IdentifierSerializer;

impl ser::Serializer for IdentifierSerializer {
    type Ok = Identifier;
    type Error = Error;

    type SerializeSeq = Impossible<Identifier, Error>;
    type SerializeTuple = Impossible<Identifier, Error>;
    type SerializeTupleStruct = Impossible<Identifier, Error>;
    type SerializeTupleVariant = Impossible<Identifier, Error>;
    type SerializeMap = Impossible<Identifier, Error>;
    type SerializeStruct = Impossible<Identifier, Error>;
    type SerializeStructVariant = Impossible<Identifier, Error>;

    serialize_unsupported! {
        i8 i16 i32 i64 u8 u16 u32 u64
        bool f32 f64 bytes unit unit_struct newtype_variant none
        some seq tuple tuple_struct tuple_variant map struct struct_variant
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        Ok(Identifier(value.to_string()))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        Ok(Identifier(value.to_owned()))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        Ok(Identifier(variant.to_owned()))
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
        Ok(Identifier(value.to_string()))
    }
}

pub struct StringSerializer;

impl ser::Serializer for StringSerializer {
    type Ok = String;
    type Error = Error;

    type SerializeSeq = Impossible<String, Error>;
    type SerializeTuple = Impossible<String, Error>;
    type SerializeTupleStruct = Impossible<String, Error>;
    type SerializeTupleVariant = Impossible<String, Error>;
    type SerializeMap = Impossible<String, Error>;
    type SerializeStruct = Impossible<String, Error>;
    type SerializeStructVariant = Impossible<String, Error>;

    serialize_unsupported! {
        bool f32 f64 bytes unit unit_struct newtype_variant none
        some seq tuple tuple_struct tuple_variant map struct struct_variant
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok> {
        Ok(value.to_string())
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok> {
        Ok(value.to_string())
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok> {
        Ok(value.to_string())
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok> {
        Ok(value.to_string())
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok> {
        Ok(value.to_string())
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok> {
        Ok(value.to_string())
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok> {
        Ok(value.to_string())
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        Ok(value.to_string())
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        Ok(value.to_string())
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        Ok(value.to_owned())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        Ok(variant.to_owned())
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
        Ok(value.to_string())
    }
}

pub struct StructureSerializer;

impl ser::Serializer for StructureSerializer {
    type Ok = Structure;
    type Error = Error;

    type SerializeSeq = SerializeStructureSeq;
    type SerializeTuple = SerializeStructureSeq;
    type SerializeTupleStruct = SerializeStructureSeq;
    type SerializeTupleVariant = SerializeStructureTupleVariant;
    type SerializeMap = SerializeStructureMap;
    type SerializeStruct = SerializeStructureStruct;
    type SerializeStructVariant = SerializeStructureStructVariant;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        char str bytes none unit unit_struct unit_variant
    }

    fn serialize_some<T>(self, value: &T) -> Result<Structure>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Structure>
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
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        // Specialization for the `Structure` type itself.
        match (name, variant) {
            ("$hcl::structure", "Block") => Ok(Structure::Block(value.serialize(BlockSerializer)?)),
            ("$hcl::structure", "Attribute") => {
                Ok(Structure::Attribute(value.serialize(AttributeSerializer)?))
            }
            (_, _) => AttributeSerializer
                .serialize_newtype_variant(name, variant_index, variant, value)
                .map(Into::into),
        }
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeStructureSeq::new(len))
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
        Ok(SerializeStructureTupleVariant::new(variant, len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeStructureMap::new())
    }

    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeStructureStruct::new(name))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeStructureStructVariant::new(variant, len))
    }
}

pub enum SerializeStructureSeq {
    Attribute(SerializeAttributeSeq),
    Block(SerializeBlockSeq),
}

impl SerializeStructureSeq {
    fn new(len: Option<usize>) -> Self {
        // Specialization for the `Block` type.
        if let Some(3) = len {
            SerializeStructureSeq::Block(SerializeBlockSeq::new())
        } else {
            SerializeStructureSeq::Attribute(SerializeAttributeSeq::new())
        }
    }
}

impl ser::SerializeSeq for SerializeStructureSeq {
    type Ok = Structure;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match self {
            SerializeStructureSeq::Attribute(attr) => attr.serialize_element(value),
            SerializeStructureSeq::Block(block) => block.serialize_element(value),
        }
    }

    fn end(self) -> Result<Self::Ok> {
        match self {
            SerializeStructureSeq::Attribute(attr) => attr.end().map(Structure::Attribute),
            SerializeStructureSeq::Block(block) => block.end().map(Structure::Block),
        }
    }
}

impl ser::SerializeTuple for SerializeStructureSeq {
    type Ok = Structure;
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

impl serde::ser::SerializeTupleStruct for SerializeStructureSeq {
    type Ok = Structure;
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

pub struct SerializeStructureTupleVariant {
    inner: SerializeAttributeTupleVariant,
}

impl SerializeStructureTupleVariant {
    fn new(variant: &'static str, len: usize) -> Self {
        SerializeStructureTupleVariant {
            inner: SerializeAttributeTupleVariant::new(variant, len),
        }
    }
}

impl ser::SerializeTupleVariant for SerializeStructureTupleVariant {
    type Ok = Structure;
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

pub struct SerializeStructureMap {
    inner: SerializeAttributeMap,
}

impl SerializeStructureMap {
    fn new() -> Self {
        SerializeStructureMap {
            inner: SerializeAttributeMap::new(),
        }
    }
}

impl ser::SerializeMap for SerializeStructureMap {
    type Ok = Structure;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.inner.serialize_key(key)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.inner.serialize_value(value)
    }

    fn end(self) -> Result<Self::Ok> {
        self.inner.end().map(Into::into)
    }
}

pub enum SerializeStructureStruct {
    Attribute(SerializeAttributeStruct),
    Block(SerializeBlockStruct),
    Other(SerializeAttributeMap),
}

impl SerializeStructureStruct {
    fn new(name: &'static str) -> Self {
        // Specialization for the `Attribute` and `Block` types.
        match name {
            "$hcl::attribute" => {
                SerializeStructureStruct::Attribute(SerializeAttributeStruct::new())
            }
            "$hcl::block" => SerializeStructureStruct::Block(SerializeBlockStruct::new()),
            _ => SerializeStructureStruct::Other(SerializeAttributeMap::new()),
        }
    }
}

impl ser::SerializeStruct for SerializeStructureStruct {
    type Ok = Structure;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match self {
            SerializeStructureStruct::Attribute(ser) => ser.serialize_field(key, value),
            SerializeStructureStruct::Block(ser) => ser.serialize_field(key, value),
            SerializeStructureStruct::Other(ser) => ser.serialize_entry(key, value),
        }
    }

    fn end(self) -> Result<Self::Ok> {
        match self {
            SerializeStructureStruct::Attribute(ser) => ser.end().map(Into::into),
            SerializeStructureStruct::Block(ser) => ser.end().map(Into::into),
            SerializeStructureStruct::Other(ser) => ser.end().map(Into::into),
        }
    }
}

pub struct SerializeStructureStructVariant {
    inner: SerializeAttributeStructVariant,
}

impl SerializeStructureStructVariant {
    fn new(variant: &'static str, len: usize) -> Self {
        SerializeStructureStructVariant {
            inner: SerializeAttributeStructVariant::new(variant, len),
        }
    }
}

impl ser::SerializeStructVariant for SerializeStructureStructVariant {
    type Ok = Structure;
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

pub struct SeqSerializer<S> {
    inner: S,
}

impl<S> SeqSerializer<S> {
    pub fn new(inner: S) -> Self {
        SeqSerializer { inner }
    }
}

impl<S> ser::Serializer for SeqSerializer<S>
where
    S: ser::Serializer + Clone,
{
    type Ok = Vec<S::Ok>;
    type Error = S::Error;

    type SerializeSeq = SerializeSeq<S>;
    type SerializeTuple = SerializeSeq<S>;
    type SerializeTupleStruct = SerializeSeq<S>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        char str bytes none unit newtype_variant unit_struct unit_variant
        tuple_variant map struct struct_variant
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq::new(self.inner, len))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }
}

pub struct SerializeSeq<S>
where
    S: ser::Serializer,
{
    inner: S,
    vec: Vec<S::Ok>,
}

impl<S> SerializeSeq<S>
where
    S: ser::Serializer,
{
    fn new(inner: S, len: Option<usize>) -> Self {
        SerializeSeq {
            inner,
            vec: Vec::with_capacity(len.unwrap_or(0)),
        }
    }
}

impl<S> ser::SerializeSeq for SerializeSeq<S>
where
    S: ser::Serializer + Clone,
{
    type Ok = Vec<S::Ok>;
    type Error = S::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        self.vec.push(value.serialize(self.inner.clone())?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.vec)
    }
}

impl<S> ser::SerializeTuple for SerializeSeq<S>
where
    S: ser::Serializer + Clone,
{
    type Ok = Vec<S::Ok>;
    type Error = S::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<S> serde::ser::SerializeTupleStruct for SerializeSeq<S>
where
    S: ser::Serializer + Clone,
{
    type Ok = Vec<S::Ok>;
    type Error = S::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

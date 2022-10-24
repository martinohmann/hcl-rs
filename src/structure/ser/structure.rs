use super::{attribute::*, block::*};
use crate::{Error, Result, Structure};
use serde::ser::{self, Serialize, SerializeMap};

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
    serialize_self! { some newtype_struct }
    forward_to_serialize_seq! { tuple tuple_struct }

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

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        SerializeStructureTupleVariant::new(variant, len)
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
        SerializeStructureStructVariant::new(variant, len)
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
            SerializeStructureSeq::Attribute(attr) => attr.end().map(Into::into),
            SerializeStructureSeq::Block(block) => block.end().map(Into::into),
        }
    }
}

impl ser::SerializeTuple for SerializeStructureSeq {
    impl_forward_to_serialize_seq!(serialize_element, Structure);
}

impl ser::SerializeTupleStruct for SerializeStructureSeq {
    impl_forward_to_serialize_seq!(serialize_field, Structure);
}

pub struct SerializeStructureTupleVariant {
    inner: SerializeAttributeTupleVariant,
}

impl SerializeStructureTupleVariant {
    pub fn new(variant: &'static str, len: usize) -> Result<Self> {
        Ok(SerializeStructureTupleVariant {
            inner: SerializeAttributeTupleVariant::new(variant, len)?,
        })
    }
}

impl ser::SerializeTupleVariant for SerializeStructureTupleVariant {
    impl_forward_to_inner!(Structure, serialize_field);
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
    impl_forward_to_inner!(Structure, serialize_key, serialize_value);
}

pub enum SerializeStructureStruct {
    Attribute(SerializeAttributeStruct),
    Block(SerializeBlockStruct),
    Other(SerializeStructureMap),
}

impl SerializeStructureStruct {
    fn new(name: &'static str) -> Self {
        // Specialization for the `Attribute` and `Block` types.
        match name {
            "$hcl::attribute" => {
                SerializeStructureStruct::Attribute(SerializeAttributeStruct::new())
            }
            "$hcl::block" => SerializeStructureStruct::Block(SerializeBlockStruct::new()),
            _ => SerializeStructureStruct::Other(SerializeStructureMap::new()),
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
            SerializeStructureStruct::Other(ser) => ser.end(),
        }
    }
}

pub struct SerializeStructureStructVariant {
    inner: SerializeAttributeStructVariant,
}

impl SerializeStructureStructVariant {
    pub fn new(variant: &'static str, len: usize) -> Result<Self> {
        Ok(SerializeStructureStructVariant {
            inner: SerializeAttributeStructVariant::new(variant, len)?,
        })
    }
}

impl ser::SerializeStructVariant for SerializeStructureStructVariant {
    impl_forward_to_inner!(Structure, serialize_field(key: &'static str));
}

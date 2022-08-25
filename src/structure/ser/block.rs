use super::{
    body::BodySerializer, expression::ExpressionSerializer, structure::StructureSerializer,
    SeqSerializer, StringSerializer,
};
use crate::{serialize_unsupported, Attribute, Block, BlockLabel, Body, Error, Result, Structure};
use serde::ser::{self, Impossible, Serialize};
use std::fmt::Display;

pub struct BlockSerializer;

impl ser::Serializer for BlockSerializer {
    type Ok = Block;
    type Error = Error;

    type SerializeSeq = SerializeBlockSeq;
    type SerializeTuple = SerializeBlockSeq;
    type SerializeTupleStruct = SerializeBlockSeq;
    type SerializeTupleVariant = SerializeBlockVariant;
    type SerializeMap = SerializeBlockMap;
    type SerializeStruct = SerializeBlockStruct;
    type SerializeStructVariant = SerializeBlockVariant;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        char str bytes none unit unit_struct unit_variant
    }
    serialize_self! { some newtype_struct }
    forward_to_serialize_seq! { tuple tuple_struct }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Block>
    where
        T: ?Sized + Serialize,
    {
        Ok(Block {
            identifier: variant.to_owned(),
            labels: Vec::new(),
            body: value.serialize(BodySerializer)?,
        })
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeBlockSeq::new())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SerializeBlockVariant::new(variant, len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeBlockMap::new())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeBlockStruct::new())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeBlockVariant::new(variant, len))
    }
}

pub struct SerializeBlockSeq {
    identifier: Option<String>,
    labels: Option<Vec<BlockLabel>>,
    body: Option<Body>,
}

impl SerializeBlockSeq {
    pub fn new() -> Self {
        SerializeBlockSeq {
            identifier: None,
            labels: None,
            body: None,
        }
    }
}

impl ser::SerializeSeq for SerializeBlockSeq {
    type Ok = Block;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.identifier.is_none() {
            self.identifier = Some(value.serialize(StringSerializer)?);
        } else if self.labels.is_none() {
            self.labels = Some(value.serialize(SeqSerializer::new(BlockLabelSerializer))?);
        } else if self.body.is_none() {
            self.body = Some(value.serialize(BodySerializer)?);
        } else {
            return Err(ser::Error::custom("expected sequence with 3 elements"));
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.identifier, self.labels, self.body) {
            (Some(ident), Some(labels), Some(body)) => Ok(Block::new(ident, labels, body)),
            (_, _, _) => Err(ser::Error::custom("expected sequence with 3 elements")),
        }
    }
}

impl ser::SerializeTuple for SerializeBlockSeq {
    type Ok = Block;
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

impl serde::ser::SerializeTupleStruct for SerializeBlockSeq {
    type Ok = Block;
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

pub struct SerializeBlockVariant {
    identifier: String,
    structures: Vec<Structure>,
}

impl SerializeBlockVariant {
    pub fn new(variant: &'static str, len: usize) -> Self {
        SerializeBlockVariant {
            identifier: variant.to_owned(),
            structures: Vec::with_capacity(len),
        }
    }
}

impl ser::SerializeTupleVariant for SerializeBlockVariant {
    type Ok = Block;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.structures.push(value.serialize(StructureSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        ser::SerializeStructVariant::end(self)
    }
}

impl ser::SerializeStructVariant for SerializeBlockVariant {
    type Ok = Block;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        let expr = value.serialize(ExpressionSerializer)?;
        self.structures.push(Attribute::new(key, expr).into());
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Block {
            identifier: self.identifier,
            labels: Vec::new(),
            body: Body(self.structures),
        })
    }
}

pub struct SerializeBlockMap {
    identifier: Option<String>,
    body: Option<Body>,
}

impl SerializeBlockMap {
    pub fn new() -> Self {
        SerializeBlockMap {
            identifier: None,
            body: None,
        }
    }
}

impl ser::SerializeMap for SerializeBlockMap {
    type Ok = Block;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.identifier.is_none() {
            self.identifier = Some(key.serialize(StringSerializer)?);
            Ok(())
        } else {
            Err(ser::Error::custom("expected map with 1 entry"))
        }
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.identifier.is_none() {
            panic!("serialize_value called before serialize_key");
        }

        self.body = Some(value.serialize(BodySerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.identifier, self.body) {
            (Some(ident), Some(body)) => Ok(Block::new(ident, Vec::<BlockLabel>::new(), body)),
            (_, _) => Err(ser::Error::custom("expected map with 1 entry")),
        }
    }
}

pub struct SerializeBlockStruct {
    identifier: Option<String>,
    labels: Option<Vec<BlockLabel>>,
    body: Option<Body>,
}

impl SerializeBlockStruct {
    pub fn new() -> Self {
        SerializeBlockStruct {
            identifier: None,
            labels: None,
            body: None,
        }
    }
}

impl ser::SerializeStruct for SerializeBlockStruct {
    type Ok = Block;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match key {
            "identifier" => self.identifier = Some(value.serialize(StringSerializer)?),
            "labels" => {
                self.labels = Some(value.serialize(SeqSerializer::new(BlockLabelSerializer))?)
            }
            "body" => self.body = Some(value.serialize(BodySerializer)?),
            _ => {
                return Err(ser::Error::custom(
                    "expected struct with fields `identifier`, `body` and optional `labels`",
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.identifier, self.body) {
            (Some(ident), Some(body)) => {
                Ok(Block::new(ident, self.labels.unwrap_or_default(), body))
            }
            (_, _) => Err(ser::Error::custom(
                "expected struct with fields `identifier`, `body` and optional `labels`",
            )),
        }
    }
}

#[derive(Clone)]
pub struct BlockLabelSerializer;

impl ser::Serializer for BlockLabelSerializer {
    type Ok = BlockLabel;
    type Error = Error;

    type SerializeSeq = Impossible<BlockLabel, Error>;
    type SerializeTuple = Impossible<BlockLabel, Error>;
    type SerializeTupleStruct = Impossible<BlockLabel, Error>;
    type SerializeTupleVariant = Impossible<BlockLabel, Error>;
    type SerializeMap = Impossible<BlockLabel, Error>;
    type SerializeStruct = Impossible<BlockLabel, Error>;
    type SerializeStructVariant = Impossible<BlockLabel, Error>;

    serialize_unsupported! {
        bool f32 f64 bytes unit unit_struct none
        seq tuple tuple_struct tuple_variant map struct struct_variant
    }
    serialize_self! { some newtype_struct }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok> {
        Ok(BlockLabel::String(value.to_string()))
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok> {
        Ok(BlockLabel::String(value.to_string()))
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok> {
        Ok(BlockLabel::String(value.to_string()))
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok> {
        Ok(BlockLabel::String(value.to_string()))
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok> {
        Ok(BlockLabel::String(value.to_string()))
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok> {
        Ok(BlockLabel::String(value.to_string()))
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok> {
        Ok(BlockLabel::String(value.to_string()))
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        Ok(BlockLabel::String(value.to_string()))
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        Ok(BlockLabel::String(value.to_string()))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        Ok(BlockLabel::String(value.to_string()))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        Ok(BlockLabel::identifier(variant))
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        // Specialization for the `BlockLabel` type itself.
        match (name, variant) {
            ("$hcl::block_label", "Identifier") => {
                Ok(BlockLabel::identifier(value.serialize(StringSerializer)?))
            }
            (_, _) => value.serialize(self),
        }
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Display,
    {
        Ok(BlockLabel::String(value.to_string()))
    }
}

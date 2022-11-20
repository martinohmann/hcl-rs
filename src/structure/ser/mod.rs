//! Serializer impls for HCL structure types.

#[cfg(test)]
mod tests;

use super::{Attribute, Block, BlockLabel, Body, Structure};
use crate::expr::ser::{
    ExpressionSerializer, SerializeExpressionMap, SerializeExpressionStruct,
    SerializeExpressionStructVariant, SerializeExpressionTupleVariant,
};
use crate::ser::{
    in_internal_serialization, IdentifierSerializer, InternalHandles,
    SerializeInternalHandleStruct, StringSerializer,
};
use crate::{Error, Expression, Identifier, Result};
use serde::ser::{self, Serialize, SerializeMap, SerializeStruct};
use std::fmt;

const BLOCK_MARKER: &str = "$hcl::Block";
const LABELED_BLOCK_MARKER: &str = "$hcl::LabeledBlock";

const STRUCTURE_HANDLE_MARKER: &str = "\x00$hcl::StructureHandle";

thread_local! {
    static STRUCTURE_HANDLES: InternalHandles<Structure> = InternalHandles::new(STRUCTURE_HANDLE_MARKER);
}

impl ser::Serialize for Attribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if in_internal_serialization() {
            STRUCTURE_HANDLES.with(|sh| sh.serialize(self.clone(), serializer))
        } else {
            let mut s = serializer.serialize_struct("Attribute", 2)?;
            s.serialize_field("key", &self.key)?;
            s.serialize_field("expr", &self.expr)?;
            s.end()
        }
    }
}

impl ser::Serialize for Block {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if in_internal_serialization() {
            STRUCTURE_HANDLES.with(|sh| sh.serialize(self.clone(), serializer))
        } else {
            let mut s = serializer.serialize_struct("Block", 3)?;
            s.serialize_field("identifier", &self.identifier)?;
            s.serialize_field("labels", &self.labels)?;
            s.serialize_field("body", &self.body)?;
            s.end()
        }
    }
}

impl ser::Serialize for Structure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if in_internal_serialization() {
            return STRUCTURE_HANDLES.with(|sh| sh.serialize(self.clone(), serializer));
        }

        match self {
            Structure::Attribute(attr) => attr.serialize(serializer),
            Structure::Block(block) => block.serialize(serializer),
        }
    }
}

pub(crate) enum Structures {
    Single(Structure),
    Multiple(Vec<Structure>),
}

impl From<Structures> for Body {
    fn from(value: Structures) -> Self {
        Body::from_iter(value)
    }
}

impl From<Attribute> for Structures {
    fn from(attr: Attribute) -> Self {
        Structures::Single(Structure::Attribute(attr))
    }
}

impl From<Block> for Structures {
    fn from(block: Block) -> Self {
        Structures::Single(Structure::Block(block))
    }
}

impl From<Vec<Structure>> for Structures {
    fn from(structures: Vec<Structure>) -> Self {
        Structures::Multiple(structures)
    }
}

impl IntoIterator for Structures {
    type Item = Structure;
    type IntoIter = std::vec::IntoIter<Structure>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Structures::Single(single) => vec![single].into_iter(),
            Structures::Multiple(multiple) => multiple.into_iter(),
        }
    }
}

pub(crate) struct BodySerializer;

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
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        let ident = Identifier::new(variant)?;

        value
            .serialize(StructureSerializer::new(ident))
            .map(Into::into)
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

pub(crate) struct SerializeBodySeq {
    vec: Vec<Structure>,
}

impl SerializeBodySeq {
    fn new(len: Option<usize>) -> Self {
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
        self.vec.extend(value.serialize(BodySerializer)?);
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

pub(crate) struct SerializeBodyTupleVariant {
    ident: Identifier,
    elements: Vec<Expression>,
}

impl SerializeBodyTupleVariant {
    fn new(ident: Identifier, len: usize) -> Self {
        SerializeBodyTupleVariant {
            ident,
            elements: Vec::with_capacity(len),
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
        self.elements.push(value.serialize(ExpressionSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Attribute::new(self.ident, self.elements).into())
    }
}

pub(crate) struct SerializeBodyMap {
    structures: Vec<Structure>,
    next_key: Option<Identifier>,
}

impl SerializeBodyMap {
    fn new(len: Option<usize>) -> Self {
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

        self.structures
            .extend(value.serialize(StructureSerializer::new(key))?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Body(self.structures))
    }
}

pub(crate) enum SerializeBodyStruct {
    InternalHandle(SerializeInternalHandleStruct),
    Map(SerializeBodyMap),
}

impl SerializeBodyStruct {
    fn new(name: &'static str, len: usize) -> Self {
        if name == STRUCTURE_HANDLE_MARKER {
            SerializeBodyStruct::InternalHandle(SerializeInternalHandleStruct::new())
        } else {
            SerializeBodyStruct::Map(SerializeBodyMap::new(Some(len)))
        }
    }
}

impl ser::SerializeStruct for SerializeBodyStruct {
    type Ok = Body;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match self {
            SerializeBodyStruct::Map(ser) => ser.serialize_entry(key, value),
            SerializeBodyStruct::InternalHandle(ser) => ser.serialize_field(key, value),
        }
    }

    fn end(self) -> Result<Self::Ok> {
        match self {
            SerializeBodyStruct::InternalHandle(ser) => ser
                .end()
                .map(|handle| STRUCTURE_HANDLES.with(|sh| sh.remove(handle)).into()),
            SerializeBodyStruct::Map(ser) => ser.end(),
        }
    }
}

pub(crate) struct SerializeBodyStructVariant {
    ident: Identifier,
    structures: Vec<Structure>,
}

impl SerializeBodyStructVariant {
    fn new(ident: Identifier, len: usize) -> Self {
        SerializeBodyStructVariant {
            ident,
            structures: Vec::with_capacity(len),
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
        let ident = Identifier::new(key)?;

        self.structures
            .extend(value.serialize(StructureSerializer::new(ident))?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Block::builder(self.ident)
            .add_structures(self.structures)
            .build()
            .into())
    }
}

pub(crate) struct StructureSerializer {
    ident: Identifier,
}

impl StructureSerializer {
    fn new(ident: Identifier) -> Self {
        StructureSerializer { ident }
    }

    fn into_attr(self, expr: Expression) -> Structures {
        Attribute::new(self.ident, expr).into()
    }
}

impl ser::Serializer for StructureSerializer {
    type Ok = Structures;
    type Error = Error;

    type SerializeSeq = SerializeStructureSeq;
    type SerializeTuple = SerializeStructureSeq;
    type SerializeTupleStruct = SerializeStructureSeq;
    type SerializeTupleVariant = SerializeStructureTupleVariant;
    type SerializeMap = SerializeStructureMap;
    type SerializeStruct = SerializeStructureStruct;
    type SerializeStructVariant = SerializeStructureStructVariant;

    serialize_self! { some }
    forward_to_serialize_seq! { tuple tuple_struct }

    fn serialize_bool(self, value: bool) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_bool(value)
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_i8(value)
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_i16(value)
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_i32(value)
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_i64(value)
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_u8(value)
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_u16(value)
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_u32(value)
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_u64(value)
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_f32(self, value: f32) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_f32(value)
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_f64(self, value: f64) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_f64(value)
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_char(value)
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_str(value)
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_bytes(value)
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_unit()
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_unit_struct(name)
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_unit_variant(name, variant_index, variant)
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + ser::Serialize,
    {
        if name == BLOCK_MARKER {
            BlockSerializer::new(self.ident).serialize_newtype_struct(name, value)
        } else if name == LABELED_BLOCK_MARKER {
            LabeledBlockSerializer::new(self.ident).serialize_newtype_struct(name, value)
        } else {
            ExpressionSerializer
                .serialize_newtype_struct(name, value)
                .map(|expr| self.into_attr(expr))
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
        if name == BLOCK_MARKER {
            BlockSerializer::new(self.ident).serialize_newtype_variant(
                name,
                variant_index,
                variant,
                value,
            )
        } else if name == LABELED_BLOCK_MARKER {
            LabeledBlockSerializer::new(self.ident).serialize_newtype_variant(
                name,
                variant_index,
                variant,
                value,
            )
        } else {
            ExpressionSerializer
                .serialize_newtype_variant(name, variant_index, variant, value)
                .map(|expr| self.into_attr(expr))
        }
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        ExpressionSerializer
            .serialize_none()
            .map(|expr| self.into_attr(expr))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeStructureSeq::new(self.ident, len))
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        SerializeStructureTupleVariant::new(self.ident, name, variant, len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeStructureMap::new(self.ident, len))
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeStructureStruct::new(self.ident, name, len))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        SerializeStructureStructVariant::new(self.ident, name, variant, len)
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + fmt::Display,
    {
        ExpressionSerializer
            .collect_str(value)
            .map(|expr| self.into_attr(expr))
    }
}

pub(crate) struct SerializeStructureSeq {
    ident: Identifier,
    elements: Vec<Expression>,
}

impl SerializeStructureSeq {
    fn new(ident: Identifier, len: Option<usize>) -> Self {
        SerializeStructureSeq {
            ident,
            elements: Vec::with_capacity(len.unwrap_or(0)),
        }
    }
}

impl ser::SerializeSeq for SerializeStructureSeq {
    type Ok = Structures;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.elements.push(value.serialize(ExpressionSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Attribute::new(self.ident, self.elements).into())
    }
}

impl ser::SerializeTuple for SerializeStructureSeq {
    impl_forward_to_serialize_seq!(serialize_element, Structures);
}

impl ser::SerializeTupleStruct for SerializeStructureSeq {
    impl_forward_to_serialize_seq!(serialize_field, Structures);
}

pub(crate) enum SerializeStructureTupleVariant {
    Attribute(Identifier, SerializeExpressionTupleVariant),
    Block(SerializeBlockTupleVariant),
    LabeledBlock(SerializeLabeledBlockTupleVariant),
}

impl SerializeStructureTupleVariant {
    fn new(
        ident: Identifier,
        name: &'static str,
        variant: &'static str,
        len: usize,
    ) -> Result<Self> {
        if name == BLOCK_MARKER {
            Ok(SerializeStructureTupleVariant::Block(
                SerializeBlockTupleVariant::new(ident, Identifier::new(variant)?, len),
            ))
        } else if name == LABELED_BLOCK_MARKER {
            Ok(SerializeStructureTupleVariant::LabeledBlock(
                SerializeLabeledBlockTupleVariant::new(ident, variant, len),
            ))
        } else {
            Ok(SerializeStructureTupleVariant::Attribute(
                ident,
                SerializeExpressionTupleVariant::new(variant, len),
            ))
        }
    }
}

impl ser::SerializeTupleVariant for SerializeStructureTupleVariant {
    type Ok = Structures;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match self {
            SerializeStructureTupleVariant::Attribute(_, ser) => ser.serialize_field(value),
            SerializeStructureTupleVariant::Block(ser) => ser.serialize_field(value),
            SerializeStructureTupleVariant::LabeledBlock(ser) => ser.serialize_field(value),
        }
    }

    fn end(self) -> Result<Self::Ok> {
        match self {
            SerializeStructureTupleVariant::Attribute(ident, ser) => {
                ser.end().map(|expr| Attribute::new(ident, expr).into())
            }
            SerializeStructureTupleVariant::Block(ser) => ser.end(),
            SerializeStructureTupleVariant::LabeledBlock(ser) => ser.end(),
        }
    }
}

pub(crate) struct SerializeStructureMap {
    ident: Identifier,
    inner: SerializeExpressionMap,
}

impl SerializeStructureMap {
    fn new(ident: Identifier, len: Option<usize>) -> Self {
        SerializeStructureMap {
            ident,
            inner: SerializeExpressionMap::new(len),
        }
    }
}

impl ser::SerializeMap for SerializeStructureMap {
    type Ok = Structures;
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
        self.inner
            .end()
            .map(|expr| Attribute::new(self.ident, expr).into())
    }
}

pub(crate) enum SerializeStructureStruct {
    Attribute(Identifier, SerializeExpressionStruct),
    Block(SerializeBlockStruct),
    LabeledBlock(SerializeLabeledBlockStruct),
}

impl SerializeStructureStruct {
    fn new(ident: Identifier, name: &'static str, len: usize) -> Self {
        if name == BLOCK_MARKER {
            SerializeStructureStruct::Block(SerializeBlockStruct::new(ident, len))
        } else if name == LABELED_BLOCK_MARKER {
            SerializeStructureStruct::LabeledBlock(SerializeLabeledBlockStruct::new(ident, len))
        } else {
            SerializeStructureStruct::Attribute(ident, SerializeExpressionStruct::new(name, len))
        }
    }
}

impl ser::SerializeStruct for SerializeStructureStruct {
    type Ok = Structures;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match self {
            SerializeStructureStruct::Attribute(_, ser) => ser.serialize_field(key, value),
            SerializeStructureStruct::Block(ser) => ser.serialize_field(key, value),
            SerializeStructureStruct::LabeledBlock(ser) => ser.serialize_field(key, value),
        }
    }

    fn end(self) -> Result<Self::Ok> {
        match self {
            SerializeStructureStruct::Attribute(ident, ser) => {
                ser.end().map(|expr| Attribute::new(ident, expr).into())
            }
            SerializeStructureStruct::Block(ser) => ser.end(),
            SerializeStructureStruct::LabeledBlock(ser) => ser.end(),
        }
    }
}

pub(crate) enum SerializeStructureStructVariant {
    Attribute(Identifier, SerializeExpressionStructVariant),
    Block(SerializeBlockStructVariant),
    LabeledBlock(SerializeLabeledBlockStructVariant),
}

impl SerializeStructureStructVariant {
    fn new(
        ident: Identifier,
        name: &'static str,
        variant: &'static str,
        len: usize,
    ) -> Result<Self> {
        if name == BLOCK_MARKER {
            Ok(SerializeStructureStructVariant::Block(
                SerializeBlockStructVariant::new(ident, Identifier::new(variant)?, len),
            ))
        } else if name == LABELED_BLOCK_MARKER {
            Ok(SerializeStructureStructVariant::LabeledBlock(
                SerializeLabeledBlockStructVariant::new(ident, variant, len),
            ))
        } else {
            Ok(SerializeStructureStructVariant::Attribute(
                ident,
                SerializeExpressionStructVariant::new(variant, len),
            ))
        }
    }
}

impl ser::SerializeStructVariant for SerializeStructureStructVariant {
    type Ok = Structures;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match self {
            SerializeStructureStructVariant::Attribute(_, ser) => ser.serialize_field(key, value),
            SerializeStructureStructVariant::Block(ser) => ser.serialize_field(key, value),
            SerializeStructureStructVariant::LabeledBlock(ser) => ser.serialize_field(key, value),
        }
    }

    fn end(self) -> Result<Self::Ok> {
        match self {
            SerializeStructureStructVariant::Attribute(ident, ser) => {
                ser.end().map(|expr| Attribute::new(ident, expr).into())
            }
            SerializeStructureStructVariant::Block(ser) => ser.end(),
            SerializeStructureStructVariant::LabeledBlock(ser) => ser.end(),
        }
    }
}

pub(crate) struct BlockSerializer {
    ident: Identifier,
}

impl BlockSerializer {
    fn new(ident: Identifier) -> Self {
        BlockSerializer { ident }
    }
}

impl ser::Serializer for BlockSerializer {
    type Ok = Structures;
    type Error = Error;

    type SerializeSeq = SerializeBlockSeq;
    type SerializeTuple = SerializeBlockSeq;
    type SerializeTupleStruct = SerializeBlockSeq;
    type SerializeTupleVariant = SerializeBlockTupleVariant;
    type SerializeMap = SerializeBlockMap;
    type SerializeStruct = SerializeBlockStruct;
    type SerializeStructVariant = SerializeBlockStructVariant;

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
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        let variant = Identifier::new(variant)?;

        Ok(Block::builder(self.ident)
            .add_structures(value.serialize(StructureSerializer::new(variant))?)
            .build()
            .into())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeBlockSeq::new(self.ident, len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SerializeBlockTupleVariant::new(
            self.ident,
            Identifier::new(variant)?,
            len,
        ))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeBlockMap::new(self.ident, len))
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeBlockStruct::new(self.ident, len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeBlockStructVariant::new(
            self.ident,
            Identifier::new(variant)?,
            len,
        ))
    }
}

pub(crate) struct SerializeBlockSeq {
    ident: Identifier,
    structures: Vec<Structure>,
}

impl SerializeBlockSeq {
    fn new(ident: Identifier, len: Option<usize>) -> Self {
        SerializeBlockSeq {
            ident,
            structures: Vec::with_capacity(len.unwrap_or(0)),
        }
    }
}

impl ser::SerializeSeq for SerializeBlockSeq {
    type Ok = Structures;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.structures
            .extend(value.serialize(StructureSerializer::new(self.ident.clone()))?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(self.structures.into())
    }
}

impl ser::SerializeTuple for SerializeBlockSeq {
    impl_forward_to_serialize_seq!(serialize_element, Structures);
}

impl ser::SerializeTupleStruct for SerializeBlockSeq {
    impl_forward_to_serialize_seq!(serialize_field, Structures);
}

pub(crate) struct SerializeBlockTupleVariant {
    ident: Identifier,
    variant: Identifier,
    structures: Vec<Structure>,
}

impl SerializeBlockTupleVariant {
    fn new(ident: Identifier, variant: Identifier, len: usize) -> Self {
        SerializeBlockTupleVariant {
            ident,
            variant,
            structures: Vec::with_capacity(len),
        }
    }
}

impl ser::SerializeTupleVariant for SerializeBlockTupleVariant {
    type Ok = Structures;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.structures
            .extend(value.serialize(StructureSerializer::new(self.variant.clone()))?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Block::builder(self.ident)
            .add_structures(self.structures)
            .build()
            .into())
    }
}

pub(crate) struct SerializeBlockMap {
    ident: Identifier,
    next_key: Option<Identifier>,
    structures: Vec<Structure>,
}

impl SerializeBlockMap {
    fn new(ident: Identifier, len: Option<usize>) -> Self {
        SerializeBlockMap {
            ident,
            next_key: None,
            structures: Vec::with_capacity(len.unwrap_or(0)),
        }
    }
}

impl ser::SerializeMap for SerializeBlockMap {
    type Ok = Structures;
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

        self.structures
            .extend(value.serialize(StructureSerializer::new(key))?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Block::builder(self.ident)
            .add_structures(self.structures)
            .build()
            .into())
    }
}

pub(crate) struct SerializeBlockStruct {
    ident: Identifier,
    structures: Vec<Structure>,
}

impl SerializeBlockStruct {
    fn new(ident: Identifier, len: usize) -> Self {
        SerializeBlockStruct {
            ident,
            structures: Vec::with_capacity(len),
        }
    }
}

impl ser::SerializeStruct for SerializeBlockStruct {
    type Ok = Structures;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        let ident = Identifier::new(key)?;

        self.structures
            .extend(value.serialize(StructureSerializer::new(ident))?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Block::builder(self.ident)
            .add_structures(self.structures)
            .build()
            .into())
    }
}

pub(crate) struct SerializeBlockStructVariant {
    ident: Identifier,
    variant: Identifier,
    structures: Vec<Structure>,
}

impl SerializeBlockStructVariant {
    fn new(ident: Identifier, variant: Identifier, len: usize) -> Self {
        SerializeBlockStructVariant {
            ident,
            variant,
            structures: Vec::with_capacity(len),
        }
    }
}

impl ser::SerializeStructVariant for SerializeBlockStructVariant {
    type Ok = Structures;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        let ident = Identifier::new(key)?;

        self.structures
            .extend(value.serialize(StructureSerializer::new(ident))?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(Block::builder(self.ident)
            .add_block(
                Block::builder(self.variant)
                    .add_structures(self.structures)
                    .build(),
            )
            .build()
            .into())
    }
}

pub(crate) struct LabeledBlockSerializer {
    ident: Identifier,
}

impl LabeledBlockSerializer {
    fn new(ident: Identifier) -> Self {
        LabeledBlockSerializer { ident }
    }
}

impl ser::Serializer for LabeledBlockSerializer {
    type Ok = Structures;
    type Error = Error;

    type SerializeSeq = SerializeBlockSeq;
    type SerializeTuple = SerializeBlockSeq;
    type SerializeTupleStruct = SerializeBlockSeq;
    type SerializeTupleVariant = SerializeLabeledBlockTupleVariant;
    type SerializeMap = SerializeLabeledBlockMap;
    type SerializeStruct = SerializeLabeledBlockStruct;
    type SerializeStructVariant = SerializeLabeledBlockStructVariant;

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
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        let mut structures = Vec::with_capacity(1);

        serialize_blocks(self.ident, value, &mut structures, |labels| {
            labels.insert(0, variant.into());
        })?;

        Ok(structures.into())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeBlockSeq::new(self.ident, len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SerializeLabeledBlockTupleVariant::new(
            self.ident, variant, len,
        ))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeLabeledBlockMap::new(self.ident, len))
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeLabeledBlockStruct::new(self.ident, len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(SerializeLabeledBlockStructVariant::new(
            self.ident, variant, len,
        ))
    }
}

pub(crate) struct SerializeLabeledBlockTupleVariant {
    ident: Identifier,
    variant: &'static str,
    structures: Vec<Structure>,
}

impl SerializeLabeledBlockTupleVariant {
    fn new(ident: Identifier, variant: &'static str, len: usize) -> Self {
        SerializeLabeledBlockTupleVariant {
            ident,
            variant,
            structures: Vec::with_capacity(len),
        }
    }
}

impl ser::SerializeTupleVariant for SerializeLabeledBlockTupleVariant {
    type Ok = Structures;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        serialize_blocks(self.ident.clone(), value, &mut self.structures, |labels| {
            labels.insert(0, self.variant.into());
        })
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(self.structures.into())
    }
}

pub(crate) struct SerializeLabeledBlockMap {
    ident: Identifier,
    next_key: Option<String>,
    structures: Vec<Structure>,
}

impl SerializeLabeledBlockMap {
    fn new(ident: Identifier, len: Option<usize>) -> Self {
        SerializeLabeledBlockMap {
            ident,
            next_key: None,
            structures: Vec::with_capacity(len.unwrap_or(0)),
        }
    }
}

impl ser::SerializeMap for SerializeLabeledBlockMap {
    type Ok = Structures;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        self.next_key = Some(key.serialize(StringSerializer)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        let key = self.next_key.take();
        let key = key.expect("serialize_value called before serialize_key");

        serialize_blocks(self.ident.clone(), value, &mut self.structures, |labels| {
            labels.insert(0, BlockLabel::from(&key));
        })
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(self.structures.into())
    }
}

pub(crate) struct SerializeLabeledBlockStruct {
    ident: Identifier,
    structures: Vec<Structure>,
}

impl SerializeLabeledBlockStruct {
    fn new(ident: Identifier, len: usize) -> Self {
        SerializeLabeledBlockStruct {
            ident,
            structures: Vec::with_capacity(len),
        }
    }
}

impl ser::SerializeStruct for SerializeLabeledBlockStruct {
    type Ok = Structures;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        serialize_blocks(self.ident.clone(), value, &mut self.structures, |labels| {
            labels.insert(0, key.into());
        })
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(self.structures.into())
    }
}

pub(crate) struct SerializeLabeledBlockStructVariant {
    ident: Identifier,
    variant: &'static str,
    structures: Vec<Structure>,
}

impl SerializeLabeledBlockStructVariant {
    fn new(ident: Identifier, variant: &'static str, len: usize) -> Self {
        SerializeLabeledBlockStructVariant {
            ident,
            variant,
            structures: Vec::with_capacity(len),
        }
    }
}

impl ser::SerializeStructVariant for SerializeLabeledBlockStructVariant {
    type Ok = Structures;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        serialize_blocks(self.ident.clone(), value, &mut self.structures, |labels| {
            labels.insert(0, self.variant.into());
            labels.insert(1, key.into());
        })
    }

    fn end(self) -> Result<Self::Ok> {
        Ok(self.structures.into())
    }
}

fn serialize_blocks<T, F>(
    ident: Identifier,
    value: &T,
    structures: &mut Vec<Structure>,
    mut f: F,
) -> Result<()>
where
    T: ?Sized + Serialize,
    F: FnMut(&mut Vec<BlockLabel>),
{
    for structure in value.serialize(StructureSerializer::new(ident))? {
        match structure {
            Structure::Attribute(attr) => {
                return Err(ser::Error::custom(format!(
                    "block expected, found attribute: {:?}",
                    attr
                )))
            }
            Structure::Block(mut block) => {
                f(&mut block.labels);
                structures.push(block.into());
            }
        }
    }

    Ok(())
}

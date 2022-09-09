//! Serializer impls for HCL structure types.

mod attribute;
mod block;
pub(crate) mod body;
mod conditional;
mod element_access;
mod expression;
mod func;
mod operation;
mod structure;
mod template;
#[cfg(test)]
mod tests;

pub use self::expression::to_expression;
use crate::{Error, Result};
use serde::ser::{self, Impossible};
use std::fmt::Display;

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
        i8 i16 i32 i64 u8 u16 u32 u64
        bool f32 f64 bytes unit unit_struct newtype_variant none
        seq tuple tuple_struct tuple_variant map struct struct_variant
    }
    serialize_self! { some newtype_struct }

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

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Display,
    {
        Ok(value.to_string())
    }
}

pub struct BoolSerializer;

impl ser::Serializer for BoolSerializer {
    type Ok = bool;
    type Error = Error;

    type SerializeSeq = Impossible<bool, Error>;
    type SerializeTuple = Impossible<bool, Error>;
    type SerializeTupleStruct = Impossible<bool, Error>;
    type SerializeTupleVariant = Impossible<bool, Error>;
    type SerializeMap = Impossible<bool, Error>;
    type SerializeStruct = Impossible<bool, Error>;
    type SerializeStructVariant = Impossible<bool, Error>;

    serialize_unsupported! {
        i8 i16 i32 i64 u8 u16 u32 u64
        f32 f64 char str bytes unit unit_struct unit_variant newtype_variant none
        seq tuple tuple_struct tuple_variant map struct struct_variant
    }
    serialize_self! { some newtype_struct }

    fn serialize_bool(self, value: bool) -> Result<Self::Ok> {
        Ok(value)
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
    serialize_self! { some newtype_struct }
    forward_to_serialize_seq! { tuple tuple_struct }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq::new(self.inner, len))
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
    impl_forward_to_serialize_seq!(serialize_element, Vec<S::Ok>, S::Error);
}

impl<S> serde::ser::SerializeTupleStruct for SerializeSeq<S>
where
    S: ser::Serializer + Clone,
{
    impl_forward_to_serialize_seq!(serialize_field, Vec<S::Ok>, S::Error);
}

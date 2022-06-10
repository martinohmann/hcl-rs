//! Serialize a Rust data structure into HCL data.
//!
//! This module provides the [`Serializer`] type and the convienince functions [`to_string`],
//! [`to_vec`] and [`to_writer`] for serializing data to HCL.
//!
//! ## Supported top-level types
//!
//! The [`Serializer`] supports serialization to HCL for types that are either structured like
//! maps or sequences of maps. For example, at the top level a struct with one or more named
//! fields is supported, while a newtype struct wrapping a primitive type like `u8` is not.
//!
//! Other example of supported top-level types:
//!
//! - tuple or newtype structs wrapping a map-like type
//! - enums with newtype or tuple variants wrapping map-like types, or struct variants
//!
//! Please note that these restrictions only apply to the top-level type that is serialized.
//! Nested fields can have any type that is serializable.
//!
//! ## Serializing a custom type
//!
//! The following example will serialize the data as a deeply nested HCL attribute.
//!
//! ```
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct User {
//!     age: u8,
//!     username: &'static str,
//!     email: &'static str,
//! }
//!
//! #[derive(Serialize)]
//! struct Data {
//!     users: Vec<User>,
//! }
//!
//! let data = Data {
//!     users: vec![
//!         User {
//!             age: 34,
//!             username: "johndoe",
//!             email: "johndoe@example.com",
//!         },
//!         User {
//!             age: 27,
//!             username: "janedoe",
//!             email: "janedoe@example.com",
//!         },
//!     ],
//! };
//!
//! let expected = r#"
//! users = [
//!   {
//!     "age" = 34
//!     "username" = "johndoe"
//!     "email" = "johndoe@example.com"
//!   },
//!   {
//!     "age" = 27
//!     "username" = "janedoe"
//!     "email" = "janedoe@example.com"
//!   }
//! ]
//! "#.trim_start();
//!
//! let serialized = hcl::to_string(&data)?;
//!
//! assert_eq!(serialized, expected);
//! #   Ok(())
//! # }
//! ```
//!
//! ## Serializing context-aware HCL
//!
//! If you need full control over the way data is serialized to HCL, you can make use of the [`Body`][Body] type which can be constructed using the builder pattern.
//!
//! The following example uses HCL blocks to format the same data from above in a different way.
//!
//! [Body]: ../struct.Body.html
//!
//! ```
//! # use std::error::Error;
//! #
//! # fn main() -> Result<(), Box<dyn Error>> {
//! use hcl::{Block, Body};
//!
//! let body = Body::builder()
//!     .add_block(
//!         Block::builder("user")
//!             .add_label("johndoe")
//!             .add_attribute(("age", 34))
//!             .add_attribute(("email", "johndoe@example.com"))
//!             .build(),
//!     )
//!     .add_block(
//!         Block::builder("user")
//!             .add_label("janedoe")
//!             .add_attribute(("age", 27))
//!             .add_attribute(("email", "janedoe@example.com"))
//!             .build(),
//!     )
//!     .build();
//!
//! let expected = r#"
//! user "johndoe" {
//!   age = 34
//!   email = "johndoe@example.com"
//! }
//!
//! user "janedoe" {
//!   age = 27
//!   email = "janedoe@example.com"
//! }
//! "#.trim_start();
//!
//! let serialized = hcl::to_string(&body)?;
//!
//! assert_eq!(serialized, expected);
//! #   Ok(())
//! # }
//! ```

mod escape;
mod format;
#[cfg(test)]
mod tests;

pub use self::format::{Format, PrettyFormatter, PrettyFormatterBuilder};
use crate::{Error, Result};
use serde::ser::{self, Impossible, Serialize, SerializeSeq};
use std::io;

/// A structure for serializing Rust values into HCL.
pub struct Serializer<W, F> {
    writer: W,
    formatter: F,
}

impl<'a, W> Serializer<W, PrettyFormatter<'a>>
where
    W: io::Write,
{
    /// Creates a new `Serializer` which serializes to the provides writer.
    pub fn new(writer: W) -> Self {
        Serializer::with_formatter(writer, PrettyFormatter::default())
    }
}

impl<W, F> Serializer<W, F>
where
    W: io::Write,
    F: Format,
{
    /// Creates a new `Serializer` which serializes to the provides writer using the provides
    /// formatter.
    pub fn with_formatter(writer: W, formatter: F) -> Serializer<W, F> {
        Serializer { writer, formatter }
    }

    /// Consumes `self` and returns the wrapped writer.
    pub fn into_inner(self) -> W {
        self.writer
    }

    fn serialize_attribute<K, V>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        self.serialize_attribute_key(key)?;
        self.serialize_attribute_value(value)?;
        self.formatter.end_attribute(&mut self.writer)?;
        Ok(())
    }

    fn serialize_attribute_key<K>(&mut self, key: &K) -> Result<()>
    where
        K: ?Sized + Serialize,
    {
        self.formatter.begin_attribute(&mut self.writer)?;
        key.serialize(IdentifierSerializer::new(self))
    }

    fn serialize_attribute_value<V>(&mut self, value: &V) -> Result<()>
    where
        V: ?Sized + Serialize,
    {
        self.formatter.begin_attribute_value(&mut self.writer)?;
        value.serialize(ExpressionSerializer::new(self))
    }

    fn serialize_array_value<V>(&mut self, value: &V) -> Result<()>
    where
        V: ?Sized + Serialize,
    {
        self.formatter.begin_array_value(&mut self.writer)?;
        value.serialize(ExpressionSerializer::new(self))?;
        self.formatter.end_array_value(&mut self.writer)?;
        Ok(())
    }

    fn serialize_object_key_value<K, V>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        self.serialize_object_key(key)?;
        self.serialize_object_value(value)
    }

    fn serialize_object_key<K>(&mut self, key: &K) -> Result<()>
    where
        K: ?Sized + Serialize,
    {
        self.formatter.begin_object_key(&mut self.writer)?;
        key.serialize(ObjectKeySerializer::new(self))
    }

    fn serialize_object_value<V>(&mut self, value: &V) -> Result<()>
    where
        V: ?Sized + Serialize,
    {
        self.formatter.begin_object_value(&mut self.writer)?;
        value.serialize(ExpressionSerializer::new(self))?;
        self.formatter.end_object_value(&mut self.writer)?;
        Ok(())
    }
}

impl<'a, W, F> ser::Serializer for &'a mut Serializer<W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = SerializeStruct<'a, W, F>;
    type SerializeStructVariant = Self;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        char str bytes none unit unit_struct unit_variant
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    /// Newtype variants have special handling for `hcl::Structure`. For this enum, the inner type
    /// is serialized, which is either `hcl::Attribute` or `hcl::Block`. These will be handled by
    /// `serialize_struct` below.
    ///
    /// Any other newtype variant is serialized as an HCL attribute (`VARIANT = VALUE`)
    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if name == "$hcl::structure" {
            value.serialize(self)
        } else {
            self.serialize_attribute(variant, value)
        }
    }

    /// A sequence of HCL attributes and blocks.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
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
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_attribute_key(variant)?;
        self.formatter.begin_attribute_value(&mut self.writer)?;
        self.formatter.begin_array(&mut self.writer)?;
        Ok(self)
    }

    /// Maps are serialized as sequences of HCL attributes (`KEY1 = VALUE1`).
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    /// Structs have special handling for `hcl::Attribute` and `hcl::Block`. Attributes are
    /// serialized as key-expression pairs (`KEY = EXPR`), whereas blocks are serialized as block
    /// identifier, block labels (if any) and block body.
    ///
    /// Any other struct is serialized as a sequence of HCL attributes.
    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        let kind = match name {
            "$hcl::attribute" => StructKind::Attribute,
            "$hcl::block" => StructKind::Block,
            _ => StructKind::Custom,
        };

        Ok(SerializeStruct::new(kind, self))
    }

    /// Struct variants are serialized as HCL attributes with object value (`VARIANT = {...}`).
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.serialize_attribute_key(variant)?;
        self.formatter.begin_attribute_value(&mut self.writer)?;
        self.formatter.begin_object(&mut self.writer)?;
        Ok(self)
    }
}

impl<'a, W, F> ser::SerializeSeq for &'a mut Serializer<W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeTuple for &'a mut Serializer<W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W, F> ser::SerializeTupleStruct for &'a mut Serializer<W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W, F> ser::SerializeTupleVariant for &'a mut Serializer<W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_array_value(value)
    }

    fn end(self) -> Result<()> {
        self.formatter.end_array(&mut self.writer)?;
        self.formatter.end_attribute(&mut self.writer)?;
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeMap for &'a mut Serializer<W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_attribute_key(key)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_attribute_value(value)?;
        self.formatter.end_attribute(&mut self.writer)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeStruct for &'a mut Serializer<W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_attribute(key, value)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeStructVariant for &'a mut Serializer<W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_attribute(key, value)
    }

    fn end(self) -> Result<()> {
        self.formatter.end_object(&mut self.writer)?;
        self.formatter.end_attribute(&mut self.writer)?;
        Ok(())
    }
}

struct IdentifierSerializer<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
}

impl<'a, W, F> IdentifierSerializer<'a, W, F> {
    fn new(ser: &'a mut Serializer<W, F>) -> IdentifierSerializer<'a, W, F> {
        IdentifierSerializer { ser }
    }
}

impl<'a, W, F> ser::Serializer for IdentifierSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        bytes none unit unit_struct unit_variant newtype_variant
        seq tuple tuple_struct tuple_variant
        map struct struct_variant
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.ser.formatter.write_ident(&mut self.ser.writer, v)?;
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }
}

struct RawExpressionSerializer<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
}

impl<'a, W, F> RawExpressionSerializer<'a, W, F> {
    fn new(ser: &'a mut Serializer<W, F>) -> RawExpressionSerializer<'a, W, F> {
        RawExpressionSerializer { ser }
    }
}

impl<'a, W, F> ser::Serializer for RawExpressionSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        bytes none unit unit_struct unit_variant newtype_variant
        seq tuple tuple_struct tuple_variant
        map struct struct_variant
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.ser.writer.write_all(v.as_bytes())?;
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }
}

struct ObjectKeySerializer<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
}

impl<'a, W, F> ObjectKeySerializer<'a, W, F> {
    fn new(ser: &'a mut Serializer<W, F>) -> ObjectKeySerializer<'a, W, F> {
        ObjectKeySerializer { ser }
    }
}

impl<'a, W, F> ser::Serializer for ObjectKeySerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        bytes none unit unit_struct unit_variant
        seq tuple tuple_struct tuple_variant
        map struct struct_variant
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.ser
            .formatter
            .write_quoted_string(&mut self.ser.writer, v)?;
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match name {
            "$hcl::identifier" => value.serialize(IdentifierSerializer::new(self.ser)),
            "$hcl::raw_expression" => {
                self.ser
                    .formatter
                    .begin_interpolated_string(&mut self.ser.writer)?;
                value.serialize(RawExpressionSerializer::new(self.ser))?;
                self.ser
                    .formatter
                    .end_interpolated_string(&mut self.ser.writer)?;
                Ok(())
            }
            _ => value.serialize(self),
        }
    }
}

struct BlockLabelSerializer<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
}

impl<'a, W, F> BlockLabelSerializer<'a, W, F> {
    fn new(ser: &'a mut Serializer<W, F>) -> BlockLabelSerializer<'a, W, F> {
        BlockLabelSerializer { ser }
    }
}

impl<'a, W, F> ser::Serializer for BlockLabelSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        bytes none unit unit_struct unit_variant
        tuple_struct tuple_variant
        map struct struct_variant
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.ser
            .formatter
            .write_quoted_string(&mut self.ser.writer, v)?;
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if name == "$hcl::identifier" {
            value.serialize(IdentifierSerializer::new(self.ser))
        } else {
            value.serialize(self)
        }
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }
}

impl<'a, W, F> ser::SerializeSeq for BlockLabelSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser.writer.write_all(b" ")?;
        value.serialize(BlockLabelSerializer::new(self.ser))
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeTuple for BlockLabelSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

struct ExpressionSerializer<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
}

impl<'a, W, F> ExpressionSerializer<'a, W, F> {
    fn new(ser: &'a mut Serializer<W, F>) -> ExpressionSerializer<'a, W, F> {
        ExpressionSerializer { ser }
    }
}

impl<'a, W, F> ser::Serializer for ExpressionSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.ser.formatter.write_bool(&mut self.ser.writer, v)?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.ser.formatter.write_int(&mut self.ser.writer, v)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.ser.formatter.write_int(&mut self.ser.writer, v)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.ser.formatter.write_float(&mut self.ser.writer, v)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.ser
            .formatter
            .write_quoted_string(&mut self.ser.writer, v)?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.ser.formatter.write_null(&mut self.ser.writer)?;
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if name == "$hcl::raw_expression" {
            value.serialize(RawExpressionSerializer::new(self.ser))
        } else {
            value.serialize(self)
        }
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if name == "$hcl::expression" {
            value.serialize(self)
        } else {
            self.ser.formatter.begin_object(&mut self.ser.writer)?;
            self.ser.serialize_object_key_value(variant, value)?;
            self.ser.formatter.end_object(&mut self.ser.writer)?;
            Ok(())
        }
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.ser.formatter.begin_array(&mut self.ser.writer)?;
        Ok(self)
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
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.ser.formatter.begin_object(&mut self.ser.writer)?;
        self.ser.serialize_object_key(variant)?;
        self.ser
            .formatter
            .begin_object_value(&mut self.ser.writer)?;
        self.ser.formatter.begin_array(&mut self.ser.writer)?;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.ser.formatter.begin_object(&mut self.ser.writer)?;
        Ok(self)
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
        self.ser.formatter.begin_object(&mut self.ser.writer)?;
        self.ser.serialize_object_key(variant)?;
        self.ser
            .formatter
            .begin_object_value(&mut self.ser.writer)?;
        self.ser.formatter.begin_object(&mut self.ser.writer)?;
        Ok(self)
    }
}

impl<'a, W, F> ser::SerializeSeq for ExpressionSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser.serialize_array_value(value)
    }

    fn end(self) -> Result<()> {
        self.ser.formatter.end_array(&mut self.ser.writer)?;
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeTuple for ExpressionSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W, F> ser::SerializeTupleStruct for ExpressionSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W, F> ser::SerializeTupleVariant for ExpressionSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        self.ser.formatter.end_array(&mut self.ser.writer)?;
        self.ser.formatter.end_object_value(&mut self.ser.writer)?;
        self.ser.formatter.end_object(&mut self.ser.writer)?;
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeMap for ExpressionSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser.serialize_object_key(key)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser.serialize_object_value(value)
    }

    fn end(self) -> Result<()> {
        self.ser.formatter.end_object(&mut self.ser.writer)?;
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeStruct for ExpressionSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser.serialize_object_key_value(key, value)
    }

    fn end(self) -> Result<()> {
        self.ser.formatter.end_object(&mut self.ser.writer)?;
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeStructVariant for ExpressionSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser.serialize_object_key_value(key, value)
    }

    fn end(self) -> Result<()> {
        self.ser.formatter.end_object(&mut self.ser.writer)?;
        self.ser.formatter.end_object_value(&mut self.ser.writer)?;
        self.ser.formatter.end_object(&mut self.ser.writer)?;
        Ok(())
    }
}

enum StructKind {
    Attribute,
    Block,
    Custom,
}

#[doc(hidden)]
pub struct SerializeStruct<'a, W, F> {
    kind: StructKind,
    ser: &'a mut Serializer<W, F>,
}

impl<'a, W, F> SerializeStruct<'a, W, F> {
    fn new(kind: StructKind, ser: &'a mut Serializer<W, F>) -> Self {
        SerializeStruct { kind, ser }
    }
}

impl<'a, W, F> ser::SerializeStruct for SerializeStruct<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self.kind {
            StructKind::Attribute => match key {
                "key" => self.ser.serialize_attribute_key(value),
                "expr" => {
                    self.ser.serialize_attribute_value(value)?;
                    self.ser.formatter.end_attribute(&mut self.ser.writer)?;
                    Ok(())
                }
                _ => Ok(()),
            },
            StructKind::Block => match key {
                "identifier" => {
                    self.ser.formatter.begin_block(&mut self.ser.writer)?;
                    value.serialize(IdentifierSerializer::new(self.ser))
                }
                "labels" => value.serialize(BlockLabelSerializer::new(self.ser)),
                "body" => {
                    self.ser.formatter.begin_block_body(&mut self.ser.writer)?;
                    value.serialize(&mut *self.ser)?;
                    self.ser.formatter.end_block(&mut self.ser.writer)?;
                    Ok(())
                }
                _ => Ok(()),
            },
            StructKind::Custom => self.ser.serialize_attribute(key, value),
        }
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

/// Serialize the given value as an HCL byte vector.
///
/// # Errors
///
/// Serialization fails if the type cannot be represented as HCL.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let mut vec = Vec::with_capacity(128);
    to_writer(&mut vec, value)?;
    Ok(vec)
}

/// Serialize the given value as an HCL string.
///
/// # Errors
///
/// Serialization fails if the type cannot be represented as HCL.
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let vec = to_vec(value)?;
    let string = unsafe {
        // We do not emit invalid UTF-8.
        String::from_utf8_unchecked(vec)
    };
    Ok(string)
}

/// Serialize the given value as HCL into the IO stream.
///
/// # Errors
///
/// Serialization fails if any operation on the writer fails or if the type cannot be represented
/// as HCL.
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ?Sized + Serialize,
{
    let mut serializer = Serializer::new(writer);
    value.serialize(&mut serializer)
}

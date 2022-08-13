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
//!
//! The same result could be acheived using the [`block!`] macro:
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
//! let users = vec![
//!     User {
//!         age: 34,
//!         username: "johndoe",
//!         email: "johndoe@example.com",
//!     },
//!     User {
//!         age: 27,
//!         username: "janedoe",
//!         email: "janedoe@example.com",
//!     },
//! ];
//!
//! let body: hcl::Body = users
//!     .into_iter()
//!     .map(|user| {
//!         hcl::block! {
//!             user (user.username) {
//!                 age = (user.age)
//!                 email = (user.email)
//!             }
//!         }
//!     })
//!     .collect();
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
//! "#
//! .trim_start();
//!
//! let serialized = hcl::to_string(&body).unwrap();
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
use crate::{
    format::Format as _,
    structure::ser::{
        BodySerializer, ExpressionSerializer, IdentifierSerializer, SerializeAttributeStruct,
        SerializeBlockStruct, StructureSerializer,
    },
};
use crate::{Error, Result};
use serde::ser::{self, Serialize};
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

    fn format<V, S>(&mut self, value: &V, serializer: S) -> Result<()>
    where
        V: ?Sized + Serialize,
        S: ser::Serializer<Error = Error>,
        S::Ok: crate::format::Format,
    {
        let expr = value.serialize(serializer)?;
        expr.format(&mut self.writer, &mut self.formatter)?;
        Ok(())
    }

    fn serialize_attribute<K, V>(&mut self, key: &K, value: &V) -> Result<()>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        self.serialize_attribute_key(key)?;
        self.serialize_attribute_value(value)
    }

    fn serialize_attribute_key<K>(&mut self, key: &K) -> Result<()>
    where
        K: ?Sized + Serialize,
    {
        self.formatter.begin_attribute(&mut self.writer)?;
        self.format(key, IdentifierSerializer)
    }

    fn serialize_attribute_value<V>(&mut self, value: &V) -> Result<()>
    where
        V: ?Sized + Serialize,
    {
        self.formatter.begin_attribute_value(&mut self.writer)?;
        self.format(value, ExpressionSerializer)?;
        self.formatter.end_attribute(&mut self.writer)?;
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

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        if name == "$hcl::body" {
            self.format(value, BodySerializer)
        } else {
            value.serialize(self)
        }
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
            self.format(value, StructureSerializer)
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
            "$hcl::attribute" => StructKind::Attribute(SerializeAttributeStruct::new()),
            "$hcl::block" => StructKind::Block(SerializeBlockStruct::new()),
            _ => StructKind::Other,
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
        self.formatter.begin_array_value(&mut self.writer)?;
        self.format(value, ExpressionSerializer)?;
        self.formatter.end_array_value(&mut self.writer)?;
        Ok(())
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
        self.serialize_attribute_value(value)
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

enum StructKind {
    Attribute(SerializeAttributeStruct),
    Block(SerializeBlockStruct),
    Other,
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
        match &mut self.kind {
            StructKind::Attribute(ser) => ser.serialize_field(key, value),
            StructKind::Block(ser) => ser.serialize_field(key, value),
            StructKind::Other => self.ser.serialize_attribute(key, value),
        }
    }

    fn end(self) -> Result<()> {
        match self.kind {
            StructKind::Attribute(ser) => {
                ser.end()?
                    .format(&mut self.ser.writer, &mut self.ser.formatter)?;
            }
            StructKind::Block(ser) => {
                ser.end()?
                    .format(&mut self.ser.writer, &mut self.ser.formatter)?;
            }
            StructKind::Other => {}
        }

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

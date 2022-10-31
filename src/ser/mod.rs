//! Serialize a Rust data structure into HCL data.
//!
//! This module provides the [`Serializer`] type and the convienince functions [`to_string`],
//! [`to_vec`] and [`to_writer`] for serializing data to HCL.
//!
//! If you want to serialize the data structures provided by this crate (e.g.
//! [`Body`](crate::Body)) consider using the functionality in the [`format`](crate::format) module
//! instead because it is more efficient.
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
//! let serialized = hcl::to_string(&body)?;
//!
//! assert_eq!(serialized, expected);
//! #   Ok(())
//! # }
//! ```

#[cfg(test)]
mod tests;

use crate::{
    format::{Format, Formatter},
    structure::ser::body::*,
    Error, Identifier, Result,
};
use serde::ser::{self, Impossible, Serialize};
use std::fmt;
use std::io;
use std::marker::PhantomData;

/// A structure for serializing Rust values into HCL.
pub struct Serializer<'a, W> {
    formatter: Formatter<'a, W>,
}

impl<'a, W> Serializer<'a, W>
where
    W: io::Write,
{
    /// Creates a new `Serializer` which serializes to the provides writer using the default
    /// formatter.
    pub fn new(writer: W) -> Serializer<'a, W> {
        Serializer::with_formatter(Formatter::new(writer))
    }

    /// Creates a new `Serializer` which uses the provides formatter to format the serialized HCL.
    pub fn with_formatter(formatter: Formatter<'a, W>) -> Serializer<'a, W> {
        Serializer { formatter }
    }

    /// Consumes `self` and returns the wrapped writer.
    pub fn into_inner(self) -> W {
        self.formatter.into_inner()
    }

    fn format<V>(&mut self, value: V) -> Result<()>
    where
        V: Format,
    {
        value.format(&mut self.formatter)
    }
}

impl<'a, W> ser::Serializer for &'a mut Serializer<'a, W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = SerializeSeq<'a, W>;
    type SerializeTuple = SerializeSeq<'a, W>;
    type SerializeTupleStruct = SerializeSeq<'a, W>;
    type SerializeTupleVariant = SerializeTupleVariant<'a, W>;
    type SerializeMap = SerializeMap<'a, W>;
    type SerializeStruct = SerializeStruct<'a, W>;
    type SerializeStructVariant = SerializeStructVariant<'a, W>;

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
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        BodySerializer
            .serialize_newtype_variant(name, variant_index, variant, value)
            .and_then(|body| self.format(body))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        BodySerializer
            .serialize_seq(len)
            .map(|inner| SerializeSeq { ser: self, inner })
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        BodySerializer
            .serialize_tuple_variant(name, variant_index, variant, len)
            .map(|inner| SerializeTupleVariant { ser: self, inner })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        BodySerializer
            .serialize_map(len)
            .map(|inner| SerializeMap { ser: self, inner })
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        BodySerializer
            .serialize_struct(name, len)
            .map(|inner| SerializeStruct { ser: self, inner })
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        BodySerializer
            .serialize_struct_variant(name, variant_index, variant, len)
            .map(|inner| SerializeStructVariant { ser: self, inner })
    }
}

macro_rules! impl_forward_to_inner {
    ($($tt:tt)+) => {
        type Ok = ();
        type Error = $crate::Error;

        impl_forward_to_inner_internal!($($tt)+);

        fn end(self) -> $crate::Result<Self::Ok, Self::Error> {
            self.inner.end().and_then(|body| self.ser.format(body))
        }
    };
}

macro_rules! impl_forward_to_inner_internal {
    ($method:ident($($arg:ident: $ty:ty),*) $(,$rest:tt)*) => {
        fn $method<T>(&mut self, $($arg: $ty,)* value: &T) -> $crate::Result<(), Self::Error>
        where
            T: ?Sized + serde::ser::Serialize,
        {
            self.inner.$method($($arg,)* value)
        }

        impl_forward_to_inner_internal!($($rest),*);
    };
    ($method:ident $(,$rest:tt)*) => {
        impl_forward_to_inner_internal!($method() $(,$rest)*);
    };
    () => {};
}

#[doc(hidden)]
pub struct SerializeSeq<'a, W> {
    inner: SerializeBodySeq,
    ser: &'a mut Serializer<'a, W>,
}

impl<'a, W> ser::SerializeSeq for SerializeSeq<'a, W>
where
    W: io::Write,
{
    impl_forward_to_inner!(serialize_element);
}

impl<'a, W> ser::SerializeTuple for SerializeSeq<'a, W>
where
    W: io::Write,
{
    impl_forward_to_inner!(serialize_element);
}

impl<'a, W> ser::SerializeTupleStruct for SerializeSeq<'a, W>
where
    W: io::Write,
{
    impl_forward_to_inner!(serialize_field);
}

#[doc(hidden)]
pub struct SerializeTupleVariant<'a, W> {
    inner: SerializeBodyTupleVariant,
    ser: &'a mut Serializer<'a, W>,
}

impl<'a, W> ser::SerializeTupleVariant for SerializeTupleVariant<'a, W>
where
    W: io::Write,
{
    impl_forward_to_inner!(serialize_field);
}

#[doc(hidden)]
pub struct SerializeMap<'a, W> {
    inner: SerializeBodyMap,
    ser: &'a mut Serializer<'a, W>,
}

impl<'a, W> ser::SerializeMap for SerializeMap<'a, W>
where
    W: io::Write,
{
    impl_forward_to_inner!(serialize_key, serialize_value);
}

#[doc(hidden)]
pub struct SerializeStructVariant<'a, W> {
    inner: SerializeBodyStructVariant,
    ser: &'a mut Serializer<'a, W>,
}

impl<'a, W> ser::SerializeStructVariant for SerializeStructVariant<'a, W>
where
    W: io::Write,
{
    impl_forward_to_inner!(serialize_field(key: &'static str));
}

#[doc(hidden)]
pub struct SerializeStruct<'a, W> {
    inner: SerializeBodyStruct,
    ser: &'a mut Serializer<'a, W>,
}

impl<'a, W> ser::SerializeStruct for SerializeStruct<'a, W>
where
    W: io::Write,
{
    impl_forward_to_inner!(serialize_field(key: &'static str));
}

/// Serialize the given value as an HCL byte vector.
///
/// If you want to serialize the data structures provided by this crate (e.g.
/// [`Body`](crate::Body)) consider using [`hcl::format::to_vec`](crate::format::to_vec) instead
/// because it is more efficient.
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
/// If you want to serialize the data structures provided by this crate (e.g.
/// [`Body`](crate::Body)) consider using [`hcl::format::to_string`](crate::format::to_string)
/// instead because it is more efficient.
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
/// If you want to serialize the data structures provided by this crate (e.g.
/// [`Body`](crate::Body)) consider using [`hcl::format::to_writer`](crate::format::to_writer)
/// instead because it is more efficient.
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

pub(crate) struct StringSerializer;

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
        T: ?Sized + fmt::Display,
    {
        Ok(value.to_string())
    }
}

pub(crate) struct IdentifierSerializer;

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
        seq tuple tuple_struct tuple_variant map struct struct_variant
    }
    serialize_self! { some newtype_struct }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        Identifier::new(value)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }
}

pub(crate) struct BoolSerializer;

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

pub(crate) struct SeqSerializer<S> {
    inner: S,
}

impl<S> SeqSerializer<S> {
    pub(crate) fn new(inner: S) -> Self {
        SeqSerializer { inner }
    }
}

impl<S> ser::Serializer for SeqSerializer<S>
where
    S: ser::Serializer + Clone,
{
    type Ok = Vec<S::Ok>;
    type Error = S::Error;

    type SerializeSeq = SerializeInnerSeq<S>;
    type SerializeTuple = SerializeInnerSeq<S>;
    type SerializeTupleStruct = SerializeInnerSeq<S>;
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
        Ok(SerializeInnerSeq::new(self.inner, len))
    }
}

pub(crate) struct SerializeInnerSeq<S>
where
    S: ser::Serializer,
{
    inner: S,
    vec: Vec<S::Ok>,
}

impl<S> SerializeInnerSeq<S>
where
    S: ser::Serializer,
{
    fn new(inner: S, len: Option<usize>) -> Self {
        SerializeInnerSeq {
            inner,
            vec: Vec::with_capacity(len.unwrap_or(0)),
        }
    }
}

impl<S> ser::SerializeSeq for SerializeInnerSeq<S>
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

impl<S> ser::SerializeTuple for SerializeInnerSeq<S>
where
    S: ser::Serializer + Clone,
{
    impl_forward_to_serialize_seq!(serialize_element, Vec<S::Ok>, S::Error);
}

impl<S> serde::ser::SerializeTupleStruct for SerializeInnerSeq<S>
where
    S: ser::Serializer + Clone,
{
    impl_forward_to_serialize_seq!(serialize_field, Vec<S::Ok>, S::Error);
}

pub(crate) struct OptionSerializer<S> {
    inner: S,
}

impl<S> OptionSerializer<S> {
    pub(crate) fn new(inner: S) -> Self {
        OptionSerializer { inner }
    }
}

impl<S> ser::Serializer for OptionSerializer<S>
where
    S: ser::Serializer,
{
    type Ok = Option<S::Ok>;
    type Error = S::Error;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str
        bytes unit newtype_struct newtype_variant unit_struct unit_variant
        seq tuple tuple_struct tuple_variant map struct struct_variant
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        Ok(Some(value.serialize(self.inner)?))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(None)
    }
}

pub(crate) struct FromStrSerializer<O> {
    marker: PhantomData<O>,
}

impl<O> FromStrSerializer<O> {
    pub(crate) fn new() -> Self {
        FromStrSerializer {
            marker: PhantomData,
        }
    }
}

impl<O> ser::Serializer for FromStrSerializer<O>
where
    O: std::str::FromStr,
    O::Err: ser::Error,
{
    type Ok = O;
    type Error = O::Err;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    serialize_unsupported! {
        i8 i16 i32 i64 u8 u16 u32 u64
        bool f32 f64 char bytes unit unit_struct none
        seq tuple tuple_struct tuple_variant map struct struct_variant
        newtype_variant
    }
    serialize_self! { some newtype_struct }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        O::from_str(value)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }
}

//! Serialize a Rust data structure into HCL data.
//!
//! This module provides the [`Serializer`] type and the convienince functions [`to_string`],
//! [`to_vec`] and [`to_writer`] for serializing data to HCL.
//!
//! Furthermore, the [`Block`] and [`LabeledBlock`] wrapper types, and the
//! [`block`][crate::ser::block], [`labeled_block`][crate::ser::labeled_block] and
//! [`doubly_labeled_block`][crate::ser::doubly_labeled_block] functions can be used to construct
//! HCL block structures from custom types. See the type and function level documentation for
//! usage examples.
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
//! ## Serializing HCL blocks using a custom type
//!
//! An example to serialize a terraform configuration block using a custom type and the
//! [`LabeledBlock`] and [`Block`] marker types from this module:
//!
//! ```
//! use hcl::expr::{Expression, Traversal, Variable};
//! use indexmap::{indexmap, IndexMap};
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Config {
//!     #[serde(
//!         rename = "resource",
//!         serialize_with = "hcl::ser::labeled_block"
//!     )]
//!     resources: Resources,
//! }
//!
//! #[derive(Serialize)]
//! struct Resources {
//!     #[serde(
//!         rename = "aws_sns_topic_subscription",
//!         serialize_with = "hcl::ser::labeled_block"
//!     )]
//!     aws_sns_topic_subscriptions: IndexMap<String, AwsSnsTopicSubscription>,
//! }
//!
//! #[derive(Serialize)]
//! struct AwsSnsTopicSubscription {
//!     topic_arn: Traversal,
//!     protocol: Expression,
//!     endpoint: Traversal,
//! }
//!
//! let subscription = AwsSnsTopicSubscription {
//!     topic_arn: Traversal::builder(Variable::new("aws_sns_topic").unwrap())
//!         .attr("my-topic")
//!         .attr("arn")
//!         .build(),
//!     protocol: "sqs".into(),
//!     endpoint: Traversal::builder(Variable::new("aws_sqs_queue").unwrap())
//!         .attr("my-queue")
//!         .attr("arn")
//!         .build()
//! };
//!
//! let config = Config {
//!     resources: Resources {
//!         aws_sns_topic_subscriptions: indexmap! {
//!             "my-subscription".into() => subscription,
//!         },
//!     },
//! };
//!
//! let expected = r#"
//! resource "aws_sns_topic_subscription" "my-subscription" {
//!   topic_arn = aws_sns_topic.my-topic.arn
//!   protocol = "sqs"
//!   endpoint = aws_sqs_queue.my-queue.arn
//! }
//! "#.trim_start();
//!
//! let serialized = hcl::to_string(&config).unwrap();
//!
//! assert_eq!(serialized, expected);
//! ```

pub(crate) mod blocks;
#[cfg(test)]
mod tests;

pub use self::blocks::{block, doubly_labeled_block, labeled_block, Block, LabeledBlock};
use crate::format::{Format, Formatter};
use crate::structure::Body;
use crate::{Error, Identifier, Result};
use serde::ser::{self, Impossible, Serialize, SerializeStruct};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

// Deprecated, this re-export will be removed in a future release.
#[doc(hidden)]
pub use crate::expr::to_expression;

thread_local! {
    static INTERNAL_SERIALIZATION: AtomicBool = AtomicBool::new(false);
}

pub(crate) fn in_internal_serialization() -> bool {
    INTERNAL_SERIALIZATION.with(|flag| flag.load(Ordering::Relaxed))
}

pub(crate) fn with_internal_serialization<R, F: FnOnce() -> R>(f: F) -> R {
    INTERNAL_SERIALIZATION.with(|flag| {
        let old = flag.load(Ordering::Relaxed);
        flag.store(true, Ordering::Relaxed);
        let _on_drop = OnDrop::new(|| {
            flag.store(old, Ordering::Relaxed);
        });
        f()
    })
}

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

    /// Serialize the given value as HCL via the serializer's `Formatter` to the underlying writer.
    ///
    /// # Errors
    ///
    /// Serialization fails if the type cannot be represented as HCL.
    pub fn serialize<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let body = Body::from_serializable(value)?;
        body.format(&mut self.formatter)
    }
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
    serializer.serialize(value)
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

struct U64Serializer;

impl ser::Serializer for U64Serializer {
    type Ok = u64;
    type Error = Error;

    type SerializeSeq = Impossible<u64, Error>;
    type SerializeTuple = Impossible<u64, Error>;
    type SerializeTupleStruct = Impossible<u64, Error>;
    type SerializeTupleVariant = Impossible<u64, Error>;
    type SerializeMap = Impossible<u64, Error>;
    type SerializeStruct = Impossible<u64, Error>;
    type SerializeStructVariant = Impossible<u64, Error>;

    serialize_unsupported! {
        i8 i16 i32 i64 u8 u16 u32 f32 f64 char str bool bytes
        unit unit_variant unit_struct newtype_struct newtype_variant
        some none seq tuple tuple_struct tuple_variant map struct struct_variant
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok> {
        Ok(value)
    }
}

pub(crate) struct SerializeInternalHandleStruct {
    handle: Option<u64>,
}

impl SerializeInternalHandleStruct {
    pub(crate) fn new() -> Self {
        SerializeInternalHandleStruct { handle: None }
    }
}

impl ser::SerializeStruct for SerializeInternalHandleStruct {
    type Ok = usize;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        assert_eq!(key, "handle", "bad handle struct");
        self.handle = Some(value.serialize(U64Serializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        let handle = self.handle.expect("bad handle reference in roundtrip");
        Ok(handle as usize)
    }
}

pub(crate) struct InternalHandles<T> {
    marker: &'static str,
    last_handle: AtomicUsize,
    handles: RefCell<BTreeMap<usize, T>>,
}

impl<T> InternalHandles<T> {
    pub(crate) fn new(marker: &'static str) -> InternalHandles<T> {
        InternalHandles {
            marker,
            last_handle: AtomicUsize::new(0),
            handles: RefCell::new(BTreeMap::new()),
        }
    }

    pub(crate) fn remove(&self, handle: usize) -> T {
        self.handles
            .borrow_mut()
            .remove(&handle)
            .expect("handle not in registry")
    }

    pub(crate) fn serialize<V, S>(&self, value: V, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        V: Into<T>,
    {
        let handle = self.last_handle.fetch_add(1, Ordering::Relaxed);
        self.handles.borrow_mut().insert(handle, value.into());
        let mut s = serializer.serialize_struct(self.marker, 1)?;
        s.serialize_field("handle", &handle)?;
        s.end()
    }
}

struct OnDrop<F: FnOnce()>(Option<F>);

impl<F: FnOnce()> OnDrop<F> {
    fn new(f: F) -> Self {
        Self(Some(f))
    }
}

impl<F: FnOnce()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        self.0.take().unwrap()();
    }
}

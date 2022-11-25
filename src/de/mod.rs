//! Deserialize HCL data to a Rust data structure.
//!
//! The `Deserializer` implementation tries to follow the [HCL JSON Specification][hcl-json-spec]
//! as close as possible.
//!
//! [hcl-json-spec]: https://github.com/hashicorp/hcl/blob/main/json/spec.md

#[cfg(test)]
mod tests;

use crate::{parser, Body, Error, Identifier, Result, Value};
use serde::de::value::StringDeserializer;
use serde::de::{self, Deserializer as _, IntoDeserializer};
use serde::forward_to_deserialize_any;
use std::fmt;
use std::marker::PhantomData;

/// A structure that deserializes HCL into Rust values.
pub struct Deserializer {
    body: Body,
}

impl Deserializer {
    /// Creates a HCL deserializer from a `&str`.
    ///
    /// # Errors
    ///
    /// An [`Error`][Error] is returned when the input is not valid HCL.
    ///
    /// [Error]: ../error/enum.Error.html
    pub fn from_str(input: &str) -> Result<Self> {
        let body = parser::parse(input)?;
        Ok(Deserializer { body })
    }
}

/// Deserialize an instance of type `T` from a string of HCL text.
///
/// By default, the deserialization will follow the [HCL JSON Specification][hcl-json-spec].
///
/// If preserving HCL semantics is required consider deserializing into a [`Body`][Body] instead or
/// use [`hcl::parse`][parse] to directly parse the input into a [`Body`][Body].
///
/// [hcl-json-spec]: https://github.com/hashicorp/hcl/blob/main/json/spec.md
/// [parse]: ../fn.parse.html
/// [Body]: ../struct.Body.html
///
/// # Example
///
/// ```
/// use serde_json::{json, Value};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let input = r#"
///     some_attr = {
///       foo = [1, 2]
///       bar = true
///     }
///
///     some_block "some_block_label" {
///       attr = "value"
///     }
/// "#;
///
/// let expected = json!({
///     "some_attr": {
///         "foo": [1, 2],
///         "bar": true
///     },
///     "some_block": {
///         "some_block_label": {
///             "attr": "value"
///         }
///     }
/// });
///
/// let value: Value = hcl::from_str(input)?;
///
/// assert_eq!(value, expected);
/// #   Ok(())
/// # }
/// ```
///
/// # Errors
///
/// This functions fails with an error if the data does not match the structure of `T`.
pub fn from_str<'de, T>(s: &'de str) -> Result<T>
where
    T: de::Deserialize<'de>,
{
    let deserializer = Deserializer::from_str(s)?;
    T::deserialize(deserializer)
}

/// Deserialize an instance of type `T` from an IO stream of HCL.
///
/// See the documentation of [`from_str`][from_str] for more information.
///
/// # Example
///
/// ```
/// use hcl::Value;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let input = r#"
///     some_attr = {
///       foo = [1, 2]
///       bar = true
///     }
///
///     some_block "some_block_label" {
///       attr = "value"
///     }
/// "#;
///
/// let expected = hcl::value!({
///     some_attr = {
///         foo = [1, 2]
///         bar = true
///     }
///     some_block = {
///         some_block_label = {
///             attr = "value"
///         }
///     }
/// });
///
/// let value: Value = hcl::from_reader(input.as_bytes())?;
///
/// assert_eq!(value, expected);
/// #   Ok(())
/// # }
/// ```
///
/// # Errors
///
/// This functions fails with an error if reading from the reader fails or if the data does not
/// match the structure of `T`.
pub fn from_reader<T, R>(mut reader: R) -> Result<T>
where
    T: de::DeserializeOwned,
    R: std::io::Read,
{
    let mut s = String::new();
    reader.read_to_string(&mut s)?;

    from_str(&s)
}

/// Deserialize an instance of type `T` from a byte slice.
///
/// See the documentation of [`from_str`][from_str] for more information.
///
/// # Errors
///
/// This functions fails with an error if `buf` does not contain valid UTF-8 or if the data does
/// not match the structure of `T`.
pub fn from_slice<'de, T>(buf: &'de [u8]) -> Result<T>
where
    T: de::Deserialize<'de>,
{
    let s = std::str::from_utf8(buf)?;
    from_str(s)
}

/// Interpret a `hcl::Body` as an instance of type `T`.
///
/// # Example
///
/// ```
/// use serde::Deserialize;
/// use hcl::{Block, Body};
///
/// # use std::error::Error;
/// #
/// # fn main() -> Result<(), Box<dyn Error>> {
/// #[derive(Deserialize, Debug)]
/// struct User {
///     name: String,
///     email: String,
/// }
///
/// #[derive(Deserialize, Debug)]
/// struct Config {
///     user: User,
/// }
///
/// let body = Body::builder()
///     .add_block(
///         Block::builder("user")
///             .add_attribute(("name", "John Doe"))
///             .add_attribute(("email", "john@doe.tld"))
///             .build()
///     )
///     .build();
///
/// let config: Config = hcl::from_body(body)?;
/// println!("{:#?}", config);
/// #   Ok(())
/// # }
/// ```
///
/// # Errors
///
/// This functions fails with an error if the data does not match the structure of `T`.
pub fn from_body<T>(body: Body) -> Result<T>
where
    T: de::DeserializeOwned,
{
    T::deserialize(Deserializer { body })
}

impl<'de> de::Deserializer<'de> for Deserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Value::from(self.body)
            .into_deserializer()
            .deserialize_any(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if name == "$hcl::Body" {
            // Specialized handling of `hcl::Body`.
            self.body.into_deserializer().deserialize_any(visitor)
        } else {
            // Generic deserialization according to the HCL JSON spec.
            self.deserialize_any(visitor)
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Value::from(self.body)
            .into_deserializer()
            .deserialize_enum(name, variants, visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

// A trait that allows enum types to report the name of their variant.
pub(crate) trait VariantName {
    fn variant_name(&self) -> &'static str;
}

// Not public API.
#[doc(hidden)]
pub struct NewtypeStructDeserializer<T, E = Error> {
    value: T,
    marker: PhantomData<E>,
}

impl<T, E> NewtypeStructDeserializer<T, E> {
    pub(crate) fn new(value: T) -> Self {
        NewtypeStructDeserializer {
            value,
            marker: PhantomData,
        }
    }
}

impl<'de, T, E> de::Deserializer<'de> for NewtypeStructDeserializer<T, E>
where
    T: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self.value.into_deserializer())
    }
}

pub(crate) struct OptionDeserializer<T, E = Error> {
    value: Option<T>,
    marker: PhantomData<E>,
}

impl<T, E> OptionDeserializer<T, E> {
    pub(crate) fn new(value: Option<T>) -> Self {
        OptionDeserializer {
            value,
            marker: PhantomData,
        }
    }
}

impl<'de, T, E> de::Deserializer<'de> for OptionDeserializer<T, E>
where
    T: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str
        string bytes byte_buf option unit unit_struct newtype_struct seq
        tuple tuple_struct map struct enum identifier ignored_any
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Some(value) => visitor.visit_some(value.into_deserializer()),
            None => visitor.visit_none(),
        }
    }
}

pub(crate) struct EnumAccess<T, E = Error> {
    value: T,
    marker: PhantomData<E>,
}

impl<T, E> EnumAccess<T, E> {
    pub(crate) fn new(value: T) -> Self {
        EnumAccess {
            value,
            marker: PhantomData,
        }
    }
}

impl<'de, T, E> de::EnumAccess<'de> for EnumAccess<T, E>
where
    T: IntoDeserializer<'de, E> + VariantName,
    E: de::Error,
{
    type Error = E;
    type Variant = VariantAccess<T, E>;

    fn variant_seed<S>(self, seed: S) -> Result<(S::Value, Self::Variant), Self::Error>
    where
        S: de::DeserializeSeed<'de>,
    {
        let variant_name = self.value.variant_name();

        seed.deserialize(variant_name.into_deserializer())
            .map(|variant| (variant, VariantAccess::new(self.value)))
    }
}

// Not public API.
#[doc(hidden)]
pub struct VariantAccess<T, E = Error> {
    value: T,
    marker: PhantomData<E>,
}

impl<T, E> VariantAccess<T, E> {
    fn new(value: T) -> Self {
        VariantAccess {
            value,
            marker: PhantomData,
        }
    }
}

impl<'de, T, E> de::VariantAccess<'de> for VariantAccess<T, E>
where
    T: IntoDeserializer<'de, E>,
    E: de::Error,
{
    type Error = E;

    fn unit_variant(self) -> Result<(), Self::Error> {
        de::Deserialize::deserialize(self.value.into_deserializer())
    }

    fn newtype_variant_seed<S>(self, seed: S) -> Result<S::Value, Self::Error>
    where
        S: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.value.into_deserializer())
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.value.into_deserializer().deserialize_seq(visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.value.into_deserializer().deserialize_map(visitor)
    }
}

pub(crate) struct FromStrVisitor<T> {
    expecting: &'static str,
    marker: PhantomData<T>,
}

impl<T> FromStrVisitor<T> {
    pub(crate) fn new(expecting: &'static str) -> FromStrVisitor<T> {
        FromStrVisitor {
            expecting,
            marker: PhantomData,
        }
    }
}

impl<'de, T> de::Visitor<'de> for FromStrVisitor<T>
where
    T: std::str::FromStr,
    T::Err: de::Error,
{
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.expecting)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        T::from_str(value).map_err(de::Error::custom)
    }
}

impl<'de> IntoDeserializer<'de, Error> for Identifier {
    type Deserializer = StringDeserializer<Error>;

    fn into_deserializer(self) -> Self::Deserializer {
        self.into_inner().into_deserializer()
    }
}

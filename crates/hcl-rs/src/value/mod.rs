//! The Value enum, a loosely typed way of representing any valid HCL value.

pub(crate) mod de;
mod from;
mod ser;

use std::fmt;

use serde::{de::DeserializeOwned, ser::Serialize};

use self::{de::ValueDeserializer, ser::ValueSerializer};
use crate::{capsule::Capsule, format, Number, Result};

/// The map type used for HCL objects.
pub type Map<K, V> = indexmap::IndexMap<K, V>;

/// Represents any valid HCL value.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub enum Value {
    /// Represents a HCL null value.
    #[default]
    Null,
    /// Represents a HCL boolean.
    Bool(bool),
    /// Represents a HCL number, either integer or float.
    Number(Number),
    /// Represents a HCL string.
    String(String),
    /// Represents a HCL array.
    Array(Vec<Value>),
    /// Represents a HCL object.
    Object(Map<String, Value>),
    /// Represents an opaque value emitted by the calling application.
    Capsule(Capsule),
}

impl Value {
    /// If the `Value` is an Array, returns the associated vector. Returns None
    /// otherwise.
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Self::Array(array) => Some(array),
            _ => None,
        }
    }

    /// If the `Value` is an Array, returns the associated mutable vector.
    /// Returns None otherwise.
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Self::Array(array) => Some(array),
            _ => None,
        }
    }

    /// If the `Value` is a Boolean, represent it as bool if possible. Returns
    /// None otherwise.
    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Self::Bool(b) => Some(b),
            _ => None,
        }
    }

    /// If the `Value` is a Number, represent it as f64 if possible. Returns
    /// None otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        self.as_number().and_then(Number::as_f64)
    }

    /// If the `Value` is a Number, represent it as i64 if possible. Returns
    /// None otherwise.
    pub fn as_i64(&self) -> Option<i64> {
        self.as_number().and_then(Number::as_i64)
    }

    /// If the `Value` is a Null, returns (). Returns None otherwise.
    pub fn as_null(&self) -> Option<()> {
        match self {
            Self::Null => Some(()),
            _ => None,
        }
    }

    /// If the `Value` is a Number, returns the associated Number. Returns None
    /// otherwise.
    pub fn as_number(&self) -> Option<&Number> {
        match self {
            Self::Number(num) => Some(num),
            _ => None,
        }
    }

    /// If the `Value` is an Object, returns the associated Map. Returns None
    /// otherwise.
    pub fn as_object(&self) -> Option<&Map<String, Value>> {
        match self {
            Self::Object(object) => Some(object),
            _ => None,
        }
    }

    /// If the `Value` is an Object, returns the associated mutable Map.
    /// Returns None otherwise.
    pub fn as_object_mut(&mut self) -> Option<&mut Map<String, Value>> {
        match self {
            Self::Object(object) => Some(object),
            _ => None,
        }
    }

    /// If the `Value` is a capsule, returns the associated Capsule. Returns None
    /// otherwise.
    pub fn as_capsule(&self) -> Option<&Capsule> {
        match self {
            Self::Capsule(capsule) => Some(capsule),
            _ => None,
        }
    }

    /// If the `Value` is a capsule, returns the associated mutable Capsule.
    /// Returns None otherwise.
    pub fn as_capsule_mut(&mut self) -> Option<&mut Capsule> {
        match self {
            Self::Capsule(capsule) => Some(capsule),
            _ => None,
        }
    }

    /// If the `Value` is a String, returns the associated str. Returns None
    /// otherwise.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// If the `Value` is a Number, represent it as u64 if possible. Returns
    /// None otherwise.
    pub fn as_u64(&self) -> Option<u64> {
        self.as_number().and_then(Number::as_u64)
    }

    /// Returns true if the `Value` is an Array. Returns false otherwise.
    ///
    /// For any Value on which `is_array` returns true, `as_array` and
    /// `as_array_mut` are guaranteed to return the vector representing the
    /// array.
    pub fn is_array(&self) -> bool {
        self.as_array().is_some()
    }

    /// Returns true if the `Value` is a Boolean. Returns false otherwise.
    ///
    /// For any Value on which `is_boolean` returns true, `as_bool` is
    /// guaranteed to return the boolean value.
    pub fn is_boolean(&self) -> bool {
        self.as_bool().is_some()
    }

    /// Returns true if the `Value` is a number that can be represented by f64.
    ///
    /// For any Value on which `is_f64` returns true, `as_f64` is guaranteed to
    /// return the floating point value.
    pub fn is_f64(&self) -> bool {
        self.as_number().map_or(false, Number::is_f64)
    }

    /// Returns true if the `Value` is an integer between `i64::MIN` and
    /// `i64::MAX`.
    ///
    /// For any Value on which `is_i64` returns true, `as_i64` is guaranteed to
    /// return the integer value.
    pub fn is_i64(&self) -> bool {
        self.as_number().map_or(false, Number::is_i64)
    }

    /// Returns true if the `Value` is a Number. Returns false otherwise.
    pub fn is_number(&self) -> bool {
        self.as_number().is_some()
    }

    /// Returns true if the `Value` is a Null. Returns false otherwise.
    ///
    /// For any Value on which `is_null` returns true, `as_null` is guaranteed
    /// to return `Some(())`.
    pub fn is_null(&self) -> bool {
        self.as_null().is_some()
    }

    /// Returns true if the `Value` is an Object. Returns false otherwise.
    ///
    /// For any Value on which `is_object` returns true, `as_object` and
    /// `as_object_mut` are guaranteed to return the map representation of the
    /// object.
    pub fn is_object(&self) -> bool {
        self.as_object().is_some()
    }

    /// Returns true if the `Value` is a Capsule. Returns false otherwise.
    ///
    /// For any Value on which `is_capsule` returns true, `as_capsule` and
    /// `as_capsule_mut` are guaranteed to return `Some(_)`.
    pub fn is_capsule(&self) -> bool {
        self.as_capsule().is_some()
    }

    /// Returns true if the `Value` is a String. Returns false otherwise.
    ///
    /// For any Value on which `is_string` returns true, `as_str` is guaranteed
    /// to return the string slice.
    pub fn is_string(&self) -> bool {
        self.as_str().is_some()
    }

    /// Returns true if the `Value` is an integer between zero and `u64::MAX`.
    ///
    /// For any Value on which `is_u64` returns true, `as_u64` is guaranteed to
    /// return the integer value.
    pub fn is_u64(&self) -> bool {
        self.as_number().map_or(false, Number::is_u64)
    }

    /// Takes the value out of the `Value`, leaving a `Null` in its place.
    pub fn take(&mut self) -> Value {
        std::mem::replace(self, Value::Null)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Formatting a `Value` as string cannot fail.
        let formatted = format::to_string(self).expect("a Value failed to format unexpectedly");
        f.write_str(&formatted)
    }
}

/// Convert a `T` into `hcl::Value` which is an enum that can represent any valid HCL value.
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use hcl::{Map, Value};
/// use serde::Serialize;
///
/// #[derive(Debug, Serialize)]
/// struct Custom {
///     foo: String,
///     bar: u64,
/// }
///
/// let custom = Custom {
///     foo: "baz".into(),
///     bar: 42,
/// };
///
/// let expected = Value::Object({
///     let mut object = Map::new();
///     object.insert("foo".into(), "baz".into());
///     object.insert("bar".into(), 42u64.into());
///     object
/// });
///
/// let value = hcl::to_value(&custom)?;
///
/// assert_eq!(value, expected);
/// #     Ok(())
/// # }
/// ```
///
/// # Errors
///
/// This conversion can fail if `T`'s implementation of `Serialize` decides to fail, or if `T`
/// contains a map with non-string keys.
pub fn to_value<T>(value: T) -> Result<Value>
where
    T: Serialize,
{
    value.serialize(ValueSerializer)
}

/// Convert a `hcl::Value` into a type `T` that implements `serde::Deserialize`.
///
/// # Example
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use hcl::{Map, Value};
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize, PartialEq)]
/// struct Custom {
///     foo: String,
///     bar: u64,
/// }
///
/// let value = Value::Object({
///     let mut object = Map::new();
///     object.insert("foo".into(), "baz".into());
///     object.insert("bar".into(), 42u64.into());
///     object
/// });
///
///
/// let expected = Custom {
///     foo: "baz".into(),
///     bar: 42,
/// };
///
/// let custom: Custom = hcl::from_value(value)?;
///
/// assert_eq!(custom, expected);
/// #     Ok(())
/// # }
/// ```
///
/// # Errors
///
/// This conversion can fail if `T`'s implementation of [`serde::Deserialize`] decides to fail.
pub fn from_value<T>(value: Value) -> Result<T>
where
    T: DeserializeOwned,
{
    T::deserialize(ValueDeserializer::new(value))
}

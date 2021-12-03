use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{self, Display};

/// Represents a HCL number.
#[derive(Debug, PartialEq, Clone)]
pub enum Number {
    /// Represents a positive integer.
    PosInt(u64),
    /// Represents a negative integer.
    NegInt(i64),
    /// Represents a float.
    Float(f64),
}

impl Number {
    /// Represents the `Number` as f64 if possible. Returns None otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Self::PosInt(n) => Some(n as f64),
            Self::NegInt(n) => Some(n as f64),
            Self::Float(n) => Some(n),
        }
    }

    /// If the `Number` is an integer, represent it as i64 if possible. Returns None otherwise.
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Self::PosInt(n) => {
                if n <= i64::max_value() as u64 {
                    Some(n as i64)
                } else {
                    None
                }
            }
            Self::NegInt(n) => Some(n),
            Self::Float(_) => None,
        }
    }

    /// If the `Number` is an integer, represent it as u64 if possible. Returns None otherwise.
    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            Self::PosInt(n) => Some(n),
            Self::NegInt(_) | Self::Float(_) => None,
        }
    }

    /// Returns true if the `Number` is a float.
    ///
    /// For any `Number` on which `is_f64` returns true, `as_f64` is guaranteed to return the
    /// float value.
    pub fn is_f64(&self) -> bool {
        match self {
            Self::Float(_) => true,
            Self::PosInt(_) | Self::NegInt(_) => false,
        }
    }

    /// Returns true if the `Number` is an integer between `i64::MIN` and `i64::MAX`.
    ///
    /// For any `Number` on which `is_i64` returns true, `as_i64` is guaranteed to return the
    /// integer value.
    pub fn is_i64(&self) -> bool {
        match *self {
            Self::PosInt(v) => v <= i64::max_value() as u64,
            Self::NegInt(_) => true,
            Self::Float(_) => false,
        }
    }

    /// Returns true if the `Number` is an integer between zero and `u64::MAX`.
    ///
    /// For any `Number` on which `is_u64` returns true, `as_u64` is guaranteed to return the
    /// integer value.
    pub fn is_u64(&self) -> bool {
        match self {
            Self::PosInt(_) => true,
            Self::NegInt(_) | Self::Float(_) => false,
        }
    }
}

macro_rules! impl_from_unsigned {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for Number {
                fn from(u: $ty) -> Self {
                    Self::PosInt(u as u64)
                }
            }
        )*
    };
}

macro_rules! impl_from_signed {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for Number {
                fn from(i: $ty) -> Self {
                    if i < 0 {
                        Self::NegInt(i as i64)
                    } else {
                        Self::PosInt(i as u64)
                    }
                }
            }
        )*
    };
}

impl_from_unsigned!(u8, u16, u32, u64, usize);
impl_from_signed!(i8, i16, i32, i64, isize);

impl From<f32> for Number {
    fn from(f: f32) -> Self {
        Self::Float(f as f64)
    }
}

impl From<f64> for Number {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl Display for Number {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::PosInt(i) => Display::fmt(&i, formatter),
            Self::NegInt(i) => Display::fmt(&i, formatter),
            Self::Float(f) => Display::fmt(&f, formatter),
        }
    }
}

impl Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Self::PosInt(i) => serializer.serialize_u64(i),
            Self::NegInt(i) => serializer.serialize_i64(i),
            Self::Float(f) => serializer.serialize_f64(f),
        }
    }
}

impl<'de> Deserialize<'de> for Number {
    fn deserialize<D>(deserializer: D) -> Result<Number, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct NumberVisitor;

        impl<'de> Visitor<'de> for NumberVisitor {
            type Value = Number;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a HCL number")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Number, E> {
                Ok(value.into())
            }

            fn visit_u64<E>(self, value: u64) -> Result<Number, E> {
                Ok(value.into())
            }

            fn visit_f64<E>(self, value: f64) -> Result<Number, E> {
                Ok(value.into())
            }
        }

        deserializer.deserialize_any(NumberVisitor)
    }
}

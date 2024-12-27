//! HCL number representation.

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::{Add, Div, Mul, Neg, Rem, Sub};
#[cfg(feature = "serde")]
use serde::de::Unexpected;

enum CoerceResult {
    PosInt(u64, u64),
    NegInt(i64, i64),
    Float(f64, f64),
}

// Coerce two numbers to a common type suitable for binary operations.
fn coerce(a: N, b: N) -> CoerceResult {
    match (a, b) {
        (N::PosInt(a), N::PosInt(b)) => CoerceResult::PosInt(a, b),
        (N::NegInt(a), N::NegInt(b)) => CoerceResult::NegInt(a, b),
        (N::Float(a), N::Float(b)) => CoerceResult::Float(a, b),
        (a, b) => CoerceResult::Float(a.to_f64(), b.to_f64()),
    }
}

/// Represents an HCL number.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd)]
pub struct Number {
    n: N,
}

#[derive(Clone, Copy)]
enum N {
    PosInt(u64),
    /// Always less than zero.
    NegInt(i64),
    /// Always finite.
    Float(f64),
}

impl N {
    fn from_finite_f64(value: f64) -> N {
        debug_assert!(value.is_finite());

        #[cfg(feature = "std")]
        let no_fraction = value.fract() == 0.0;

        // `core::f64` does not have the `fract()` method.
        #[cfg(not(feature = "std"))]
        #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        let no_fraction = value - (value as i64 as f64) == 0.0;

        if no_fraction {
            #[allow(clippy::cast_possible_truncation)]
            N::from(value as i64)
        } else {
            N::Float(value)
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match *self {
            N::PosInt(n) => i64::try_from(n).ok(),
            N::NegInt(n) => Some(n),
            N::Float(_) => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match *self {
            N::PosInt(n) => Some(n),
            N::NegInt(n) => u64::try_from(n).ok(),
            N::Float(_) => None,
        }
    }

    fn to_f64(self) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        match self {
            N::PosInt(n) => n as f64,
            N::NegInt(n) => n as f64,
            N::Float(n) => n,
        }
    }

    fn is_f64(&self) -> bool {
        match self {
            N::Float(_) => true,
            N::PosInt(_) | N::NegInt(_) => false,
        }
    }

    fn is_i64(&self) -> bool {
        match self {
            N::NegInt(_) => true,
            N::PosInt(_) | N::Float(_) => false,
        }
    }

    fn is_u64(&self) -> bool {
        match self {
            N::PosInt(_) => true,
            N::NegInt(_) | N::Float(_) => false,
        }
    }
}

impl PartialEq for N {
    fn eq(&self, other: &Self) -> bool {
        match coerce(*self, *other) {
            CoerceResult::PosInt(a, b) => a == b,
            CoerceResult::NegInt(a, b) => a == b,
            CoerceResult::Float(a, b) => a == b,
        }
    }
}

// N is `Eq` because we ensure that the wrapped f64 is always finite.
impl Eq for N {}

impl PartialOrd for N {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match coerce(*self, *other) {
            CoerceResult::PosInt(a, b) => a.partial_cmp(&b),
            CoerceResult::NegInt(a, b) => a.partial_cmp(&b),
            CoerceResult::Float(a, b) => a.partial_cmp(&b),
        }
    }
}

impl Hash for N {
    fn hash<H>(&self, h: &mut H)
    where
        H: Hasher,
    {
        match *self {
            N::PosInt(n) => n.hash(h),
            N::NegInt(n) => n.hash(h),
            N::Float(n) => {
                if n == 0.0f64 {
                    // There are 2 zero representations, +0 and -0, which
                    // compare equal but have different bits. We use the +0 hash
                    // for both so that hash(+0) == hash(-0).
                    0.0f64.to_bits().hash(h);
                } else {
                    n.to_bits().hash(h);
                }
            }
        }
    }
}

impl From<i64> for N {
    fn from(i: i64) -> Self {
        if i < 0 {
            N::NegInt(i)
        } else {
            #[allow(clippy::cast_sign_loss)]
            N::PosInt(i as u64)
        }
    }
}

impl Neg for N {
    type Output = N;

    fn neg(self) -> Self::Output {
        match self {
            #[allow(clippy::cast_possible_wrap)]
            N::PosInt(value) => N::NegInt(-(value as i64)),
            N::NegInt(value) => N::from(-value),
            N::Float(value) => N::Float(-value),
        }
    }
}

impl Add for N {
    type Output = N;

    fn add(self, rhs: Self) -> Self::Output {
        match coerce(self, rhs) {
            CoerceResult::PosInt(a, b) => N::PosInt(a + b),
            CoerceResult::NegInt(a, b) => N::NegInt(a + b),
            CoerceResult::Float(a, b) => N::from_finite_f64(a + b),
        }
    }
}

impl Sub for N {
    type Output = N;

    fn sub(self, rhs: Self) -> Self::Output {
        match coerce(self, rhs) {
            CoerceResult::PosInt(a, b) => {
                if b > a {
                    #[allow(clippy::cast_possible_wrap)]
                    N::NegInt(a as i64 - b as i64)
                } else {
                    N::PosInt(a - b)
                }
            }
            CoerceResult::NegInt(a, b) => N::from(a - b),
            CoerceResult::Float(a, b) => N::from_finite_f64(a - b),
        }
    }
}

impl Mul for N {
    type Output = N;

    fn mul(self, rhs: Self) -> Self::Output {
        match coerce(self, rhs) {
            CoerceResult::PosInt(a, b) => N::PosInt(a * b),
            CoerceResult::NegInt(a, b) => N::from(a * b),
            CoerceResult::Float(a, b) => N::from_finite_f64(a * b),
        }
    }
}

impl Div for N {
    type Output = N;

    fn div(self, rhs: Self) -> Self::Output {
        N::from_finite_f64(self.to_f64() / rhs.to_f64())
    }
}

impl Rem for N {
    type Output = N;

    fn rem(self, rhs: Self) -> Self::Output {
        match coerce(self, rhs) {
            CoerceResult::PosInt(a, b) => N::PosInt(a % b),
            CoerceResult::NegInt(a, b) => N::NegInt(a % b),
            CoerceResult::Float(a, b) => N::from_finite_f64(a % b),
        }
    }
}

impl Number {
    /// Creates a new `Number` from a `f64`. Returns `None` if the float is infinite or NaN.
    ///
    /// # Example
    ///
    /// ```
    /// # use hcl_primitives::Number;
    /// assert!(Number::from_f64(42.0).is_some());
    /// assert!(Number::from_f64(f64::NAN).is_none());
    /// assert!(Number::from_f64(f64::INFINITY).is_none());
    /// assert!(Number::from_f64(f64::NEG_INFINITY).is_none());
    /// ```
    pub fn from_f64(f: f64) -> Option<Number> {
        if f.is_finite() {
            Some(Number::from_finite_f64(f))
        } else {
            None
        }
    }

    pub(crate) fn from_finite_f64(f: f64) -> Number {
        Number {
            n: N::from_finite_f64(f),
        }
    }

    /// Represents the `Number` as f64 if possible. Returns None otherwise.
    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        Some(self.n.to_f64())
    }

    /// If the `Number` is an integer, represent it as i64 if possible. Returns None otherwise.
    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        self.n.as_i64()
    }

    /// If the `Number` is an integer, represent it as u64 if possible. Returns None otherwise.
    #[inline]
    pub fn as_u64(&self) -> Option<u64> {
        self.n.as_u64()
    }

    /// Returns true if the `Number` is a float.
    ///
    /// For any `Number` on which `is_f64` returns true, `as_f64` is guaranteed to return the
    /// float value.
    #[inline]
    pub fn is_f64(&self) -> bool {
        self.n.is_f64()
    }

    /// Returns true if the `Number` is an integer between `i64::MIN` and `i64::MAX`.
    ///
    /// For any `Number` on which `is_i64` returns true, `as_i64` is guaranteed to return the
    /// integer value.
    #[inline]
    pub fn is_i64(&self) -> bool {
        self.n.is_i64()
    }

    /// Returns true if the `Number` is an integer between zero and `u64::MAX`.
    ///
    /// For any `Number` on which `is_u64` returns true, `as_u64` is guaranteed to return the
    /// integer value.
    #[inline]
    pub fn is_u64(&self) -> bool {
        self.n.is_u64()
    }

    // Not public API. Used to generate better deserialization errors in `hcl-rs`.
    #[cfg(feature = "serde")]
    #[doc(hidden)]
    #[cold]
    pub fn unexpected(&self) -> Unexpected {
        match self.n {
            N::PosInt(v) => Unexpected::Unsigned(v),
            N::NegInt(v) => Unexpected::Signed(v),
            N::Float(v) => Unexpected::Float(v),
        }
    }
}

macro_rules! impl_from_unsigned {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for Number {
                #[inline]
                fn from(u: $ty) -> Self {
                    Number {
                        #[allow(clippy::cast_lossless)]
                        n: N::PosInt(u as u64)
                    }
                }
            }
        )*
    };
}

macro_rules! impl_from_signed {
    ($($ty:ty),*) => {
        $(
            impl From<$ty> for Number {
                #[inline]
                fn from(i: $ty) -> Self {
                    Number {
                        #[allow(clippy::cast_lossless)]
                        n: N::from(i as i64)
                    }
                }
            }
        )*
    };
}

macro_rules! impl_binary_ops {
    ($($op:ty => $method:ident),*) => {
        $(
            impl $op for Number {
                type Output = Number;

                fn $method(self, rhs: Self) -> Self::Output {
                    Number {
                        n: self.n.$method(rhs.n)
                    }
                }
            }
        )*
    };
}

impl_from_unsigned!(u8, u16, u32, u64, usize);
impl_from_signed!(i8, i16, i32, i64, isize);
impl_binary_ops!(Add => add, Sub => sub, Mul => mul, Div => div, Rem => rem);

impl Neg for Number {
    type Output = Number;

    fn neg(self) -> Self::Output {
        Number { n: -self.n }
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.n {
            N::PosInt(v) => f.write_str(itoa::Buffer::new().format(v)),
            N::NegInt(v) => f.write_str(itoa::Buffer::new().format(v)),
            N::Float(v) => f.write_str(ryu::Buffer::new().format_finite(v)),
        }
    }
}

impl fmt::Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Number({self})")
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Number {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.n {
            N::PosInt(v) => serializer.serialize_u64(v),
            N::NegInt(v) => serializer.serialize_i64(v),
            N::Float(v) => serializer.serialize_f64(v),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Number {
    fn deserialize<D>(deserializer: D) -> Result<Number, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct NumberVisitor;

        impl<'de> serde::de::Visitor<'de> for NumberVisitor {
            type Value = Number;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an HCL number")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Number, E> {
                Ok(value.into())
            }

            fn visit_u64<E>(self, value: u64) -> Result<Number, E> {
                Ok(value.into())
            }

            fn visit_f64<E>(self, value: f64) -> Result<Number, E>
            where
                E: serde::de::Error,
            {
                Number::from_f64(value).ok_or_else(|| serde::de::Error::custom("not an HCL number"))
            }
        }

        deserializer.deserialize_any(NumberVisitor)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserializer<'de> for Number {
    type Error = super::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.n {
            N::PosInt(i) => visitor.visit_u64(i),
            N::NegInt(i) => visitor.visit_i64(i),
            N::Float(f) => visitor.visit_f64(f),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct enum map struct identifier ignored_any
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! float {
        ($f:expr) => {
            Number::from_finite_f64($f)
        };
    }

    macro_rules! int {
        ($i:expr) => {
            Number::from($i)
        };
    }

    macro_rules! assert_op {
        ($expr:expr, $expected:expr, $check:ident) => {
            let result = $expr;
            assert_eq!(result, $expected, "incorrect number op result");
            assert!(result.$check());
        };
    }

    #[test]
    fn neg() {
        assert_op!(-int!(1u64), int!(-1i64), is_i64);
        assert_op!(-float!(1.5), float!(-1.5), is_f64);
        assert_op!(-float!(1.0), int!(-1i64), is_i64);
    }

    #[test]
    fn add() {
        assert_op!(int!(1i64) + int!(2u64), int!(3), is_u64);
        assert_op!(float!(1.5) + float!(1.5), int!(3), is_u64);
        assert_op!(float!(1.5) + int!(-1i64), float!(0.5), is_f64);
        assert_op!(int!(-1i64) + int!(-2i64), int!(-3i64), is_i64);
    }

    #[test]
    fn sub() {
        assert_op!(int!(1i64) - int!(2u64), int!(-1i64), is_i64);
        assert_op!(int!(-1i64) - int!(-2i64), int!(1u64), is_u64);
        assert_op!(float!(1.5) - float!(1.5), int!(0), is_u64);
        assert_op!(float!(1.5) - int!(-1i64), float!(2.5), is_f64);
    }

    #[test]
    fn mul() {
        assert_op!(int!(-1i64) * int!(2u64), int!(-2i64), is_i64);
        assert_op!(int!(-1i64) * int!(-2i64), int!(2u64), is_u64);
        assert_op!(float!(1.5) * float!(1.5), float!(2.25), is_f64);
        assert_op!(float!(1.5) * int!(-1i64), float!(-1.5), is_f64);
    }

    #[test]
    fn div() {
        assert_op!(int!(1u64) / int!(2u64), float!(0.5), is_f64);
        assert_op!(float!(4.1) / float!(2.0), float!(2.05), is_f64);
        assert_op!(int!(4u64) / int!(2u64), int!(2u64), is_u64);
        assert_op!(int!(-4i64) / int!(2u64), int!(-2i64), is_i64);
        assert_op!(float!(4.0) / float!(2.0), int!(2), is_u64);
        assert_op!(float!(-4.0) / float!(2.0), int!(-2), is_i64);
    }

    #[test]
    fn rem() {
        assert_op!(int!(3u64) % int!(2u64), int!(1u64), is_u64);
        assert_op!(
            float!(4.1) % float!(2.0),
            float!(0.099_999_999_999_999_64),
            is_f64
        );
        assert_op!(int!(4u64) % int!(2u64), int!(0u64), is_u64);
        assert_op!(int!(-4i64) % int!(3u64), int!(-1i64), is_i64);
        assert_op!(float!(4.0) % float!(2.0), int!(0), is_u64);
        assert_op!(float!(-4.0) % float!(3.0), int!(-1), is_i64);
    }
}

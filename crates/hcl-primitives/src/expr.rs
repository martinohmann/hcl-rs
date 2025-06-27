//! Primitives for the HCL expression sub-language.

use crate::Error;
use core::fmt;
use core::str::FromStr;

/// An operator that can be applied to an expression.
///
/// For more details, check the section about operations in the [HCL syntax
/// specification](https://github.com/hashicorp/hcl/blob/main/hclsyntax/spec.md#operations).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum UnaryOperator {
    /// Negate operator (`-`).
    Neg,
    /// Not operator (`!`).
    Not,
}

impl UnaryOperator {
    /// Returns the `UnaryOperator` as a static `&str`.
    pub fn as_str(&self) -> &'static str {
        match self {
            UnaryOperator::Neg => "-",
            UnaryOperator::Not => "!",
        }
    }
}

impl fmt::Display for UnaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for UnaryOperator {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" => Ok(UnaryOperator::Neg),
            "!" => Ok(UnaryOperator::Not),
            _ => Err(Error::new(format!("invalid unary operator: `{s}`"))),
        }
    }
}

/// An operator that can be applied to two expressions.
///
/// For more details, check the section about operations in the [HCL syntax
/// specification](https://github.com/hashicorp/hcl/blob/main/hclsyntax/spec.md#operations).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinaryOperator {
    /// Equal operator (`==`).
    Eq,
    /// Not-equal operator (`!=`).
    NotEq,
    /// Less-equal operator (`<=`).
    LessEq,
    /// Greater-equal operator (`>=`).
    GreaterEq,
    /// Less operator (`<`).
    Less,
    /// Greater operator (`>`).
    Greater,
    /// Plus operator (`+`).
    Plus,
    /// Minus operator (`-`).
    Minus,
    /// Multiply operator (`*`).
    Mul,
    /// Division operator (`/`).
    Div,
    /// Modulo operator (`%`).
    Mod,
    /// And operator (`&&`).
    And,
    /// Or operator (`||`).
    Or,
}

impl BinaryOperator {
    /// Returns the `BinaryOperator` as a static `&str`.
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryOperator::Eq => "==",
            BinaryOperator::NotEq => "!=",
            BinaryOperator::LessEq => "<=",
            BinaryOperator::GreaterEq => ">=",
            BinaryOperator::Less => "<",
            BinaryOperator::Greater => ">",
            BinaryOperator::Plus => "+",
            BinaryOperator::Minus => "-",
            BinaryOperator::Mul => "*",
            BinaryOperator::Div => "/",
            BinaryOperator::Mod => "%",
            BinaryOperator::And => "&&",
            BinaryOperator::Or => "||",
        }
    }

    /// Returns the operator precedence level. Higher numbers mean higher precedence.
    pub fn precedence(self) -> u8 {
        match self {
            BinaryOperator::Mul | BinaryOperator::Div | BinaryOperator::Mod => 6,
            BinaryOperator::Plus | BinaryOperator::Minus => 5,
            BinaryOperator::LessEq
            | BinaryOperator::GreaterEq
            | BinaryOperator::Less
            | BinaryOperator::Greater => 4,
            BinaryOperator::Eq | BinaryOperator::NotEq => 3,
            BinaryOperator::And => 2,
            BinaryOperator::Or => 1,
        }
    }
}

impl fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for BinaryOperator {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "==" => Ok(BinaryOperator::Eq),
            "!=" => Ok(BinaryOperator::NotEq),
            "<=" => Ok(BinaryOperator::LessEq),
            ">=" => Ok(BinaryOperator::GreaterEq),
            "<" => Ok(BinaryOperator::Less),
            ">" => Ok(BinaryOperator::Greater),
            "+" => Ok(BinaryOperator::Plus),
            "-" => Ok(BinaryOperator::Minus),
            "*" => Ok(BinaryOperator::Mul),
            "/" => Ok(BinaryOperator::Div),
            "%" => Ok(BinaryOperator::Mod),
            "&&" => Ok(BinaryOperator::And),
            "||" => Ok(BinaryOperator::Or),
            _ => Err(Error::new(format!("invalid binary operator: `{s}`"))),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for UnaryOperator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for BinaryOperator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for UnaryOperator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(crate::de::FromStrVisitor::<Self>::new("unary operator"))
    }
}

#[cfg(feature = "serde")]
impl serde::de::IntoDeserializer<'_, Error> for UnaryOperator {
    type Deserializer = serde::de::value::StrDeserializer<'static, Error>;

    fn into_deserializer(self) -> Self::Deserializer {
        self.as_str().into_deserializer()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for BinaryOperator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(crate::de::FromStrVisitor::<Self>::new("binary operator"))
    }
}

#[cfg(feature = "serde")]
impl serde::de::IntoDeserializer<'_, Error> for BinaryOperator {
    type Deserializer = serde::de::value::StrDeserializer<'static, Error>;

    fn into_deserializer(self) -> Self::Deserializer {
        self.as_str().into_deserializer()
    }
}

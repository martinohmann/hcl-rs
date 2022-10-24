use super::{expression::ExpressionSerializer, IdentifierSerializer, SeqSerializer};
use crate::{Error, Expression, Result, Traversal, TraversalOperator};
use serde::ser::{self, Impossible, Serialize};

pub struct TraversalSerializer;

impl ser::Serializer for TraversalSerializer {
    type Ok = Traversal;
    type Error = Error;

    type SerializeSeq = Impossible<Traversal, Error>;
    type SerializeTuple = Impossible<Traversal, Error>;
    type SerializeTupleStruct = Impossible<Traversal, Error>;
    type SerializeTupleVariant = Impossible<Traversal, Error>;
    type SerializeMap = Impossible<Traversal, Error>;
    type SerializeStruct = SerializeTraversalStruct;
    type SerializeStructVariant = Impossible<Traversal, Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        char str bytes none unit unit_struct unit_variant
        newtype_variant seq tuple tuple_struct tuple_variant map struct_variant
    }
    serialize_self! { some newtype_struct }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeTraversalStruct::new())
    }
}

pub struct SerializeTraversalStruct {
    expr: Option<Expression>,
    operators: Option<Vec<TraversalOperator>>,
}

impl SerializeTraversalStruct {
    pub fn new() -> Self {
        SerializeTraversalStruct {
            expr: None,
            operators: None,
        }
    }
}

impl ser::SerializeStruct for SerializeTraversalStruct {
    type Ok = Traversal;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match key {
            "expr" => self.expr = Some(value.serialize(ExpressionSerializer)?),
            "operators" => {
                self.operators =
                    Some(value.serialize(SeqSerializer::new(TraversalOperatorSerializer))?)
            }
            _ => {
                return Err(ser::Error::custom(
                    "expected struct with fields `expr` and `operators`",
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.expr, self.operators) {
            (Some(expr), Some(operators)) => Ok(Traversal { expr, operators }),
            (_, _) => Err(ser::Error::custom(
                "expected struct with fields `expr` and `operators`",
            )),
        }
    }
}

#[derive(Clone)]
struct TraversalOperatorSerializer;

impl ser::Serializer for TraversalOperatorSerializer {
    type Ok = TraversalOperator;
    type Error = Error;

    type SerializeSeq = Impossible<TraversalOperator, Error>;
    type SerializeTuple = Impossible<TraversalOperator, Error>;
    type SerializeTupleStruct = Impossible<TraversalOperator, Error>;
    type SerializeTupleVariant = Impossible<TraversalOperator, Error>;
    type SerializeMap = Impossible<TraversalOperator, Error>;
    type SerializeStruct = Impossible<TraversalOperator, Error>;
    type SerializeStructVariant = Impossible<TraversalOperator, Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 f32 f64
        char str bytes none unit unit_struct
        seq tuple tuple_struct tuple_variant
        map struct struct_variant
    }
    serialize_self! { some }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        Ok(TraversalOperator::LegacyIndex(v))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        match (name, variant) {
            ("$hcl::traversal_operator", "AttrSplat") => Ok(TraversalOperator::AttrSplat),
            ("$hcl::traversal_operator", "FullSplat") => Ok(TraversalOperator::FullSplat),
            (_, _) => Ok(TraversalOperator::GetAttr(variant.into())),
        }
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        if name == "$hcl::expression" {
            Ok(TraversalOperator::Index(
                value.serialize(ExpressionSerializer)?,
            ))
        } else {
            value.serialize(self)
        }
    }

    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        if name == "$hcl::identifier" {
            Ok(TraversalOperator::GetAttr(
                value.serialize(IdentifierSerializer)?,
            ))
        } else {
            value.serialize(self)
        }
    }
}

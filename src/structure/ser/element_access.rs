use super::{expression::ExpressionSerializer, StringSerializer};
use crate::{
    serialize_unsupported, structure::Identifier, ElementAccess, ElementAccessOperator, Error,
    Expression, Result,
};
use serde::ser::{self, Impossible, Serialize};

pub struct ElementAccessSerializer;

impl ser::Serializer for ElementAccessSerializer {
    type Ok = ElementAccess;
    type Error = Error;

    type SerializeSeq = Impossible<ElementAccess, Error>;
    type SerializeTuple = Impossible<ElementAccess, Error>;
    type SerializeTupleStruct = Impossible<ElementAccess, Error>;
    type SerializeTupleVariant = Impossible<ElementAccess, Error>;
    type SerializeMap = Impossible<ElementAccess, Error>;
    type SerializeStruct = SerializeElementAccessStruct;
    type SerializeStructVariant = Impossible<ElementAccess, Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        char str bytes none unit unit_struct unit_variant
        newtype_variant seq tuple tuple_struct tuple_variant map struct_variant
    }
    serialize_self! { some newtype_struct }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeElementAccessStruct::new())
    }
}

pub struct SerializeElementAccessStruct {
    expr: Option<Expression>,
    operator: Option<ElementAccessOperator>,
}

impl SerializeElementAccessStruct {
    pub fn new() -> Self {
        SerializeElementAccessStruct {
            expr: None,
            operator: None,
        }
    }
}

impl ser::SerializeStruct for SerializeElementAccessStruct {
    type Ok = ElementAccess;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match key {
            "expr" => self.expr = Some(value.serialize(ExpressionSerializer)?),
            "operator" => self.operator = Some(value.serialize(ElementAccessOperatorSerializer)?),
            _ => {
                return Err(ser::Error::custom(
                    "expected struct with fields `expr` and `operator`",
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.expr, self.operator) {
            (Some(expr), Some(operator)) => Ok(ElementAccess::new(expr, operator)),
            (_, _) => Err(ser::Error::custom(
                "expected struct with fields `expr` and `operator`",
            )),
        }
    }
}

#[derive(Clone)]
struct ElementAccessOperatorSerializer;

impl ser::Serializer for ElementAccessOperatorSerializer {
    type Ok = ElementAccessOperator;
    type Error = Error;

    type SerializeSeq = Impossible<ElementAccessOperator, Error>;
    type SerializeTuple = Impossible<ElementAccessOperator, Error>;
    type SerializeTupleStruct = Impossible<ElementAccessOperator, Error>;
    type SerializeTupleVariant = Impossible<ElementAccessOperator, Error>;
    type SerializeMap = Impossible<ElementAccessOperator, Error>;
    type SerializeStruct = Impossible<ElementAccessOperator, Error>;
    type SerializeStructVariant = Impossible<ElementAccessOperator, Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 f32 f64
        char str bytes none unit unit_struct
        seq tuple tuple_struct tuple_variant
        map struct struct_variant
    }
    serialize_self! { some }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        Ok(ElementAccessOperator::LegacyIndex(v))
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        match (name, variant) {
            ("$hcl::element_access_operator", "AttrSplat") => Ok(ElementAccessOperator::AttrSplat),
            ("$hcl::element_access_operator", "FullSplat") => Ok(ElementAccessOperator::FullSplat),
            (_, _) => Ok(ElementAccessOperator::GetAttr(variant.into())),
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
            Ok(ElementAccessOperator::Index(
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
            Ok(ElementAccessOperator::GetAttr(Identifier::from(
                value.serialize(StringSerializer)?,
            )))
        } else {
            value.serialize(self)
        }
    }
}

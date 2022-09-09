use super::expression::ExpressionSerializer;
use crate::{Conditional, Error, Expression, Result};
use serde::ser::{self, Impossible};

pub struct ConditionalSerializer;

impl ser::Serializer for ConditionalSerializer {
    type Ok = Conditional;
    type Error = Error;

    type SerializeSeq = SerializeConditionalSeq;
    type SerializeTuple = SerializeConditionalSeq;
    type SerializeTupleStruct = SerializeConditionalSeq;
    type SerializeTupleVariant = Impossible<Conditional, Error>;
    type SerializeMap = Impossible<Conditional, Error>;
    type SerializeStruct = SerializeConditionalStruct;
    type SerializeStructVariant = Impossible<Conditional, Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        char str bytes none unit unit_struct unit_variant
        newtype_variant tuple_variant map struct_variant
    }
    serialize_self! { some newtype_struct }
    forward_to_serialize_seq! { tuple tuple_struct }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeConditionalSeq::new())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeConditionalStruct::new())
    }
}

pub struct SerializeConditionalSeq {
    predicate: Option<Expression>,
    true_expr: Option<Expression>,
    false_expr: Option<Expression>,
}

impl SerializeConditionalSeq {
    pub fn new() -> Self {
        SerializeConditionalSeq {
            predicate: None,
            true_expr: None,
            false_expr: None,
        }
    }
}

impl ser::SerializeSeq for SerializeConditionalSeq {
    type Ok = Conditional;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.predicate.is_none() {
            self.predicate = Some(value.serialize(ExpressionSerializer)?);
        } else if self.true_expr.is_none() {
            self.true_expr = Some(value.serialize(ExpressionSerializer)?);
        } else if self.false_expr.is_none() {
            self.false_expr = Some(value.serialize(ExpressionSerializer)?);
        } else {
            return Err(ser::Error::custom("expected sequence with 3 elements"));
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.predicate, self.true_expr, self.false_expr) {
            (Some(predicate), Some(true_expr), Some(false_expr)) => Ok(Conditional {
                predicate,
                true_expr,
                false_expr,
            }),
            (_, _, _) => Err(ser::Error::custom("expected sequence with 3 elements")),
        }
    }
}

impl ser::SerializeTuple for SerializeConditionalSeq {
    impl_forward_to_serialize_seq!(serialize_element, Conditional);
}

impl ser::SerializeTupleStruct for SerializeConditionalSeq {
    impl_forward_to_serialize_seq!(serialize_field, Conditional);
}

pub struct SerializeConditionalStruct {
    predicate: Option<Expression>,
    true_expr: Option<Expression>,
    false_expr: Option<Expression>,
}

impl SerializeConditionalStruct {
    pub fn new() -> Self {
        SerializeConditionalStruct {
            predicate: None,
            true_expr: None,
            false_expr: None,
        }
    }
}

impl ser::SerializeStruct for SerializeConditionalStruct {
    type Ok = Conditional;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match key {
            "predicate" => self.predicate = Some(value.serialize(ExpressionSerializer)?),
            "true_expr" => self.true_expr = Some(value.serialize(ExpressionSerializer)?),
            "false_expr" => self.false_expr = Some(value.serialize(ExpressionSerializer)?),
            _ => {
                return Err(ser::Error::custom(
                    "expected struct with fields `predicate`, `true_expr` and `false_expr`",
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.predicate, self.true_expr, self.false_expr) {
            (Some(predicate), Some(true_expr), Some(false_expr)) => Ok(Conditional {
                predicate,
                true_expr,
                false_expr,
            }),
            (_, _, _) => Err(ser::Error::custom(
                "expected struct with fields `predicate`, `true_expr` and `false_expr`",
            )),
        }
    }
}

use super::expression::ExpressionSerializer;
use crate::ser::FromStrSerializer;
use crate::{
    BinaryOp, BinaryOperator, Error, Expression, Operation, Result, UnaryOp, UnaryOperator,
};
use serde::ser::{self, Impossible, Serialize};

pub struct OperationSerializer;

impl ser::Serializer for OperationSerializer {
    type Ok = Operation;
    type Error = Error;

    type SerializeSeq = SerializeOperationSeq;
    type SerializeTuple = SerializeOperationSeq;
    type SerializeTupleStruct = SerializeOperationSeq;
    type SerializeTupleVariant = Impossible<Operation, Error>;
    type SerializeMap = Impossible<Operation, Error>;
    type SerializeStruct = SerializeOperationStruct;
    type SerializeStructVariant = Impossible<Operation, Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        char str bytes none unit unit_struct unit_variant
        tuple_variant map struct_variant
    }
    serialize_self! { some newtype_struct }
    forward_to_serialize_seq! { tuple tuple_struct }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        // Specialization for the `Operation` type itself.
        match (name, variant) {
            ("$hcl::operation", "Binary") => {
                Ok(Operation::Binary(value.serialize(BinaryOpSerializer)?))
            }
            (_, _) => Ok(Operation::Unary(value.serialize(UnaryOpSerializer)?)),
        }
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeOperationSeq::new(len))
    }

    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeOperationStruct::new(name))
    }
}

pub enum SerializeOperationSeq {
    Unary(SerializeUnaryOpSeq),
    Binary(SerializeBinaryOpSeq),
}

impl SerializeOperationSeq {
    fn new(len: Option<usize>) -> Self {
        // Specialization for the `BinaryOp` type.
        if let Some(3) = len {
            SerializeOperationSeq::Binary(SerializeBinaryOpSeq::new())
        } else {
            SerializeOperationSeq::Unary(SerializeUnaryOpSeq::new())
        }
    }
}

impl ser::SerializeSeq for SerializeOperationSeq {
    type Ok = Operation;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match self {
            SerializeOperationSeq::Unary(attr) => attr.serialize_element(value),
            SerializeOperationSeq::Binary(block) => block.serialize_element(value),
        }
    }

    fn end(self) -> Result<Self::Ok> {
        match self {
            SerializeOperationSeq::Unary(attr) => attr.end().map(Into::into),
            SerializeOperationSeq::Binary(block) => block.end().map(Into::into),
        }
    }
}

impl ser::SerializeTuple for SerializeOperationSeq {
    impl_forward_to_serialize_seq!(serialize_element, Operation);
}

impl ser::SerializeTupleStruct for SerializeOperationSeq {
    impl_forward_to_serialize_seq!(serialize_field, Operation);
}

pub enum SerializeOperationStruct {
    Unary(SerializeUnaryOpStruct),
    Binary(SerializeBinaryOpStruct),
}

impl SerializeOperationStruct {
    pub fn new(name: &'static str) -> Self {
        // Specialization for the `UnaryOp` and `BinaryOp` types.
        match name {
            "$hcl::binary_op" => SerializeOperationStruct::Binary(SerializeBinaryOpStruct::new()),
            _ => SerializeOperationStruct::Unary(SerializeUnaryOpStruct::new()),
        }
    }
}

impl ser::SerializeStruct for SerializeOperationStruct {
    type Ok = Operation;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match self {
            SerializeOperationStruct::Unary(ser) => ser.serialize_field(key, value),
            SerializeOperationStruct::Binary(ser) => ser.serialize_field(key, value),
        }
    }

    fn end(self) -> Result<Self::Ok> {
        match self {
            SerializeOperationStruct::Unary(ser) => ser.end().map(Into::into),
            SerializeOperationStruct::Binary(ser) => ser.end().map(Into::into),
        }
    }
}

pub struct UnaryOpSerializer;

impl ser::Serializer for UnaryOpSerializer {
    type Ok = UnaryOp;
    type Error = Error;

    type SerializeSeq = SerializeUnaryOpSeq;
    type SerializeTuple = SerializeUnaryOpSeq;
    type SerializeTupleStruct = SerializeUnaryOpSeq;
    type SerializeTupleVariant = Impossible<UnaryOp, Error>;
    type SerializeMap = Impossible<UnaryOp, Error>;
    type SerializeStruct = SerializeUnaryOpStruct;
    type SerializeStructVariant = Impossible<UnaryOp, Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        char str bytes none unit unit_struct unit_variant
        newtype_variant tuple_variant map struct_variant
    }
    serialize_self! { some newtype_struct }
    forward_to_serialize_seq! { tuple tuple_struct }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeUnaryOpSeq::new())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeUnaryOpStruct::new())
    }
}

pub struct SerializeUnaryOpSeq {
    operator: Option<UnaryOperator>,
    expr: Option<Expression>,
}

impl SerializeUnaryOpSeq {
    pub fn new() -> Self {
        SerializeUnaryOpSeq {
            operator: None,
            expr: None,
        }
    }
}

impl ser::SerializeSeq for SerializeUnaryOpSeq {
    type Ok = UnaryOp;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.operator.is_none() {
            self.operator = Some(value.serialize(FromStrSerializer::new())?);
        } else if self.expr.is_none() {
            self.expr = Some(value.serialize(ExpressionSerializer)?);
        } else {
            return Err(ser::Error::custom("expected sequence with 2 elements"));
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.operator, self.expr) {
            (Some(operator), Some(expr)) => Ok(UnaryOp { operator, expr }),
            (_, _) => Err(ser::Error::custom("expected sequence with 2 elements")),
        }
    }
}

impl ser::SerializeTuple for SerializeUnaryOpSeq {
    impl_forward_to_serialize_seq!(serialize_element, UnaryOp);
}

impl ser::SerializeTupleStruct for SerializeUnaryOpSeq {
    impl_forward_to_serialize_seq!(serialize_field, UnaryOp);
}

pub struct SerializeUnaryOpStruct {
    operator: Option<UnaryOperator>,
    expr: Option<Expression>,
}

impl SerializeUnaryOpStruct {
    pub fn new() -> Self {
        SerializeUnaryOpStruct {
            operator: None,
            expr: None,
        }
    }
}

impl ser::SerializeStruct for SerializeUnaryOpStruct {
    type Ok = UnaryOp;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match key {
            "operator" => self.operator = Some(value.serialize(FromStrSerializer::new())?),
            "expr" => self.expr = Some(value.serialize(ExpressionSerializer)?),
            _ => {
                return Err(ser::Error::custom(
                    "expected struct with fields `operator` and `expr`",
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.operator, self.expr) {
            (Some(operator), Some(expr)) => Ok(UnaryOp { operator, expr }),
            (_, _) => Err(ser::Error::custom(
                "expected struct with fields `operator` and `expr`",
            )),
        }
    }
}

pub struct BinaryOpSerializer;

impl ser::Serializer for BinaryOpSerializer {
    type Ok = BinaryOp;
    type Error = Error;

    type SerializeSeq = SerializeBinaryOpSeq;
    type SerializeTuple = SerializeBinaryOpSeq;
    type SerializeTupleStruct = SerializeBinaryOpSeq;
    type SerializeTupleVariant = Impossible<BinaryOp, Error>;
    type SerializeMap = Impossible<BinaryOp, Error>;
    type SerializeStruct = SerializeBinaryOpStruct;
    type SerializeStructVariant = Impossible<BinaryOp, Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        char str bytes none unit unit_struct unit_variant
        newtype_variant tuple_variant map struct_variant
    }
    serialize_self! { some newtype_struct }
    forward_to_serialize_seq! { tuple tuple_struct }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeBinaryOpSeq::new())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeBinaryOpStruct::new())
    }
}

pub struct SerializeBinaryOpSeq {
    lhs_expr: Option<Expression>,
    operator: Option<BinaryOperator>,
    rhs_expr: Option<Expression>,
}

impl SerializeBinaryOpSeq {
    pub fn new() -> Self {
        SerializeBinaryOpSeq {
            lhs_expr: None,
            operator: None,
            rhs_expr: None,
        }
    }
}

impl ser::SerializeSeq for SerializeBinaryOpSeq {
    type Ok = BinaryOp;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.lhs_expr.is_none() {
            self.lhs_expr = Some(value.serialize(ExpressionSerializer)?);
        } else if self.operator.is_none() {
            self.operator = Some(value.serialize(FromStrSerializer::new())?);
        } else if self.rhs_expr.is_none() {
            self.rhs_expr = Some(value.serialize(ExpressionSerializer)?);
        } else {
            return Err(ser::Error::custom("expected sequence with 3 elements"));
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.lhs_expr, self.operator, self.rhs_expr) {
            (Some(lhs_expr), Some(operator), Some(rhs_expr)) => Ok(BinaryOp {
                lhs_expr,
                operator,
                rhs_expr,
            }),
            (_, _, _) => Err(ser::Error::custom("expected sequence with 3 elements")),
        }
    }
}

impl ser::SerializeTuple for SerializeBinaryOpSeq {
    impl_forward_to_serialize_seq!(serialize_element, BinaryOp);
}

impl ser::SerializeTupleStruct for SerializeBinaryOpSeq {
    impl_forward_to_serialize_seq!(serialize_field, BinaryOp);
}

pub struct SerializeBinaryOpStruct {
    lhs_expr: Option<Expression>,
    operator: Option<BinaryOperator>,
    rhs_expr: Option<Expression>,
}

impl SerializeBinaryOpStruct {
    pub fn new() -> Self {
        SerializeBinaryOpStruct {
            lhs_expr: None,
            operator: None,
            rhs_expr: None,
        }
    }
}

impl ser::SerializeStruct for SerializeBinaryOpStruct {
    type Ok = BinaryOp;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match key {
            "lhs_expr" => self.lhs_expr = Some(value.serialize(ExpressionSerializer)?),
            "operator" => self.operator = Some(value.serialize(FromStrSerializer::new())?),
            "rhs_expr" => self.rhs_expr = Some(value.serialize(ExpressionSerializer)?),
            _ => {
                return Err(ser::Error::custom(
                    "expected struct with fields `lhs_expr`, `operator` and `rhs_expr`",
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.lhs_expr, self.operator, self.rhs_expr) {
            (Some(lhs_expr), Some(operator), Some(rhs_expr)) => Ok(BinaryOp {
                lhs_expr,
                operator,
                rhs_expr,
            }),
            (_, _, _) => Err(ser::Error::custom(
                "expected struct with fields `lhs_expr`, `operator` and `rhs_expr`",
            )),
        }
    }
}

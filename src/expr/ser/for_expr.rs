use super::ExpressionSerializer;
use crate::expr::{Expression, ForExpr};
use crate::ser::{BoolSerializer, IdentifierSerializer, OptionSerializer};
use crate::{Error, Identifier, Result};
use serde::ser::{self, Impossible};

pub struct ForExprSerializer;

impl ser::Serializer for ForExprSerializer {
    type Ok = ForExpr;
    type Error = Error;

    type SerializeSeq = Impossible<ForExpr, Error>;
    type SerializeTuple = Impossible<ForExpr, Error>;
    type SerializeTupleStruct = Impossible<ForExpr, Error>;
    type SerializeTupleVariant = Impossible<ForExpr, Error>;
    type SerializeMap = Impossible<ForExpr, Error>;
    type SerializeStruct = SerializeForExprStruct;
    type SerializeStructVariant = Impossible<ForExpr, Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str
        bytes none unit unit_struct unit_variant newtype_variant
        seq tuple tuple_struct tuple_variant map struct_variant
    }
    serialize_self! { some newtype_struct }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeForExprStruct::new())
    }
}

pub struct SerializeForExprStruct {
    key_var: Option<Option<Identifier>>,
    value_var: Option<Identifier>,
    collection_expr: Option<Expression>,
    key_expr: Option<Option<Expression>>,
    value_expr: Option<Expression>,
    grouping: Option<bool>,
    cond_expr: Option<Option<Expression>>,
}

impl SerializeForExprStruct {
    pub fn new() -> Self {
        SerializeForExprStruct {
            key_var: None,
            value_var: None,
            collection_expr: None,
            key_expr: None,
            value_expr: None,
            grouping: None,
            cond_expr: None,
        }
    }
}

impl ser::SerializeStruct for SerializeForExprStruct {
    type Ok = ForExpr;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match key {
            "key_var" => {
                self.key_var = Some(value.serialize(OptionSerializer::new(IdentifierSerializer))?);
            }
            "value_var" => {
                self.value_var = Some(value.serialize(IdentifierSerializer)?);
            }
            "collection_expr" => {
                self.collection_expr = Some(value.serialize(ExpressionSerializer)?);
            }
            "key_expr" => self.key_expr = Some(value.serialize(OptionSerializer::new(ExpressionSerializer))?),
            "value_expr" => self.value_expr = Some(value.serialize(ExpressionSerializer)?),
            "grouping" => self.grouping = Some(value.serialize(BoolSerializer)?),
            "cond_expr" => self.cond_expr = Some(value.serialize(OptionSerializer::new(ExpressionSerializer))?),
            _ => {
                return Err(ser::Error::custom(
                    "expected struct with fields `key_var`, `value_var`, `collection_expr`, `key_expr`, `value_expr`, `grouping` and `cond_expr`",
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (
            self.key_var,
            self.value_var,
            self.collection_expr,
            self.key_expr,
            self.value_expr,
            self.grouping,
            self.cond_expr
        ) {
            (
                Some(key_var),
                Some(value_var),
                Some(collection_expr),
                Some(key_expr),
                Some(value_expr),
                Some(grouping),
                Some(cond_expr)
            ) => Ok(ForExpr {
                key_var,
                value_var,
                collection_expr,
                key_expr,
                value_expr,
                grouping,
                cond_expr,
            }),
            (_, _, _, _, _, _, _) => Err(ser::Error::custom(
                "expected struct with fields `key_var`, `value_var`, `collection_expr`, `key_expr`, `value_expr`, `grouping` and `cond_expr`",
            )),
        }
    }
}

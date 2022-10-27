use super::{
    expression::ExpressionSerializer, BoolSerializer, IdentifierSerializer, SeqSerializer,
};
use crate::{Error, Expression, FuncCall, Identifier, Result};
use serde::ser::{self, Impossible};

pub struct FuncCallSerializer;

impl ser::Serializer for FuncCallSerializer {
    type Ok = FuncCall;
    type Error = Error;

    type SerializeSeq = Impossible<FuncCall, Error>;
    type SerializeTuple = Impossible<FuncCall, Error>;
    type SerializeTupleStruct = Impossible<FuncCall, Error>;
    type SerializeTupleVariant = Impossible<FuncCall, Error>;
    type SerializeMap = Impossible<FuncCall, Error>;
    type SerializeStruct = SerializeFuncCallStruct;
    type SerializeStructVariant = Impossible<FuncCall, Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        char str bytes none unit unit_struct unit_variant
        newtype_variant seq tuple tuple_struct tuple_variant map struct_variant
    }
    serialize_self! { some newtype_struct }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeFuncCallStruct::new())
    }
}

pub struct SerializeFuncCallStruct {
    name: Option<Identifier>,
    args: Option<Vec<Expression>>,
    expand_final: Option<bool>,
}

impl SerializeFuncCallStruct {
    pub fn new() -> Self {
        SerializeFuncCallStruct {
            name: None,
            args: None,
            expand_final: None,
        }
    }
}

impl ser::SerializeStruct for SerializeFuncCallStruct {
    type Ok = FuncCall;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match key {
            "name" => self.name = Some(value.serialize(IdentifierSerializer)?),
            "args" => self.args = Some(value.serialize(SeqSerializer::new(ExpressionSerializer))?),
            "expand_final" => self.expand_final = Some(value.serialize(BoolSerializer)?),
            _ => {
                return Err(ser::Error::custom(
                    "expected struct with fields `name`, `args` and optional `expand_final`",
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.name, self.args) {
            (Some(name), Some(args)) => Ok(FuncCall {
                name,
                args,
                expand_final: self.expand_final.unwrap_or_default(),
            }),
            (_, _) => Err(ser::Error::custom(
                "expected struct with fields `name`, `args` and optional `expand_final`",
            )),
        }
    }
}

use super::{expression::ExpressionSerializer, BoolSerializer, OptionSerializer, StringSerializer};
use crate::{Error, Expression, ForExpr, ForIntro, ForListExpr, ForObjectExpr, Identifier, Result};
use serde::ser::{self, Impossible, Serialize};

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
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        char str bytes none unit unit_struct unit_variant
        seq tuple tuple_struct tuple_variant map struct_variant
    }
    serialize_self! { some newtype_struct }

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
        // Specialization for the `ForExpr` type itself.
        match (name, variant) {
            ("$hcl::for_expr", "Object") => {
                Ok(ForExpr::Object(value.serialize(ForObjectExprSerializer)?))
            }
            (_, _) => Ok(ForExpr::List(value.serialize(ForListExprSerializer)?)),
        }
    }

    fn serialize_struct(self, name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeForExprStruct::new(name))
    }
}

pub enum SerializeForExprStruct {
    List(SerializeForListExprStruct),
    Object(SerializeForObjectExprStruct),
}

impl SerializeForExprStruct {
    fn new(name: &'static str) -> Self {
        // Specialization for the `ForListExpr` and `ForObjectExpr` types.
        match name {
            "$hcl::for_object_expr" => {
                SerializeForExprStruct::Object(SerializeForObjectExprStruct::new())
            }
            _ => SerializeForExprStruct::List(SerializeForListExprStruct::new()),
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
        match self {
            SerializeForExprStruct::List(ser) => ser.serialize_field(key, value),
            SerializeForExprStruct::Object(ser) => ser.serialize_field(key, value),
        }
    }

    fn end(self) -> Result<Self::Ok> {
        match self {
            SerializeForExprStruct::List(ser) => ser.end().map(Into::into),
            SerializeForExprStruct::Object(ser) => ser.end().map(Into::into),
        }
    }
}

pub struct ForListExprSerializer;

impl ser::Serializer for ForListExprSerializer {
    type Ok = ForListExpr;
    type Error = Error;

    type SerializeSeq = Impossible<ForListExpr, Error>;
    type SerializeTuple = Impossible<ForListExpr, Error>;
    type SerializeTupleStruct = Impossible<ForListExpr, Error>;
    type SerializeTupleVariant = Impossible<ForListExpr, Error>;
    type SerializeMap = Impossible<ForListExpr, Error>;
    type SerializeStruct = SerializeForListExprStruct;
    type SerializeStructVariant = Impossible<ForListExpr, Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str
        bytes none unit unit_struct unit_variant newtype_variant
        seq tuple tuple_struct tuple_variant map struct_variant
    }
    serialize_self! { some newtype_struct }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeForListExprStruct::new())
    }
}

pub struct SerializeForListExprStruct {
    intro: Option<ForIntro>,
    expr: Option<Expression>,
    cond: Option<Option<Expression>>,
}

impl SerializeForListExprStruct {
    pub fn new() -> Self {
        SerializeForListExprStruct {
            intro: None,
            expr: None,
            cond: None,
        }
    }
}

impl ser::SerializeStruct for SerializeForListExprStruct {
    type Ok = ForListExpr;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match key {
            "intro" => self.intro = Some(value.serialize(ForIntroSerializer)?),
            "expr" => self.expr = Some(value.serialize(ExpressionSerializer)?),
            "cond" => {
                self.cond = Some(value.serialize(OptionSerializer::new(ExpressionSerializer))?)
            }
            _ => {
                return Err(ser::Error::custom(
                    "expected struct with fields `intro`, `expr` and `cond`",
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.intro, self.expr, self.cond) {
            (Some(intro), Some(expr), Some(cond)) => Ok(ForListExpr { intro, expr, cond }),
            (_, _, _) => Err(ser::Error::custom(
                "expected struct with fields `intro`, `expr` and `cond`",
            )),
        }
    }
}

pub struct ForObjectExprSerializer;

impl ser::Serializer for ForObjectExprSerializer {
    type Ok = ForObjectExpr;
    type Error = Error;

    type SerializeSeq = Impossible<ForObjectExpr, Error>;
    type SerializeTuple = Impossible<ForObjectExpr, Error>;
    type SerializeTupleStruct = Impossible<ForObjectExpr, Error>;
    type SerializeTupleVariant = Impossible<ForObjectExpr, Error>;
    type SerializeMap = Impossible<ForObjectExpr, Error>;
    type SerializeStruct = SerializeForObjectExprStruct;
    type SerializeStructVariant = Impossible<ForObjectExpr, Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str
        bytes none unit unit_struct unit_variant newtype_variant
        seq tuple tuple_struct tuple_variant map struct_variant
    }
    serialize_self! { some newtype_struct }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeForObjectExprStruct::new())
    }
}

pub struct SerializeForObjectExprStruct {
    intro: Option<ForIntro>,
    key_expr: Option<Expression>,
    value_expr: Option<Expression>,
    value_grouping: Option<bool>,
    cond: Option<Option<Expression>>,
}

impl SerializeForObjectExprStruct {
    pub fn new() -> Self {
        SerializeForObjectExprStruct {
            intro: None,
            key_expr: None,
            value_expr: None,
            value_grouping: None,
            cond: None,
        }
    }
}

impl ser::SerializeStruct for SerializeForObjectExprStruct {
    type Ok = ForObjectExpr;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match key {
            "intro" => self.intro = Some(value.serialize(ForIntroSerializer)?),
            "key_expr" => self.key_expr = Some(value.serialize(ExpressionSerializer)?),
            "value_expr" => self.value_expr = Some(value.serialize(ExpressionSerializer)?),
            "value_grouping" => self.value_grouping = Some(value.serialize(BoolSerializer)?),
            "cond" => self.cond = Some(value.serialize(OptionSerializer::new(ExpressionSerializer))?),
            _ => {
                return Err(ser::Error::custom(
                    "expected struct with fields `intro`, `key_expr`, `value_expr`, `value_grouping` and `cond`",
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.intro, self.key_expr, self.value_expr, self.value_grouping, self.cond) {
            (Some(intro), Some(key_expr), Some(value_expr), Some(value_grouping), Some(cond)) => {
                Ok(ForObjectExpr {
                    intro,
                    key_expr,
                    value_expr,
                    value_grouping,
                    cond,
                })
            },
            (_, _, _, _, _) => Err(ser::Error::custom(
                "expected struct with fields `intro`, `key_expr`, `value_expr`, `value_grouping` and `cond`",
            )),
        }
    }
}

pub struct ForIntroSerializer;

impl ser::Serializer for ForIntroSerializer {
    type Ok = ForIntro;
    type Error = Error;

    type SerializeSeq = Impossible<ForIntro, Error>;
    type SerializeTuple = Impossible<ForIntro, Error>;
    type SerializeTupleStruct = Impossible<ForIntro, Error>;
    type SerializeTupleVariant = Impossible<ForIntro, Error>;
    type SerializeMap = Impossible<ForIntro, Error>;
    type SerializeStruct = SerializeForIntroStruct;
    type SerializeStructVariant = Impossible<ForIntro, Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str
        bytes none unit unit_struct unit_variant newtype_variant
        seq tuple tuple_struct tuple_variant map struct_variant
    }
    serialize_self! { some newtype_struct }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeForIntroStruct::new())
    }
}

pub struct SerializeForIntroStruct {
    key: Option<Option<Identifier>>,
    value: Option<Identifier>,
    expr: Option<Expression>,
}

impl SerializeForIntroStruct {
    pub fn new() -> Self {
        SerializeForIntroStruct {
            key: None,
            value: None,
            expr: None,
        }
    }
}

impl ser::SerializeStruct for SerializeForIntroStruct {
    type Ok = ForIntro;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match key {
            "key" => {
                let key = value.serialize(OptionSerializer::new(StringSerializer))?;
                self.key = Some(key.map(Identifier::from))
            }
            "value" => self.value = Some(Identifier::from(value.serialize(StringSerializer)?)),
            "expr" => self.expr = Some(value.serialize(ExpressionSerializer)?),
            _ => {
                return Err(ser::Error::custom(
                    "expected struct with fields `key`, `value` and `expr`",
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.key, self.value, self.expr) {
            (Some(key), Some(value), Some(expr)) => Ok(ForIntro { key, value, expr }),
            (_, _, _) => Err(ser::Error::custom(
                "expected struct with fields `key`, `value` and `expr`",
            )),
        }
    }
}

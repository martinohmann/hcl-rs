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
    pub fn new(name: &'static str) -> Self {
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
    element_expr: Option<Expression>,
    cond_expr: Option<Option<Expression>>,
}

impl SerializeForListExprStruct {
    pub fn new() -> Self {
        SerializeForListExprStruct {
            intro: None,
            element_expr: None,
            cond_expr: None,
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
            "element_expr" => self.element_expr = Some(value.serialize(ExpressionSerializer)?),
            "cond_expr" => {
                self.cond_expr = Some(value.serialize(OptionSerializer::new(ExpressionSerializer))?)
            }
            _ => {
                return Err(ser::Error::custom(
                    "expected struct with fields `intro`, `element_expr` and `cond_expr`",
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.intro, self.element_expr, self.cond_expr) {
            (Some(intro), Some(element_expr), Some(cond_expr)) => Ok(ForListExpr {
                intro,
                element_expr,
                cond_expr,
            }),
            (_, _, _) => Err(ser::Error::custom(
                "expected struct with fields `intro`, `element_expr` and `cond_expr`",
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
    grouping: Option<bool>,
    cond_expr: Option<Option<Expression>>,
}

impl SerializeForObjectExprStruct {
    pub fn new() -> Self {
        SerializeForObjectExprStruct {
            intro: None,
            key_expr: None,
            value_expr: None,
            grouping: None,
            cond_expr: None,
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
            "grouping" => self.grouping = Some(value.serialize(BoolSerializer)?),
            "cond_expr" => self.cond_expr = Some(value.serialize(OptionSerializer::new(ExpressionSerializer))?),
            _ => {
                return Err(ser::Error::custom(
                    "expected struct with fields `intro`, `key_expr`, `value_expr`, `grouping` and `cond_expr`",
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.intro, self.key_expr, self.value_expr, self.grouping, self.cond_expr) {
            (Some(intro), Some(key_expr), Some(value_expr), Some(grouping), Some(cond_expr)) => {
                Ok(ForObjectExpr {
                    intro,
                    key_expr,
                    value_expr,
                    grouping,
                    cond_expr,
                })
            },
            (_, _, _, _, _) => Err(ser::Error::custom(
                "expected struct with fields `intro`, `key_expr`, `value_expr`, `grouping` and `cond_expr`",
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
    key_var: Option<Option<Identifier>>,
    value_var: Option<Identifier>,
    collection_expr: Option<Expression>,
}

impl SerializeForIntroStruct {
    pub fn new() -> Self {
        SerializeForIntroStruct {
            key_var: None,
            value_var: None,
            collection_expr: None,
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
            "key_var" => {
                let key = value.serialize(OptionSerializer::new(StringSerializer))?;
                self.key_var = Some(key.map(Identifier::from))
            }
            "value_var" => {
                self.value_var = Some(Identifier::from(value.serialize(StringSerializer)?))
            }
            "collection_expr" => {
                self.collection_expr = Some(value.serialize(ExpressionSerializer)?)
            }
            _ => {
                return Err(ser::Error::custom(
                    "expected struct with fields `key_var`, `value_var` and `collection_expr`",
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.key_var, self.value_var, self.collection_expr) {
            (Some(key_var), Some(value_var), Some(collection_expr)) => Ok(ForIntro {
                key_var,
                value_var,
                collection_expr,
            }),
            (_, _, _) => Err(ser::Error::custom(
                "expected struct with fields `key_var`, `value_var` and `collection_expr`",
            )),
        }
    }
}

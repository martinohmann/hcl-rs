use super::marker;
use super::{Attribute, Block, BlockLabel, Expression, ObjectKey, RawExpression, Structure};
use serde::ser::{Serialize, SerializeMap, SerializeStruct, Serializer};

enum Marker<'a> {
    Raw(&'a RawExpression),
    Ident(&'a str),
    Attribute(&'a Attribute),
    Block(&'a Block),
}

impl<'a> Serialize for Marker<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Marker::Raw(v) => serializer.serialize_newtype_struct(marker::RAW_NAME, v.as_str()),
            Marker::Ident(v) => serializer.serialize_newtype_struct(marker::IDENT_NAME, v),
            Marker::Attribute(v) => serializer.serialize_newtype_struct(marker::ATTRIBUTE_NAME, v),
            Marker::Block(v) => serializer.serialize_newtype_struct(marker::BLOCK_NAME, v),
        }
    }
}

impl Serialize for RawExpression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_str().serialize(serializer)
    }
}

impl Serialize for ObjectKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ObjectKey::String(s) => s.serialize(serializer),
            ObjectKey::Identifier(ident) => Marker::Ident(ident).serialize(serializer),
            ObjectKey::RawExpression(expr) => Marker::Raw(expr).serialize(serializer),
        }
    }
}

impl Serialize for Expression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Expression::Null => serializer.serialize_unit(),
            Expression::Bool(b) => serializer.serialize_bool(*b),
            Expression::Number(n) => n.serialize(serializer),
            Expression::String(s) => serializer.serialize_str(s),
            Expression::Array(v) => v.serialize(serializer),
            Expression::Object(v) => v.serialize(serializer),
            Expression::Raw(expr) => Marker::Raw(expr).serialize(serializer),
        }
    }
}

impl Serialize for Attribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_map(Some(1))?;
        s.serialize_entry(self.key(), self.expr())?;
        s.end()
    }
}

impl Serialize for Block {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct(marker::BLOCK_NAME, 3)?;
        s.serialize_field(marker::IDENT_FIELD, self.identifier())?;
        s.serialize_field(marker::LABELS_FIELD, self.labels())?;
        s.serialize_field(marker::BODY_FIELD, self.body())?;
        s.end()
    }
}

impl Serialize for BlockLabel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            BlockLabel::StringLit(s) => s.serialize(serializer),
            BlockLabel::Identifier(ident) => Marker::Ident(ident).serialize(serializer),
        }
    }
}

impl Serialize for Structure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Structure::Attribute(attr) => Marker::Attribute(attr).serialize(serializer),
            Structure::Block(block) => Marker::Block(block).serialize(serializer),
        }
    }
}

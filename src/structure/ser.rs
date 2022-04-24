use super::marker;
use super::{Attribute, Block, BlockLabel, Expression, ObjectKey, RawExpression, Structure};
use serde::{Serialize, Serializer};

impl Serialize for RawExpression {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut s = serializer.serialize_struct(marker::RAW_EXPRESSION_NAME, 1)?;
        s.serialize_field(marker::RAW_EXPRESSION_FIELD, self.as_str())?;
        s.end()
    }
}

impl Serialize for ObjectKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ObjectKey::String(s) => s.serialize(serializer),
            ObjectKey::Identifier(ident) => {
                use serde::ser::SerializeStruct;

                let mut s = serializer.serialize_struct(marker::IDENT_NAME, 1)?;
                s.serialize_field(marker::IDENT_FIELD, &ident)?;
                s.end()
            }
            ObjectKey::RawExpression(expr) => expr.serialize(serializer),
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
            Expression::Raw(expr) => expr.serialize(serializer),
        }
    }
}

impl Serialize for Attribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let mut s = serializer.serialize_struct(marker::ATTRIBUTE_NAME, 2)?;
        s.serialize_field(marker::IDENT_FIELD, self.key())?;
        s.serialize_field(marker::EXPRESSION_FIELD, self.expr())?;
        s.end()
    }
}

impl Serialize for Block {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        let len = if self.labels().is_empty() { 2 } else { 3 };

        let mut s = serializer.serialize_struct(marker::BLOCK_NAME, len)?;
        s.serialize_field(marker::IDENT_FIELD, self.identifier())?;

        if !self.labels.is_empty() {
            s.serialize_field(marker::BLOCK_LABELS_FIELD, self.labels())?;
        }

        s.serialize_field(marker::BLOCK_BODY_FIELD, self.body())?;
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
            BlockLabel::Identifier(ident) => {
                use serde::ser::SerializeStruct;

                let mut s = serializer.serialize_struct(marker::IDENT_NAME, 1)?;
                s.serialize_field(marker::IDENT_FIELD, &ident)?;
                s.end()
            }
        }
    }
}

impl Serialize for Structure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Structure::Attribute(attr) => attr.serialize(serializer),
            Structure::Block(block) => block.serialize(serializer),
        }
    }
}

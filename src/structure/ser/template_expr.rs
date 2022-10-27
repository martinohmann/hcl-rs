use super::{FromStrSerializer, IdentifierSerializer, StringSerializer};
use crate::{Error, Heredoc, HeredocStripMode, Identifier, Result, TemplateExpr};
use serde::ser::{self, Impossible, Serialize};
use std::fmt::Display;

pub struct TemplateExprSerializer;

impl ser::Serializer for TemplateExprSerializer {
    type Ok = TemplateExpr;
    type Error = Error;

    type SerializeSeq = Impossible<TemplateExpr, Error>;
    type SerializeTuple = Impossible<TemplateExpr, Error>;
    type SerializeTupleStruct = Impossible<TemplateExpr, Error>;
    type SerializeTupleVariant = Impossible<TemplateExpr, Error>;
    type SerializeMap = Impossible<TemplateExpr, Error>;
    type SerializeStruct = SerializeTemplateExprStruct;
    type SerializeStructVariant = Impossible<TemplateExpr, Error>;

    serialize_unsupported! {
        i8 i16 i32 i64 u8 u16 u32 u64
        bool f32 f64 bytes unit unit_struct none
        seq tuple tuple_struct tuple_variant map struct_variant
    }
    serialize_self! { some newtype_struct }

    fn serialize_char(self, value: char) -> Result<Self::Ok> {
        Ok(TemplateExpr::QuotedString(value.to_string()))
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok> {
        Ok(TemplateExpr::QuotedString(value.to_owned()))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        Ok(TemplateExpr::QuotedString(variant.to_owned()))
    }

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
        // Specialization for the `TemplateExpr` type itself.
        match (name, variant) {
            ("$hcl::template_expr", "QuotedString") => Ok(TemplateExpr::QuotedString(
                value.serialize(StringSerializer)?,
            )),
            ("$hcl::template_expr", "Heredoc") => {
                Ok(TemplateExpr::Heredoc(value.serialize(HeredocSerializer)?))
            }
            (_, _) => value.serialize(self),
        }
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeTemplateExprStruct::new())
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Display,
    {
        Ok(TemplateExpr::QuotedString(value.to_string()))
    }
}

pub struct SerializeTemplateExprStruct {
    inner: SerializeHeredocStruct,
}

impl SerializeTemplateExprStruct {
    pub fn new() -> Self {
        SerializeTemplateExprStruct {
            inner: SerializeHeredocStruct::new(),
        }
    }
}

impl ser::SerializeStruct for SerializeTemplateExprStruct {
    impl_forward_to_inner!(TemplateExpr, serialize_field(key: &'static str));
}

pub struct HeredocSerializer;

impl ser::Serializer for HeredocSerializer {
    type Ok = Heredoc;
    type Error = Error;

    type SerializeSeq = SerializeHeredocSeq;
    type SerializeTuple = SerializeHeredocSeq;
    type SerializeTupleStruct = SerializeHeredocSeq;
    type SerializeTupleVariant = Impossible<Heredoc, Error>;
    type SerializeMap = SerializeHeredocMap;
    type SerializeStruct = SerializeHeredocStruct;
    type SerializeStructVariant = Impossible<Heredoc, Error>;

    serialize_unsupported! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
        char str bytes none unit unit_struct unit_variant
        tuple_variant struct_variant
    }
    serialize_self! { some newtype_struct }
    forward_to_serialize_seq! { tuple tuple_struct }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Heredoc>
    where
        T: ?Sized + Serialize,
    {
        Ok(Heredoc {
            delimiter: Identifier::new(variant)?,
            template: value.serialize(StringSerializer)?,
            strip: HeredocStripMode::None,
        })
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeHeredocSeq::new())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeHeredocMap::new())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(SerializeHeredocStruct::new())
    }
}

pub struct SerializeHeredocSeq {
    delimiter: Option<Identifier>,
    template: Option<String>,
    strip: Option<HeredocStripMode>,
}

impl SerializeHeredocSeq {
    pub fn new() -> Self {
        SerializeHeredocSeq {
            delimiter: None,
            template: None,
            strip: None,
        }
    }
}

impl ser::SerializeSeq for SerializeHeredocSeq {
    type Ok = Heredoc;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.delimiter.is_none() {
            self.delimiter = Some(value.serialize(IdentifierSerializer)?);
        } else if self.template.is_none() {
            self.template = Some(value.serialize(StringSerializer)?);
        } else if self.strip.is_none() {
            self.strip = Some(value.serialize(FromStrSerializer::new())?);
        } else {
            return Err(ser::Error::custom("expected sequence with 2 or 3 elements"));
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.delimiter, self.template) {
            (Some(delimiter), Some(template)) => Ok(Heredoc {
                delimiter,
                template,
                strip: self.strip.unwrap_or(HeredocStripMode::None),
            }),
            (_, _) => Err(ser::Error::custom("expected sequence with 2 or 3 elements")),
        }
    }
}

impl ser::SerializeTuple for SerializeHeredocSeq {
    impl_forward_to_serialize_seq!(serialize_element, Heredoc);
}

impl serde::ser::SerializeTupleStruct for SerializeHeredocSeq {
    impl_forward_to_serialize_seq!(serialize_field, Heredoc);
}

pub struct SerializeHeredocMap {
    delimiter: Option<Identifier>,
    template: Option<String>,
}

impl SerializeHeredocMap {
    pub fn new() -> Self {
        SerializeHeredocMap {
            delimiter: None,
            template: None,
        }
    }
}

impl ser::SerializeMap for SerializeHeredocMap {
    type Ok = Heredoc;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.delimiter.is_none() {
            self.delimiter = Some(key.serialize(IdentifierSerializer)?);
            Ok(())
        } else {
            Err(ser::Error::custom("expected map with 1 entry"))
        }
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.template.is_none() {
            panic!("serialize_value called before serialize_key");
        }

        self.template = Some(value.serialize(StringSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.delimiter, self.template) {
            (Some(delimiter), Some(template)) => Ok(Heredoc {
                delimiter,
                template,
                strip: HeredocStripMode::None,
            }),
            (_, _) => Err(ser::Error::custom("expected map with 1 entry")),
        }
    }
}

pub struct SerializeHeredocStruct {
    delimiter: Option<Identifier>,
    template: Option<String>,
    strip: Option<HeredocStripMode>,
}

impl SerializeHeredocStruct {
    pub fn new() -> Self {
        SerializeHeredocStruct {
            delimiter: None,
            template: None,
            strip: None,
        }
    }
}

impl ser::SerializeStruct for SerializeHeredocStruct {
    type Ok = Heredoc;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + ser::Serialize,
    {
        match key {
            "delimiter" => self.delimiter = Some(value.serialize(IdentifierSerializer)?),
            "template" => self.template = Some(value.serialize(StringSerializer)?),
            "strip" => self.strip = Some(value.serialize(FromStrSerializer::new())?),
            _ => {
                return Err(ser::Error::custom(
                    "expected struct with fields `delimiter`, `template` and optional `strip`",
                ))
            }
        };

        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        match (self.delimiter, self.template) {
            (Some(delimiter), Some(template)) => Ok(Heredoc {
                delimiter,
                template,
                strip: self.strip.unwrap_or(HeredocStripMode::None),
            }),
            (_, _) => Err(ser::Error::custom(
                "expected struct with fields `delimiter`, `template` and optional `strip`",
            )),
        }
    }
}

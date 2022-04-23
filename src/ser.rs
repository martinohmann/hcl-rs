//! Serialize a Rust data structure into HCL data.

use crate::{
    format::{Format, PrettyFormatter},
    Error, Result,
};
use serde::ser::{self, Impossible, Serialize, SerializeSeq};
use std::io;

/// A structure for serializing Rust values into HCL.
///
/// Please note that as of current, serialization into HCL blocks is not supported.
pub struct Serializer<W, F> {
    writer: W,
    formatter: F,
}

impl<'a, W> Serializer<W, PrettyFormatter<'a>>
where
    W: io::Write,
{
    /// Creates a new `Serializer` which serializes to the provides writer.
    pub fn new(writer: W) -> Self {
        Serializer::with_formatter(writer, PrettyFormatter::default())
    }
}

impl<W, F> Serializer<W, F>
where
    W: io::Write,
    F: Format,
{
    /// Creates a new `Serializer` which serializes to the provides writer using the provides
    /// formatter.
    pub fn with_formatter(writer: W, formatter: F) -> Serializer<W, F> {
        Serializer { writer, formatter }
    }

    /// Consumes `self` and returns the wrapped writer.
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<'a, W, F> ser::Serializer for &'a mut Serializer<W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, _v: bool) -> Result<()> {
        Err(structure_expected())
    }

    fn serialize_i8(self, _v: i8) -> Result<()> {
        Err(structure_expected())
    }

    fn serialize_i16(self, _v: i16) -> Result<()> {
        Err(structure_expected())
    }

    fn serialize_i32(self, _v: i32) -> Result<()> {
        Err(structure_expected())
    }

    fn serialize_i64(self, _v: i64) -> Result<()> {
        Err(structure_expected())
    }

    fn serialize_u8(self, _v: u8) -> Result<()> {
        Err(structure_expected())
    }

    fn serialize_u16(self, _v: u16) -> Result<()> {
        Err(structure_expected())
    }

    fn serialize_u32(self, _v: u32) -> Result<()> {
        Err(structure_expected())
    }

    fn serialize_u64(self, _v: u64) -> Result<()> {
        Err(structure_expected())
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(structure_expected())
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(structure_expected())
    }

    fn serialize_char(self, _v: char) -> Result<()> {
        Err(structure_expected())
    }

    fn serialize_str(self, _v: &str) -> Result<()> {
        Err(structure_expected())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        Err(structure_expected())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Err(structure_expected())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        variant.serialize(AttributeKeySerializer::new(self))?;
        self.writer.write_all(b" = ")?;
        value.serialize(ValueSerializer::new(self))?;
        self.writer.write_all(b"\n")?;
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(structure_expected())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(structure_expected())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(structure_expected())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        variant.serialize(AttributeKeySerializer::new(self))?;
        self.writer.write_all(b" = ")?;
        self.formatter.begin_array(&mut self.writer)?;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        variant.serialize(AttributeKeySerializer::new(self))?;
        self.writer.write_all(b" = ")?;
        self.formatter.begin_object(&mut self.writer)?;
        Ok(self)
    }
}

impl<'a, W, F> ser::SerializeTupleVariant for &'a mut Serializer<W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.formatter.begin_array_value(&mut self.writer)?;
        value.serialize(ValueSerializer::new(self))?;
        self.formatter.end_array_value(&mut self.writer)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.formatter.end_array(&mut self.writer)?;
        self.writer.write_all(b"\n")?;
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeMap for &'a mut Serializer<W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. HCL
    // only allows string keys so the implementation below will produce invalid
    // HCL if the key serializes as something other than a string.
    //
    // A real HCL serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(AttributeKeySerializer::new(self))
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.writer.write_all(b" = ")?;
        value.serialize(ValueSerializer::new(self))?;
        self.writer.write_all(b"\n")?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a, W, F> ser::SerializeStruct for &'a mut Serializer<W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.formatter.begin_object_key(&mut self.writer)?;
        key.serialize(AttributeKeySerializer::new(self))?;
        self.formatter.begin_object_value(&mut self.writer)?;
        value.serialize(ValueSerializer::new(self))?;
        self.formatter.end_object_value(&mut self.writer)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.formatter.end_object(&mut self.writer)?;
        Ok(())
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a, W, F> ser::SerializeStructVariant for &'a mut Serializer<W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.formatter.begin_object_key(&mut self.writer)?;
        key.serialize(AttributeKeySerializer::new(self))?;
        self.formatter.begin_object_value(&mut self.writer)?;
        value.serialize(ValueSerializer::new(self))?;
        self.formatter.end_object_value(&mut self.writer)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.formatter.end_object(&mut self.writer)?;
        Ok(())
    }
}

struct AttributeKeySerializer<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
}

impl<'a, W, F> AttributeKeySerializer<'a, W, F> {
    fn new(ser: &'a mut Serializer<W, F>) -> AttributeKeySerializer<'a, W, F> {
        AttributeKeySerializer { ser }
    }
}

impl<'a, W, F> ser::Serializer for AttributeKeySerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, _v: bool) -> Result<()> {
        Err(not_an_attribute_key())
    }

    fn serialize_i8(self, _v: i8) -> Result<()> {
        Err(not_an_attribute_key())
    }

    fn serialize_i16(self, _v: i16) -> Result<()> {
        Err(not_an_attribute_key())
    }

    fn serialize_i32(self, _v: i32) -> Result<()> {
        Err(not_an_attribute_key())
    }

    fn serialize_i64(self, _v: i64) -> Result<()> {
        Err(not_an_attribute_key())
    }

    fn serialize_u8(self, _v: u8) -> Result<()> {
        Err(not_an_attribute_key())
    }

    fn serialize_u16(self, _v: u16) -> Result<()> {
        Err(not_an_attribute_key())
    }

    fn serialize_u32(self, _v: u32) -> Result<()> {
        Err(not_an_attribute_key())
    }

    fn serialize_u64(self, _v: u64) -> Result<()> {
        Err(not_an_attribute_key())
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(not_an_attribute_key())
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(not_an_attribute_key())
    }

    // Serialize a char as a single-character string. Other formats may
    // represent this differently.
    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.ser.formatter.write_ident(&mut self.ser.writer, v)?;
        Ok(())
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        Err(not_an_attribute_key())
    }

    // An absent optional is represented as the HCL `null`.
    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    // A present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`. Unfortunately this is typically
    // what people expect when working with HCL. Other formats are encouraged
    // to behave more intelligently if possible.
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data. Map this to
    // HCL as `null`.
    fn serialize_unit(self) -> Result<()> {
        Err(not_an_attribute_key())
    }

    // Unit struct means a named value containing no data. Again, since there is
    // no data, map this to HCL as `null`. There is no need to serialize the
    // name in most formats.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // Serialize this to HCL in externally tagged form as `{ NAME = VALUE }`.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(not_an_attribute_key())
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in HCL is `[`.
    //
    // The length of the sequence may or may not be known ahead of time. This
    // doesn't make a difference in HCL because the length is not represented
    // explicitly in the serialized form. Some serializers may only be able to
    // support sequences for which the length is known up front.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(not_an_attribute_key())
    }

    // Tuples look just like sequences in HCL. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(not_an_attribute_key())
    }

    // Tuple structs look just like sequences in HCL.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(not_an_attribute_key())
    }

    // Tuple variants are represented in HCL as `{ NAME = [DATA...] }`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(not_an_attribute_key())
    }

    // Maps are represented in HCL as `{ K = V, K = V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(not_an_attribute_key())
    }

    // Structs look just like maps in HCL. In particular, HCL requires that we
    // serialize the field names of the struct. Other formats may be able to
    // omit the field names when serializing structs because the corresponding
    // Deserialize implementation is required to know what the keys are without
    // looking at the serialized data.
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(not_an_attribute_key())
    }

    // Struct variants are represented in HCL as `{ NAME = { K = V, ... } }`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(not_an_attribute_key())
    }
}

struct ObjectKeySerializer<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
}

impl<'a, W, F> ObjectKeySerializer<'a, W, F> {
    fn new(ser: &'a mut Serializer<W, F>) -> ObjectKeySerializer<'a, W, F> {
        ObjectKeySerializer { ser }
    }
}

impl<'a, W, F> ser::Serializer for ObjectKeySerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;
    type SerializeTuple = Impossible<(), Error>;
    type SerializeTupleStruct = Impossible<(), Error>;
    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;

    fn serialize_bool(self, _v: bool) -> Result<()> {
        Err(not_an_object_key())
    }

    fn serialize_i8(self, _v: i8) -> Result<()> {
        Err(not_an_object_key())
    }

    fn serialize_i16(self, _v: i16) -> Result<()> {
        Err(not_an_object_key())
    }

    fn serialize_i32(self, _v: i32) -> Result<()> {
        Err(not_an_object_key())
    }

    fn serialize_i64(self, _v: i64) -> Result<()> {
        Err(not_an_object_key())
    }

    fn serialize_u8(self, _v: u8) -> Result<()> {
        Err(not_an_object_key())
    }

    fn serialize_u16(self, _v: u16) -> Result<()> {
        Err(not_an_object_key())
    }

    fn serialize_u32(self, _v: u32) -> Result<()> {
        Err(not_an_object_key())
    }

    fn serialize_u64(self, _v: u64) -> Result<()> {
        Err(not_an_object_key())
    }

    fn serialize_f32(self, _v: f32) -> Result<()> {
        Err(not_an_object_key())
    }

    fn serialize_f64(self, _v: f64) -> Result<()> {
        Err(not_an_object_key())
    }

    // Serialize a char as a single-character string. Other formats may
    // represent this differently.
    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    // This only works for strings that don't require escape sequences but you
    // get the idea. For example it would emit invalid HCL if the input string
    // contains a '"' character.
    fn serialize_str(self, v: &str) -> Result<()> {
        self.ser.formatter.write_str(&mut self.ser.writer, v)?;
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<()> {
        Err(not_an_object_key())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        Err(not_an_object_key())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // Serialize this to HCL in externally tagged form as `{ NAME = VALUE }`.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(not_an_object_key())
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in HCL is `[`.
    //
    // The length of the sequence may or may not be known ahead of time. This
    // doesn't make a difference in HCL because the length is not represented
    // explicitly in the serialized form. Some serializers may only be able to
    // support sequences for which the length is known up front.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(not_an_object_key())
    }

    // Tuples look just like sequences in HCL. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(not_an_object_key())
    }

    // Tuple structs look just like sequences in HCL.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(not_an_object_key())
    }

    // Tuple variants are represented in HCL as `{ NAME = [DATA...] }`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(not_an_object_key())
    }

    // Maps are represented in HCL as `{ K = V, K = V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(not_an_object_key())
    }

    // Structs look just like maps in HCL. In particular, HCL requires that we
    // serialize the field names of the struct. Other formats may be able to
    // omit the field names when serializing structs because the corresponding
    // Deserialize implementation is required to know what the keys are without
    // looking at the serialized data.
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(not_an_object_key())
    }

    // Struct variants are represented in HCL as `{ NAME = { K = V, ... } }`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(not_an_object_key())
    }
}

struct ValueSerializer<'a, W: 'a, F: 'a> {
    ser: &'a mut Serializer<W, F>,
}

impl<'a, W, F> ValueSerializer<'a, W, F> {
    fn new(ser: &'a mut Serializer<W, F>) -> ValueSerializer<'a, W, F> {
        ValueSerializer { ser }
    }
}

impl<'a, W, F> ser::Serializer for ValueSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.ser.formatter.write_bool(&mut self.ser.writer, v)?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.ser.formatter.write_int(&mut self.ser.writer, v)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.ser.formatter.write_int(&mut self.ser.writer, v)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.ser.formatter.write_float(&mut self.ser.writer, v)?;
        Ok(())
    }

    // Serialize a char as a single-character string. Other formats may
    // represent this differently.
    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())
    }

    // This only works for strings that don't require escape sequences but you
    // get the idea. For example it would emit invalid HCL if the input string
    // contains a '"' character.
    fn serialize_str(self, v: &str) -> Result<()> {
        self.ser.formatter.write_str(&mut self.ser.writer, v)?;
        Ok(())
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    // An absent optional is represented as the HCL `null`.
    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    // A present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`. Unfortunately this is typically
    // what people expect when working with HCL. Other formats are encouraged
    // to behave more intelligently if possible.
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data. Map this to
    // HCL as `null`.
    fn serialize_unit(self) -> Result<()> {
        self.ser.formatter.write_null(&mut self.ser.writer)?;
        Ok(())
    }

    // Unit struct means a named value containing no data. Again, since there is
    // no data, map this to HCL as `null`. There is no need to serialize the
    // name in most formats.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // Serialize this to HCL in externally tagged form as `{ NAME = VALUE }`.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser.formatter.begin_object(&mut self.ser.writer)?;
        self.ser.formatter.begin_object_key(&mut self.ser.writer)?;
        variant.serialize(ObjectKeySerializer::new(self.ser))?;
        self.ser
            .formatter
            .begin_object_value(&mut self.ser.writer)?;
        value.serialize(ValueSerializer::new(self.ser))?;
        self.ser.formatter.end_object_value(&mut self.ser.writer)?;
        self.ser.formatter.end_object(&mut self.ser.writer)?;
        Ok(())
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in HCL is `[`.
    //
    // The length of the sequence may or may not be known ahead of time. This
    // doesn't make a difference in HCL because the length is not represented
    // explicitly in the serialized form. Some serializers may only be able to
    // support sequences for which the length is known up front.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.ser.formatter.begin_array(&mut self.ser.writer)?;
        Ok(self)
    }

    // Tuples look just like sequences in HCL. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    // Tuple structs look just like sequences in HCL.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    // Tuple variants are represented in HCL as `{ NAME = [DATA...] }`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.ser.formatter.begin_object(&mut self.ser.writer)?;
        self.ser.formatter.begin_object_key(&mut self.ser.writer)?;
        variant.serialize(ObjectKeySerializer::new(self.ser))?;
        self.ser
            .formatter
            .begin_object_value(&mut self.ser.writer)?;
        self.ser.formatter.begin_array(&mut self.ser.writer)?;
        Ok(self)
    }

    // Maps are represented in HCL as `{ K = V, K = V, ... }`.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        self.ser.formatter.begin_object(&mut self.ser.writer)?;
        Ok(self)
    }

    // Structs look just like maps in HCL. In particular, HCL requires that we
    // serialize the field names of the struct. Other formats may be able to
    // omit the field names when serializing structs because the corresponding
    // Deserialize implementation is required to know what the keys are without
    // looking at the serialized data.
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    // Struct variants are represented in HCL as `{ NAME = { K = V, ... } }`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.ser.formatter.begin_object(&mut self.ser.writer)?;
        self.ser.formatter.begin_object_key(&mut self.ser.writer)?;
        variant.serialize(ObjectKeySerializer::new(self.ser))?;
        self.ser
            .formatter
            .begin_object_value(&mut self.ser.writer)?;
        self.ser.formatter.begin_object(&mut self.ser.writer)?;
        Ok(self)
    }
}

impl<'a, W, F> ser::SerializeSeq for ValueSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser.formatter.begin_array_value(&mut self.ser.writer)?;
        value.serialize(ValueSerializer::new(self.ser))?;
        self.ser.formatter.end_array_value(&mut self.ser.writer)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.ser.formatter.end_array(&mut self.ser.writer)?;
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeTuple for ValueSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser.formatter.begin_array_value(&mut self.ser.writer)?;
        value.serialize(ValueSerializer::new(self.ser))?;
        self.ser.formatter.end_array_value(&mut self.ser.writer)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.ser.formatter.end_array(&mut self.ser.writer)?;
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeTupleStruct for ValueSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser.formatter.begin_array_value(&mut self.ser.writer)?;
        value.serialize(ValueSerializer::new(self.ser))?;
        self.ser.formatter.end_array_value(&mut self.ser.writer)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.ser.formatter.end_array(&mut self.ser.writer)?;
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeTupleVariant for ValueSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser.formatter.begin_array_value(&mut self.ser.writer)?;
        value.serialize(ValueSerializer::new(self.ser))?;
        self.ser.formatter.end_array_value(&mut self.ser.writer)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.ser.formatter.end_array(&mut self.ser.writer)?;
        self.ser.formatter.end_object_value(&mut self.ser.writer)?;
        self.ser.formatter.end_object(&mut self.ser.writer)?;
        Ok(())
    }
}

impl<'a, W, F> ser::SerializeMap for ValueSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. HCL
    // only allows string keys so the implementation below will produce invalid
    // HCL if the key serializes as something other than a string.
    //
    // A real HCL serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser.formatter.begin_object_key(&mut self.ser.writer)?;
        key.serialize(ObjectKeySerializer::new(self.ser))
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser
            .formatter
            .begin_object_value(&mut self.ser.writer)?;
        value.serialize(ValueSerializer::new(self.ser))?;
        self.ser.formatter.end_object_value(&mut self.ser.writer)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.ser.formatter.end_object(&mut self.ser.writer)?;
        Ok(())
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a, W, F> ser::SerializeStruct for ValueSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser.formatter.begin_object_key(&mut self.ser.writer)?;
        key.serialize(ObjectKeySerializer::new(self.ser))?;
        self.ser
            .formatter
            .begin_object_value(&mut self.ser.writer)?;
        value.serialize(ValueSerializer::new(self.ser))?;
        self.ser.formatter.end_object_value(&mut self.ser.writer)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.ser.formatter.end_object(&mut self.ser.writer)?;
        Ok(())
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a, W, F> ser::SerializeStructVariant for ValueSerializer<'a, W, F>
where
    W: io::Write,
    F: Format,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.ser.formatter.begin_object_key(&mut self.ser.writer)?;
        key.serialize(ObjectKeySerializer::new(self.ser))?;
        self.ser
            .formatter
            .begin_object_value(&mut self.ser.writer)?;
        value.serialize(ValueSerializer::new(self.ser))?;
        self.ser.formatter.end_object_value(&mut self.ser.writer)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        self.ser.formatter.end_object(&mut self.ser.writer)?;
        self.ser.formatter.end_object_value(&mut self.ser.writer)?;
        self.ser.formatter.end_object(&mut self.ser.writer)?;
        Ok(())
    }
}

/// Serialize the given value as an HCL byte vector.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: ?Sized + Serialize,
{
    let mut vec = Vec::with_capacity(128);
    to_writer(&mut vec, value)?;
    Ok(vec)
}

/// Serialize the given value as an HCL string.
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let vec = to_vec(value)?;
    let string = unsafe {
        // We do not emit invalid UTF-8.
        String::from_utf8_unchecked(vec)
    };
    Ok(string)
}

/// Serialize the given value as HCL into the IO stream.
///
/// # Errors
///
/// Serialization fails if any operation on the writer fails.
pub fn to_writer<W, T>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: ?Sized + Serialize,
{
    let mut serializer = Serializer::new(writer);
    value.serialize(&mut serializer)
}

fn structure_expected() -> Error {
    Error::new("structure expected")
}

fn not_an_attribute_key() -> Error {
    Error::new("not an attribute key")
}

fn not_an_object_key() -> Error {
    Error::new("not an object key")
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::format::CompactFormatter;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_to_string() {
        let value = json!({
            "foo": [1, 2, 3],
            "bar": "baz",
            "qux": {
                "foo": "bar",
                "baz": "qux"
            }
        });

        let expected = r#"foo = [
  1,
  2,
  3
]
bar = "baz"
qux = {
  "foo" = "bar"
  "baz" = "qux"
}
"#;

        assert_eq!(to_string(&value).unwrap(), expected);
    }

    #[test]
    fn test_to_string_compact() {
        let value = json!({
            "foo": [1, 2, 3],
            "bar": "baz",
            "qux": {
                "foo": "bar",
                "baz": "qux"
            }
        });

        let expected = r#"foo = [1, 2, 3]
bar = "baz"
qux = { "foo" = "bar", "baz" = "qux" }
"#;

        let mut buf = Vec::<u8>::new();

        let mut ser = Serializer::with_formatter(&mut buf, CompactFormatter::default());

        value.serialize(&mut ser).unwrap();

        assert_eq!(std::str::from_utf8(&buf).unwrap(), expected);
    }
}

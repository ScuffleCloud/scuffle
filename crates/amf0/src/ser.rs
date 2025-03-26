use std::io;

use byteorder::{BigEndian, WriteBytesExt};
use serde::Serialize;
use serde::ser::{
    Impossible, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple, SerializeTupleStruct,
    SerializeTupleVariant,
};

use crate::{Amf0Error, Amf0Marker};

pub fn to_writer<W>(writer: W, value: &impl serde::Serialize) -> crate::Result<()>
where
    W: io::Write,
{
    let mut serializer = Serializer { writer };
    value.serialize(&mut serializer)
}

pub fn to_bytes(value: &impl serde::Serialize) -> crate::Result<Vec<u8>> {
    let mut writer = Vec::new();
    to_writer(&mut writer, value)?;
    Ok(writer)
}

pub struct Serializer<W> {
    writer: W,
}

impl<W> serde::ser::Serializer for &mut Serializer<W>
where
    W: io::Write,
{
    type Error = Amf0Error;
    type Ok = ();
    type SerializeMap = Self;
    type SerializeSeq = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.writer.write_u8(Amf0Marker::Boolean as u8)?;
        self.writer.write_u8(v as u8)?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.writer.write_u8(Amf0Marker::Number as u8)?;
        self.writer.write_f64::<BigEndian>(v)?;
        Ok(())
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::UnsupportedType("char"))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        if v.len() <= (u16::MAX as usize) {
            // Normal string
            self.writer.write_u8(Amf0Marker::String as u8)?;
            self.writer.write_u16::<BigEndian>(v.len() as u16)?;
            self.writer.write_all(v.as_bytes())?;
        } else {
            // Long string
            let len: u32 = v.len().try_into()?;

            self.writer.write_u8(Amf0Marker::LongString as u8)?;
            self.writer.write_u32::<BigEndian>(len)?;
            self.writer.write_all(v.as_bytes())?;
        }

        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::UnsupportedType("bytes (&[u8])"))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        // Serialize None as null
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        // Serialize Some as the inner value
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        // Serialize unit as null
        self.writer.write_u8(Amf0Marker::Null as u8)?;
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let len: u32 = len.ok_or(Amf0Error::UnknownLength)?.try_into()?;

        self.writer.write_u8(Amf0Marker::StrictArray as u8)?;
        self.writer.write_u32::<BigEndian>(len)?;
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        // Serialize tuples as arrays
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
        // Serialize tuple structs as arrays
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.writer.write_u8(Amf0Marker::Object as u8)?;
        Ok(self)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        // Serialize unit structs as null
        self.serialize_unit()
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        // Serialize newtype structs as the inner value
        value.serialize(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        // Serialize structs as objects
        self.serialize_map(Some(len))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        // Serialize unit variants as strings
        self.serialize_str(variant)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        self.writer.write_u8(Amf0Marker::Object as u8)?;
        self.writer.write_u16::<BigEndian>(variant.len() as u16)?; // key
        self.writer.write_all(variant.as_bytes())?;
        value.serialize(&mut *self)?; // value
        self.writer.write_u24::<BigEndian>(Amf0Marker::ObjectEnd as u32)?;

        Ok(())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let len: u32 = len.try_into()?;

        self.writer.write_u8(Amf0Marker::Object as u8)?;
        self.writer.write_u16::<BigEndian>(variant.len() as u16)?; // key
        self.writer.write_all(variant.as_bytes())?;
        self.writer.write_u8(Amf0Marker::StrictArray as u8)?; // value
        self.writer.write_u32::<BigEndian>(len)?;

        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.writer.write_u8(Amf0Marker::Object as u8)?;
        self.writer.write_u16::<BigEndian>(variant.len() as u16)?; // key
        self.writer.write_all(variant.as_bytes())?;
        self.writer.write_u8(Amf0Marker::Object as u8)?; // value

        Ok(self)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<W> SerializeSeq for &mut Serializer<W>
where
    W: io::Write,
{
    type Error = Amf0Error;
    type Ok = ();

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W> SerializeTuple for &mut Serializer<W>
where
    W: io::Write,
{
    type Error = Amf0Error;
    type Ok = ();

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W> SerializeTupleStruct for &mut Serializer<W>
where
    W: io::Write,
{
    type Error = Amf0Error;
    type Ok = ();

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<W> SerializeMap for &mut Serializer<W>
where
    W: io::Write,
{
    type Error = Amf0Error;
    type Ok = ();

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        key.serialize(&mut MapKeySerializer {
            writer: &mut self.writer,
        })
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + serde::Serialize,
        V: ?Sized + serde::Serialize,
    {
        self.serialize_key(key)?;
        self.serialize_value(value)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_u24::<BigEndian>(Amf0Marker::ObjectEnd as u32)?;
        Ok(())
    }
}

impl<W> SerializeStruct for &mut Serializer<W>
where
    W: io::Write,
{
    type Error = Amf0Error;
    type Ok = ();

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        key.serialize(&mut MapKeySerializer {
            writer: &mut self.writer,
        })?;
        value.serialize(&mut **self)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_u24::<BigEndian>(Amf0Marker::ObjectEnd as u32)?;
        Ok(())
    }
}

struct MapKeySerializer<W> {
    writer: W,
}

impl<W> serde::ser::Serializer for &mut MapKeySerializer<W>
where
    W: io::Write,
{
    type Error = Amf0Error;
    type Ok = ();
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        let len: u16 = v.len().try_into()?;

        self.writer.write_u16::<BigEndian>(len)?;
        self.writer.write_all(v.as_bytes())?;

        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_tuple_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Amf0Error::MapKeyNotString)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<W> SerializeTupleVariant for &mut Serializer<W>
where
    W: io::Write,
{
    type Error = Amf0Error;
    type Ok = ();

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.writer.write_u24::<BigEndian>(Amf0Marker::ObjectEnd as u32)?;
        Ok(())
    }
}

impl<W> SerializeStructVariant for &mut Serializer<W>
where
    W: io::Write,
{
    type Error = Amf0Error;
    type Ok = ();

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut MapKeySerializer {
            writer: &mut self.writer,
        })?;
        value.serialize(&mut **self)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        // Close the inner value object
        self.writer.write_u24::<BigEndian>(Amf0Marker::ObjectEnd as u32)?;
        // Close the outer object
        self.writer.write_u24::<BigEndian>(Amf0Marker::ObjectEnd as u32)?;
        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use crate::{Amf0Marker, to_bytes};

    #[test]
    fn string() {
        let value = "hello";

        let bytes = to_bytes(&value).unwrap();

        #[rustfmt::skip]
        assert_eq!(
            bytes,
            [
                Amf0Marker::String as u8,
                0, 5, // length
                b'h', b'e', b'l', b'l', b'o',
            ]
        );
    }

    #[test]
    fn bool() {
        let bytes = to_bytes(&true).unwrap();
        assert_eq!(bytes, [Amf0Marker::Boolean as u8, 1]);

        let bytes = to_bytes(&false).unwrap();
        assert_eq!(bytes, [Amf0Marker::Boolean as u8, 0]);
    }

    fn number_test<T>(one: T)
    where
        T: serde::Serialize,
    {
        let bytes = to_bytes(&one).unwrap();
        let mut expected = vec![Amf0Marker::Number as u8];
        expected.extend(&1.0f64.to_be_bytes());
        assert_eq!(bytes, expected);
    }

    #[test]
    fn numbers() {
        number_test(1u8);
        number_test(1u16);
        number_test(1u32);
        number_test(1u64);
        number_test(1i8);
        number_test(1i16);
        number_test(1i32);
        number_test(1i64);
        number_test(1.0f32);
        number_test(1.0f64);
    }

    #[test]
    fn simple_struct() {
        #[derive(serde::Serialize)]
        struct Test {
            a: f64,
            b: String,
        }

        let value = Test {
            a: 1.0,
            b: "hello".to_string(),
        };

        let bytes = to_bytes(&value).unwrap();

        #[rustfmt::skip]
        let mut expected = vec![
            Amf0Marker::Object as u8,
            Amf0Marker::String as u8,
            0, 1, // length
            b'a',
            Amf0Marker::Number as u8,
        ];
        expected.extend(&1.0f64.to_be_bytes());
        #[rustfmt::skip]
        expected.extend_from_slice(&[
            Amf0Marker::String as u8,
            0, 1, // length
            b'b',
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
            0, 0, Amf0Marker::ObjectEnd as u8,
        ]);
        assert_eq!(bytes, expected);
    }

    #[test]
    fn simple_enum() {
        #[derive(serde::Serialize)]
        enum Test {
            A,
            B,
        }

        let value = Test::A;
        let bytes = to_bytes(&value).unwrap();

        #[rustfmt::skip]
        let expected = vec![
            Amf0Marker::String as u8,
            0, 1, // length
            b'A',
        ];
        assert_eq!(bytes, expected);

        let value = Test::B;
        let bytes = to_bytes(&value).unwrap();

        #[rustfmt::skip]
        let expected = vec![
            Amf0Marker::String as u8,
            0, 1, // length
            b'B',
        ];
        assert_eq!(bytes, expected);
    }

    #[test]
    fn complex_enum() {
        #[derive(serde::Serialize)]
        enum Test {
            A(bool),                    // transparent
            B { a: String, b: String }, // object
            C(bool, String),            // array
        }

        let value = Test::A(true);
        let bytes = to_bytes(&value).unwrap();
        #[rustfmt::skip]
        let expected = vec![
            Amf0Marker::Object as u8,
            0, 1, // length
            b'A',
            Amf0Marker::Boolean as u8,
            1,
            0, 0, Amf0Marker::ObjectEnd as u8,
        ];
        assert_eq!(bytes, expected);

        let value = Test::B {
            a: "hello".to_string(),
            b: "world".to_string(),
        };
        let bytes = to_bytes(&value).unwrap();
        #[rustfmt::skip]
        let expected = vec![
            Amf0Marker::Object as u8,
            0, 1, // length
            b'B',
            Amf0Marker::Object as u8,
            0, 1, // length
            b'a',
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
            0, 1, // length
            b'b',
            Amf0Marker::String as u8,
            0, 5, // length
            b'w', b'o', b'r', b'l', b'd',
            0, 0, Amf0Marker::ObjectEnd as u8,
            0, 0, Amf0Marker::ObjectEnd as u8,
        ];
        assert_eq!(bytes, expected);

        let value = Test::C(true, "hello".to_string());
        let bytes = to_bytes(&value).unwrap();
        #[rustfmt::skip]
        let expected = vec![
            Amf0Marker::Object as u8,
            0, 1, // length
            b'C',
            Amf0Marker::StrictArray as u8,
            0, 0, 0, 2, // array length
            Amf0Marker::Boolean as u8,
            1,
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
            0, 0, Amf0Marker::ObjectEnd as u8,
        ];
        assert_eq!(bytes, expected);
    }
}

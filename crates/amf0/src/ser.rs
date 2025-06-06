//! Serialize a Rust data structure into AMF0 data.

use std::io;

use serde::Serialize;
use serde::ser::{
    Impossible, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple, SerializeTupleStruct,
    SerializeTupleVariant,
};

use crate::Amf0Error;
use crate::encoder::Amf0Encoder;

/// Serialize a value into a given writer.
pub fn to_writer<W>(writer: W, value: &impl serde::Serialize) -> crate::Result<()>
where
    W: io::Write,
{
    let mut serializer = Amf0Encoder::new(writer);
    value.serialize(&mut serializer)
}

/// Serialize a value into a new byte vector.
pub fn to_bytes(value: &impl serde::Serialize) -> crate::Result<Vec<u8>> {
    let mut writer = Vec::new();
    to_writer(&mut writer, value)?;
    Ok(writer)
}

impl<W> serde::ser::Serializer for &mut Amf0Encoder<W>
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
        self.encode_boolean(v)
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
        self.encode_number(v)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut buf: [u8; 4] = [0; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.encode_string(v)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let mut seq = self.serialize_seq(Some(v.len()))?;

        for b in v {
            SerializeSeq::serialize_element(&mut seq, b)?;
        }

        SerializeSeq::end(seq)
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
        self.encode_null()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let len = len.ok_or(Amf0Error::UnknownLength)?.try_into()?;
        self.encode_array_header(len)?;
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
        self.encode_object_header()?;
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
        variant.serialize(&mut *self)?;
        value.serialize(&mut *self)?;

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

        variant.serialize(&mut *self)?;
        self.encode_array_header(len)?;

        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        variant.serialize(&mut *self)?;
        self.encode_object_header()?;

        Ok(self)
    }

    fn is_human_readable(&self) -> bool {
        false
    }
}

impl<W> SerializeSeq for &mut Amf0Encoder<W>
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

impl<W> SerializeTuple for &mut Amf0Encoder<W>
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

impl<W> SerializeTupleStruct for &mut Amf0Encoder<W>
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

impl<W> SerializeMap for &mut Amf0Encoder<W>
where
    W: io::Write,
{
    type Error = Amf0Error;
    type Ok = ();

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        key.serialize(&mut MapKeySerializer { ser: self })
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
        self.encode_object_trailer()
    }
}

impl<W> SerializeStruct for &mut Amf0Encoder<W>
where
    W: io::Write,
{
    type Error = Amf0Error;
    type Ok = ();

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde::Serialize,
    {
        key.serialize(&mut MapKeySerializer { ser: *self })?;
        value.serialize(&mut **self)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encode_object_trailer()
    }
}

struct MapKeySerializer<'a, W> {
    ser: &'a mut Amf0Encoder<W>,
}

impl<W> serde::ser::Serializer for &mut MapKeySerializer<'_, W>
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
        self.ser.encode_object_key(v)
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

impl<W> SerializeTupleVariant for &mut Amf0Encoder<W>
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

impl<W> SerializeStructVariant for &mut Amf0Encoder<W>
where
    W: io::Write,
{
    type Error = Amf0Error;
    type Ok = ();

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut MapKeySerializer { ser: *self })?;
        value.serialize(&mut **self)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.encode_object_trailer()
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use std::collections::HashMap;
    use std::hash::Hash;

    use serde_derive::Serialize;

    use crate::{Amf0Error, Amf0Marker, Amf0Value, to_bytes};

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

        let value = "a".repeat(u16::MAX as usize + 1);
        let bytes = to_bytes(&value).unwrap();

        let mut expected = vec![Amf0Marker::LongString as u8];
        expected.extend_from_slice(&(value.len() as u32).to_be_bytes());
        expected.extend(value.as_bytes());
        assert_eq!(bytes, expected);
    }

    #[test]
    fn char() {
        let value = 'a';
        let bytes = to_bytes(&value).unwrap();

        let mut expected = vec![Amf0Marker::String as u8];
        expected.extend((1u16).to_be_bytes());
        expected.extend(b"a");

        assert_eq!(bytes, expected);
    }

    #[test]
    fn bool() {
        let bytes = to_bytes(&true).unwrap();
        assert_eq!(bytes, [Amf0Marker::Boolean as u8, 1]);

        let bytes = to_bytes(&false).unwrap();
        assert_eq!(bytes, [Amf0Marker::Boolean as u8, 0]);
    }

    #[test]
    fn optional() {
        let bytes = to_bytes(&()).unwrap();
        assert_eq!(bytes, [Amf0Marker::Null as u8]);

        let bytes = to_bytes(&None::<String>).unwrap();
        assert_eq!(bytes, [Amf0Marker::Null as u8]);

        #[derive(Serialize)]
        struct Unit;
        let bytes = to_bytes(&Unit).unwrap();
        assert_eq!(bytes, [Amf0Marker::Null as u8]);

        let bytes = to_bytes(&Some("abc")).unwrap();
        assert_eq!(bytes, [Amf0Marker::String as u8, 0, 3, b'a', b'b', b'c']);
    }

    #[test]
    fn tuple_struct() {
        #[derive(Serialize)]
        struct TupleStruct(String, String);

        let value = TupleStruct("hello".to_string(), "world".to_string());
        let bytes = to_bytes(&value).unwrap();

        #[rustfmt::skip]
        assert_eq!(
            bytes,
            [
                Amf0Marker::StrictArray as u8,
                0, 0, 0, 2, // array length
                Amf0Marker::String as u8,
                0, 5, // length
                b'h', b'e', b'l', b'l', b'o',
                Amf0Marker::String as u8,
                0, 5, // length
                b'w', b'o', b'r', b'l', b'd',
            ]
        );
    }

    #[test]
    fn newtype_struct() {
        #[derive(Serialize)]
        struct NewtypeStruct(String);

        let value = NewtypeStruct("hello".to_string());
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
    fn array() {
        let vec = vec![false, true, false];
        let bytes = to_bytes(&vec).unwrap();
        #[rustfmt::skip]
        assert_eq!(
            bytes,
            [
                Amf0Marker::StrictArray as u8,
                0, 0, 0, 3, // array length
                Amf0Marker::Boolean as u8,
                0,
                Amf0Marker::Boolean as u8,
                1,
                Amf0Marker::Boolean as u8,
                0,
            ]
        );

        let byte_vec = vec![0u8, 1]; // 2 bytes
        let bytes = to_bytes(&byte_vec).unwrap();

        #[rustfmt::skip]
        let mut expected = vec![
            Amf0Marker::StrictArray as u8,
            0, 0, 0, 2, // array length
            Amf0Marker::Number as u8,
        ];
        expected.extend(&0.0f64.to_be_bytes());
        expected.push(Amf0Marker::Number as u8);
        expected.extend(&1.0f64.to_be_bytes());
        assert_eq!(bytes, expected);

        let bytes = to_bytes(&("a", false, true)).unwrap();
        #[rustfmt::skip]
        assert_eq!(
            bytes,
            [
                Amf0Marker::StrictArray as u8,
                0, 0, 0, 3, // array length
                Amf0Marker::String as u8,
                0, 1, // length
                b'a',
                Amf0Marker::Boolean as u8,
                0,
                Amf0Marker::Boolean as u8,
                1,
            ]
        );
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
        #[derive(Serialize)]
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
            0, 1, // length
            b'a',
            Amf0Marker::Number as u8,
        ];
        expected.extend(&1.0f64.to_be_bytes());
        #[rustfmt::skip]
        expected.extend_from_slice(&[
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
        #[derive(Serialize)]
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
        #[derive(Serialize)]
        enum Test {
            A(bool),
            B { a: String, b: String },
            C(bool, String),
        }

        let value = Test::A(true);
        let bytes = to_bytes(&value).unwrap();
        #[rustfmt::skip]
        let expected = vec![
            Amf0Marker::String as u8,
            0, 1, // length
            b'A',
            Amf0Marker::Boolean as u8,
            1,
        ];
        assert_eq!(bytes, expected);

        let value = Test::B {
            a: "hello".to_string(),
            b: "world".to_string(),
        };
        let bytes = to_bytes(&value).unwrap();
        #[rustfmt::skip]
        let expected = vec![
            Amf0Marker::String as u8,
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
        ];
        assert_eq!(bytes, expected);

        let value = Test::C(true, "hello".to_string());
        let bytes = to_bytes(&value).unwrap();
        #[rustfmt::skip]
        let expected = vec![
            Amf0Marker::String as u8,
            0, 1, // length
            b'C',
            Amf0Marker::StrictArray as u8,
            0, 0, 0, 2, // array length
            Amf0Marker::Boolean as u8,
            1,
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
        ];
        assert_eq!(bytes, expected);
    }

    fn test_invalid_map_key<T>(key: T)
    where
        T: Eq + Hash + serde::Serialize,
    {
        let mut map = HashMap::new();
        map.insert(key, Amf0Value::Number(1.0));
        let err = to_bytes(&map).unwrap_err();
        assert!(matches!(err, Amf0Error::MapKeyNotString));
    }

    #[test]
    fn invalid_map_keys() {
        test_invalid_map_key(false);

        test_invalid_map_key(1u8);
        test_invalid_map_key(1u16);
        test_invalid_map_key(1u32);
        test_invalid_map_key(1u64);

        test_invalid_map_key(1i8);
        test_invalid_map_key(1i16);
        test_invalid_map_key(1i32);
        test_invalid_map_key(1i64);

        test_invalid_map_key('a');

        test_invalid_map_key([1u8, 2, 3]);

        test_invalid_map_key(None::<String>);
        test_invalid_map_key(Some("hello"));
        test_invalid_map_key(());

        test_invalid_map_key(vec![1, 2, 3]);
        test_invalid_map_key((1, 2, 3));

        #[derive(Serialize, Eq, PartialEq, Hash)]
        struct Tuple(String, String);
        test_invalid_map_key(Tuple("hello".to_string(), "world".to_string()));

        #[derive(Serialize, Eq, PartialEq, Hash)]
        struct Struct {
            a: String,
        }
        test_invalid_map_key(Struct { a: "hello".to_string() });

        #[derive(Serialize, Eq, PartialEq, Hash)]
        struct Unit;
        test_invalid_map_key(Unit);

        #[derive(Serialize, Eq, PartialEq, Hash)]
        struct Newtype(String);
        test_invalid_map_key(Newtype("hello".to_string()));

        #[derive(Serialize, Eq, PartialEq, Hash)]
        enum Enum {
            A,
            B(bool),
            C(String, String),
            D { a: String },
        }
        test_invalid_map_key(Enum::A);
        test_invalid_map_key(Enum::B(true));
        test_invalid_map_key(Enum::C("hello".to_string(), "world".to_string()));
        test_invalid_map_key(Enum::D { a: "hello".to_string() });
    }
}

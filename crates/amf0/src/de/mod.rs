//! Deserialize AMF0 data to a Rust data structure.

use std::io;

use scuffle_bytes_util::zero_copy::ZeroCopyReader;
use serde::de::{EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess};

use crate::decoder::{Amf0Decoder, ObjectHeader};
use crate::{Amf0Error, Amf0Marker};

mod stream;

pub use stream::*;

/// Deserialize a value from a given [`bytes::Buf`].
pub fn from_buf<'de, T>(buf: impl bytes::Buf) -> crate::Result<T>
where
    T: serde::de::Deserialize<'de>,
{
    let mut de = Amf0Decoder::from_buf(buf);
    let value = T::deserialize(&mut de)?;
    Ok(value)
}

/// Deserialize a value from a given [`io::Read`].
pub fn from_reader<'de, T>(reader: impl io::Read) -> crate::Result<T>
where
    T: serde::de::Deserialize<'de>,
{
    let mut de = Amf0Decoder::from_reader(reader);
    let value = T::deserialize(&mut de)?;
    Ok(value)
}

/// Deserialize a value from a given byte slice.
pub fn from_slice<'de, T>(bytes: &'de [u8]) -> crate::Result<T>
where
    T: serde::de::Deserialize<'de>,
{
    let mut de = Amf0Decoder::from_slice(bytes);
    let value = T::deserialize(&mut de)?;
    Ok(value)
}

impl<'de, R> serde::de::Deserializer<'de> for &mut Amf0Decoder<R>
where
    R: ZeroCopyReader<'de>,
{
    type Error = Amf0Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let marker = self.peek_marker()?;

        match marker {
            Amf0Marker::Boolean => self.deserialize_bool(visitor),
            Amf0Marker::Number | Amf0Marker::Date => self.deserialize_f64(visitor),
            Amf0Marker::String | Amf0Marker::LongString | Amf0Marker::XmlDocument => self.deserialize_str(visitor),
            Amf0Marker::Null | Amf0Marker::Undefined => self.deserialize_unit(visitor),
            Amf0Marker::Object | Amf0Marker::TypedObject | Amf0Marker::EcmaArray => self.deserialize_map(visitor),
            Amf0Marker::StrictArray => self.deserialize_seq(visitor),
            _ => Err(Amf0Error::UnsupportedMarker(marker)),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value = self.decode_boolean()?;
        visitor.visit_bool(value)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_i64(visitor)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value = self.decode_number()?;
        visitor.visit_i64(value as i64)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_u64(visitor)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value = self.decode_number()?;
        visitor.visit_u64(value as u64)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_f64(visitor)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value = self.decode_number()?;
        visitor.visit_f64(value)
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(Amf0Error::CharNotSupported)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value = self.decode_string()?;
        value.into_deserializer().deserialize_string(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let value = self.decode_string()?;
        value.into_deserializer().deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let marker = self.peek_marker()?;

        if marker == Amf0Marker::Null || marker == Amf0Marker::Undefined {
            self.next_marker = None; // clear the marker buffer

            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.decode_null()?;
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        if name == stream::MULTI_VALUE_NEW_TYPE {
            visitor.visit_seq(MultiValueDe { de: self })
        } else {
            visitor.visit_newtype_struct(self)
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let size = self.decode_strict_array_header()? as usize;

        visitor.visit_seq(StrictArray {
            de: self,
            remaining: size,
        })
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let size = self.decode_strict_array_header()? as usize;

        if len != size {
            return Err(Amf0Error::WrongArrayLength {
                expected: len,
                got: size,
            });
        }

        visitor.visit_seq(StrictArray {
            de: self,
            remaining: size,
        })
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let header = self.decode_object_header()?;

        match header {
            ObjectHeader::Object | ObjectHeader::TypedObject { .. } => visitor.visit_map(Object { de: self }),
            ObjectHeader::EcmaArray { size } => visitor.visit_map(EcmaArray {
                de: self,
                remaining: size as usize,
            }),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_enum(Enum { de: self })
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let s = self.decode_string()?;
        s.into_deserializer().deserialize_identifier(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct StrictArray<'a, R> {
    de: &'a mut Amf0Decoder<R>,
    remaining: usize,
}

impl<'de, R> SeqAccess<'de> for StrictArray<'_, R>
where
    R: ZeroCopyReader<'de>,
{
    type Error = Amf0Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            return Ok(None);
        }

        self.remaining -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

struct Object<'a, R> {
    de: &'a mut Amf0Decoder<R>,
}

impl<'de, R> MapAccess<'de> for Object<'_, R>
where
    R: ZeroCopyReader<'de>,
{
    type Error = Amf0Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        let Some(key) = self.de.decode_object_key()? else {
            // Reached ObjectEnd marker
            return Ok(None);
        };

        let string_de = key.into_deserializer();
        seed.deserialize(string_de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct EcmaArray<'a, R> {
    de: &'a mut Amf0Decoder<R>,
    remaining: usize,
}

impl<'de, R> MapAccess<'de> for EcmaArray<'_, R>
where
    R: ZeroCopyReader<'de>,
{
    type Error = Amf0Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            // There might be an object end marker after the last key
            if self.de.has_remaining()? && self.de.peek_marker()? == Amf0Marker::ObjectEnd {
                self.de.next_marker = None; // clear the marker buffer
            }

            return Ok(None);
        }

        self.remaining -= 1;

        // Object keys are not preceeded with a marker and are always normal strings
        let s = self.de.decode_normal_string()?;
        let string_de = s.into_deserializer();
        seed.deserialize(string_de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

struct Enum<'a, R> {
    de: &'a mut Amf0Decoder<R>,
}

impl<'de, R> EnumAccess<'de> for Enum<'_, R>
where
    R: ZeroCopyReader<'de>,
{
    type Error = Amf0Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let variant = self.de.decode_string()?;
        let string_de = IntoDeserializer::<Self::Error>::into_deserializer(variant);
        let value = seed.deserialize(string_de)?;

        Ok((value, self))
    }
}

impl<'de, R> VariantAccess<'de> for Enum<'_, R>
where
    R: ZeroCopyReader<'de>,
{
    type Error = Amf0Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_map(self.de, visitor)
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use core::f64;
    use std::collections::HashMap;
    use std::fmt::Debug;

    use bytes::Bytes;
    use scuffle_bytes_util::StringCow;
    use serde_derive::Deserialize;

    use crate::de::MultiValue;
    use crate::decoder::Amf0Decoder;
    use crate::{Amf0Error, Amf0Marker, Amf0Object, Amf0Value, from_buf};

    #[test]
    fn string() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
        ];

        let value: String = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, "hello");

        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::LongString as u8,
            0, 0, 0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
        ];

        let value: String = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, "hello");

        let bytes = [Amf0Marker::Boolean as u8];
        let err = from_buf::<String>(Bytes::from_owner(bytes)).unwrap_err();
        assert!(matches!(
            err,
            Amf0Error::UnexpectedType {
                expected: [Amf0Marker::String, Amf0Marker::LongString, Amf0Marker::XmlDocument],
                got: Amf0Marker::Boolean
            }
        ));
    }

    #[test]
    fn bool() {
        let bytes = [Amf0Marker::Boolean as u8, 1];
        let value: bool = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert!(value);

        let bytes = [Amf0Marker::String as u8];
        let err = from_buf::<bool>(Bytes::from_owner(bytes)).unwrap_err();
        assert!(matches!(
            err,
            Amf0Error::UnexpectedType {
                expected: [Amf0Marker::Boolean],
                got: Amf0Marker::String
            }
        ));
    }

    fn number_test<'de, T>(one: T)
    where
        T: serde::Deserialize<'de> + PartialEq + Debug,
    {
        const NUMBER_ONE: [u8; 9] = const {
            let one = 1.0f64.to_be_bytes();
            [
                Amf0Marker::Number as u8,
                one[0],
                one[1],
                one[2],
                one[3],
                one[4],
                one[5],
                one[6],
                one[7],
            ]
        };

        let value: T = from_buf(Bytes::from_static(&NUMBER_ONE)).unwrap();
        assert_eq!(value, one);
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
        number_test(1f32);
        number_test(1f64);

        let mut bytes = vec![Amf0Marker::Date as u8];
        bytes.extend_from_slice(&f64::consts::PI.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes()); // timezone
        let value: f64 = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, f64::consts::PI);

        let bytes = [Amf0Marker::Boolean as u8];
        let err = from_buf::<f64>(Bytes::from_owner(bytes)).unwrap_err();
        assert!(matches!(
            err,
            Amf0Error::UnexpectedType {
                expected: [Amf0Marker::Number, Amf0Marker::Date],
                got: Amf0Marker::Boolean
            }
        ));
    }

    #[test]
    fn char() {
        let err = from_buf::<char>(Bytes::from_owner([])).unwrap_err();
        assert!(matches!(err, Amf0Error::CharNotSupported));
    }

    #[test]
    fn optional() {
        let bytes = [Amf0Marker::Null as u8];
        let value: Option<bool> = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, None);

        let bytes = [Amf0Marker::Null as u8];
        from_buf::<()>(Bytes::from_owner(bytes)).unwrap();

        let bytes = [Amf0Marker::String as u8];
        let err = from_buf::<()>(Bytes::from_owner(bytes)).unwrap_err();
        assert!(matches!(
            err,
            Amf0Error::UnexpectedType {
                expected: [Amf0Marker::Null, Amf0Marker::Undefined],
                got: Amf0Marker::String
            }
        ));

        let bytes = [Amf0Marker::Undefined as u8];
        let value: Option<bool> = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, None);

        let bytes = [Amf0Marker::Boolean as u8, 0];
        let value: Option<bool> = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Some(false));

        #[derive(Deserialize, PartialEq, Debug)]
        struct Unit;

        let bytes = [Amf0Marker::Null as u8];
        let value: Unit = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Unit);
    }

    #[test]
    fn newtype_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Test(String);

        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
        ];
        let value: Test = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Test("hello".to_string()));
    }

    #[test]
    fn tuple_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Test(bool, String);

        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::StrictArray as u8,
            0, 0, 0, 2, // length
            Amf0Marker::Boolean as u8,
            1,
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
        ];
        let value: Test = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Test(true, "hello".to_string()));

        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::StrictArray as u8,
            0, 0, 0, 1, // length
            Amf0Marker::Boolean as u8,
            1,
        ];
        let err = from_buf::<Test>(Bytes::from_owner(bytes)).unwrap_err();
        assert!(matches!(err, Amf0Error::WrongArrayLength { expected: 2, got: 1 }));
    }

    #[test]
    fn typed_object() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Test {
            a: bool,
            b: String,
        }

        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::TypedObject as u8,
            0, 1, // name length
            b'a', // name
            0, 1, // length
            b'a', // key
            Amf0Marker::Boolean as u8,
            1,
            0, 1, // length
            b'b', // key
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
            0, 0, Amf0Marker::ObjectEnd as u8,
        ];
        let value: Test = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(
            value,
            Test {
                a: true,
                b: "hello".to_string()
            }
        );
    }

    #[test]
    fn simple_struct() {
        #[derive(Deserialize, Debug, PartialEq)]
        struct Test {
            a: bool,
            b: String,
            c: f64,
        }

        #[rustfmt::skip]
        let mut bytes = vec![
            Amf0Marker::Object as u8,
            0, 1, // length
            b'a', // key
            Amf0Marker::Boolean as u8, // value
            1,
            0, 1, // length
            b'b', // key
            Amf0Marker::String as u8, // value
            0, 1, // length
            b'b', // value
            0, 1, // length
            b'c', // key
            Amf0Marker::Number as u8, // value
        ];
        bytes.extend_from_slice(&f64::consts::PI.to_be_bytes());
        bytes.extend_from_slice(&[0, 0, Amf0Marker::ObjectEnd as u8]);
        let value: Test = from_buf(Bytes::from_owner(bytes)).unwrap();

        assert_eq!(
            value,
            Test {
                a: true,
                b: "b".to_string(),
                c: f64::consts::PI,
            }
        );

        #[rustfmt::skip]
        let mut bytes = vec![
            Amf0Marker::EcmaArray as u8,
            0, 0, 0, 3, // length
            0, 1, // length
            b'a', // key
            Amf0Marker::Boolean as u8, // value
            1,
            0, 1, // length
            b'b', // key
            Amf0Marker::String as u8, // value
            0, 1, // length
            b'b', // value
            0, 1, // length
            b'c', // key
            Amf0Marker::Number as u8, // value
        ];
        bytes.extend_from_slice(&f64::consts::PI.to_be_bytes());
        bytes.extend_from_slice(&[0, 0, 0]); // not object end marker
        let value: Test = from_buf(Bytes::from_owner(bytes)).unwrap();

        assert_eq!(
            value,
            Test {
                a: true,
                b: "b".to_string(),
                c: f64::consts::PI,
            }
        );

        let err = from_buf::<Test>(Bytes::from_owner([Amf0Marker::String as u8])).unwrap_err();
        assert!(matches!(
            err,
            Amf0Error::UnexpectedType {
                expected: [Amf0Marker::Object, Amf0Marker::TypedObject, Amf0Marker::EcmaArray],
                got: Amf0Marker::String
            }
        ));
    }

    #[test]
    fn simple_enum() {
        #[derive(Deserialize, Debug, PartialEq)]
        enum Test {
            A,
            B,
        }

        #[rustfmt::skip]
        let bytes = vec![
            Amf0Marker::String as u8,
            0, 1, // length
            b'A',
        ];
        let value: Test = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Test::A);

        #[rustfmt::skip]
        let bytes = vec![
            Amf0Marker::String as u8,
            0, 1, // length
            b'B',
        ];
        let value: Test = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Test::B);
    }

    #[test]
    fn complex_enum() {
        #[derive(Deserialize, Debug, PartialEq)]
        enum Test {
            A(bool),
            B { a: String, b: String },
            C(bool, String),
        }

        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::String as u8,
            0, 1, // length
            b'A',
            Amf0Marker::Boolean as u8,
            1,
        ];
        let value: Test = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Test::A(true));

        #[rustfmt::skip]
        let bytes = [
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
        let value: Test = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(
            value,
            Test::B {
                a: "hello".to_string(),
                b: "world".to_string()
            }
        );

        #[rustfmt::skip]
        let bytes = [
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
        let value: Test = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(value, Test::C(true, "hello".to_string()));
    }

    #[test]
    fn series() {
        #[rustfmt::skip]
        let mut bytes = vec![
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
            Amf0Marker::Boolean as u8,
            1,
            Amf0Marker::Number as u8,
        ];
        bytes.extend_from_slice(&f64::consts::PI.to_be_bytes());

        let mut de = Amf0Decoder::from_buf(Bytes::from_owner(bytes));
        let value: String = serde::de::Deserialize::deserialize(&mut de).unwrap();
        assert_eq!(value, "hello");
        let value: bool = serde::de::Deserialize::deserialize(&mut de).unwrap();
        assert!(value);
        let value: f64 = serde::de::Deserialize::deserialize(&mut de).unwrap();
        assert_eq!(value, f64::consts::PI);
    }

    #[test]
    fn flatten() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::Object as u8,
            0, 1, // length
            b'a',
            Amf0Marker::Boolean as u8,
            1,
            0, 1, // length
            b'b',
            Amf0Marker::String as u8,
            0, 1, // length
            b'b',
            0, 1, // length
            b'c',
            Amf0Marker::String as u8,
            0, 1, // length
            b'c',
            0, 0, Amf0Marker::ObjectEnd as u8,
        ];

        #[derive(Deserialize, Debug, PartialEq)]
        struct Test<'a> {
            b: String,
            #[serde(flatten, borrow)]
            other: HashMap<StringCow<'a>, Amf0Value<'a>>,
        }

        let value: Test = from_buf(Bytes::from_owner(bytes)).unwrap();
        assert_eq!(
            value,
            Test {
                b: "b".to_string(),
                other: vec![
                    ("a".into(), Amf0Value::from(true)),
                    ("c".into(), StringCow::from_static("c").into())
                ]
                .into_iter()
                .collect(),
            }
        );
    }

    #[test]
    fn all() {
        let bytes = [
            Amf0Marker::String as u8,
            0,
            5, // length
            b'h',
            b'e',
            b'l',
            b'l',
            b'o',
            Amf0Marker::Boolean as u8,
            1,
            Amf0Marker::Object as u8,
            0,
            1, // length
            b'a',
            Amf0Marker::Boolean as u8,
            1,
            0,
            0,
            Amf0Marker::ObjectEnd as u8,
        ];

        let mut de = Amf0Decoder::from_buf(Bytes::from_owner(bytes));
        let values = de.decode_all().unwrap();
        assert_eq!(
            values,
            vec![
                Amf0Value::String("hello".into()),
                Amf0Value::Boolean(true),
                Amf0Value::Object([("a".into(), Amf0Value::Boolean(true))].into_iter().collect())
            ]
        );
    }

    #[test]
    fn multi_value() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
            Amf0Marker::Boolean as u8,
            1,
            Amf0Marker::Object as u8,
            0, 1, // length
            b'a',
            Amf0Marker::Boolean as u8,
            1,
            0, 0, Amf0Marker::ObjectEnd as u8,
        ];

        let mut de = Amf0Decoder::from_buf(Bytes::from_owner(bytes));
        let values: MultiValue<(String, bool, Amf0Object)> = de.deserialize().unwrap();
        assert_eq!(values.0.0, "hello");
        assert!(values.0.1);
        assert_eq!(
            values.0.2,
            [("a".into(), Amf0Value::Boolean(true))].into_iter().collect::<Amf0Object>()
        );
    }

    #[test]
    fn deserializer_stream() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::String as u8,
            0, 5, // length
            b'h', b'e', b'l', b'l', b'o',
            Amf0Marker::String as u8,
            0, 5, // length
            b'w', b'o', b'r', b'l', b'd',
            Amf0Marker::String as u8,
            0, 1, // length
            b'a',
        ];

        let mut de = Amf0Decoder::from_buf(Bytes::from_owner(bytes));
        let mut stream = de.deserialize_stream::<String>();
        assert_eq!(stream.next().unwrap().unwrap(), "hello");
        assert_eq!(stream.next().unwrap().unwrap(), "world");
        assert_eq!(stream.next().unwrap().unwrap(), "a");
        assert_eq!(stream.next().transpose().unwrap(), None);
    }
}

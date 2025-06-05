//! AMF0 value types.

use std::io;

use scuffle_bytes_util::StringCow;
use serde::de::IntoDeserializer;

use crate::Amf0Error;
use crate::encoder::Amf0Encoder;
use crate::object::Amf0Object;

/// Represents any AMF0 value.
#[derive(Debug, PartialEq, Clone)]
pub enum Amf0Value<'a> {
    /// AMF0 Number.
    Number(f64),
    /// AMF0 Boolean.
    Boolean(bool),
    /// AMF0 String.
    String(StringCow<'a>),
    /// AMF0 Object.
    Object(Amf0Object<'a>),
    /// AMF0 Null.
    Null,
    /// AMF0 Array.
    Array(Vec<Amf0Value<'a>>),
}

impl Amf0Value<'_> {
    /// Converts this AMF0 value into an owned version (static lifetime).
    pub fn into_owned(self) -> Amf0Value<'static> {
        match self {
            Amf0Value::Number(v) => Amf0Value::Number(v),
            Amf0Value::Boolean(v) => Amf0Value::Boolean(v),
            Amf0Value::String(v) => Amf0Value::String(v.into_owned()),
            Amf0Value::Object(v) => {
                Amf0Value::Object(v.into_iter().map(|(k, v)| (k.into_owned(), v.into_owned())).collect())
            }
            Amf0Value::Null => Amf0Value::Null,
            Amf0Value::Array(v) => Amf0Value::Array(v.into_iter().map(|v| v.into_owned()).collect()),
        }
    }

    /// Encode this AMF0 value with the given encoder.
    pub fn encode<W: io::Write>(&self, encoder: &mut Amf0Encoder<W>) -> Result<(), Amf0Error> {
        match self {
            Amf0Value::Number(v) => encoder.encode_number(*v),
            Amf0Value::Boolean(v) => encoder.encode_boolean(*v),
            Amf0Value::String(v) => encoder.encode_string(v.as_str()),
            Amf0Value::Object(v) => encoder.encode_object(v),
            Amf0Value::Null => encoder.encode_null(),
            Amf0Value::Array(v) => encoder.encode_array(v),
        }
    }
}

impl From<f64> for Amf0Value<'_> {
    fn from(value: f64) -> Self {
        Amf0Value::Number(value)
    }
}

impl From<bool> for Amf0Value<'_> {
    fn from(value: bool) -> Self {
        Amf0Value::Boolean(value)
    }
}

impl<'a> From<StringCow<'a>> for Amf0Value<'a> {
    fn from(value: StringCow<'a>) -> Self {
        Amf0Value::String(value)
    }
}

// object
impl<'a> From<Amf0Object<'a>> for Amf0Value<'a> {
    fn from(value: Amf0Object<'a>) -> Self {
        Amf0Value::Object(value)
    }
}

// owned array
impl<'a> From<Vec<Amf0Value<'a>>> for Amf0Value<'a> {
    fn from(value: Vec<Amf0Value<'a>>) -> Self {
        Amf0Value::Array(value)
    }
}

impl<'a> FromIterator<Amf0Value<'a>> for Amf0Value<'a> {
    fn from_iter<T: IntoIterator<Item = Amf0Value<'a>>>(iter: T) -> Self {
        Amf0Value::Array(iter.into_iter().collect())
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::Deserialize<'de> for Amf0Value<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Amf0ValueVisitor;

        impl<'de> serde::de::Visitor<'de> for Amf0ValueVisitor {
            type Value = Amf0Value<'de>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an AMF0 value")
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::Boolean(v))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_f64(v as f64)
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_f64(v as f64)
            }

            #[inline]
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::Number(v))
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(v.to_owned())
            }

            #[inline]
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StringCow::from(v).into())
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(StringCow::from(v).into())
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::Null)
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                serde::Deserialize::deserialize(deserializer)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(value) = seq.next_element()? {
                    vec.push(value);
                }

                Ok(vec.into())
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                use crate::object::Amf0Object;

                let mut object = Amf0Object::new();

                while let Some((key, value)) = map.next_entry()? {
                    object.insert(key, value);
                }

                Ok(object.into())
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let array = v.iter().map(|b| Amf0Value::Number(*b as f64)).collect();
                Ok(Amf0Value::Array(array))
            }
        }

        deserializer.deserialize_any(Amf0ValueVisitor)
    }
}

#[cfg(feature = "serde")]
impl serde::ser::Serialize for Amf0Value<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Amf0Value::Number(v) => serializer.serialize_f64(*v),
            Amf0Value::Boolean(v) => serializer.serialize_bool(*v),
            Amf0Value::String(v) => v.serialize(serializer),
            Amf0Value::Object(v) => {
                let mut map = serializer.serialize_map(Some(v.len()))?;

                for (key, value) in v.iter() {
                    serde::ser::SerializeMap::serialize_entry(&mut map, key, value)?;
                }

                serde::ser::SerializeMap::end(map)
            }
            Amf0Value::Null => serializer.serialize_none(),
            Amf0Value::Array(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;

                for value in v.iter() {
                    serde::ser::SerializeSeq::serialize_element(&mut seq, value)?;
                }

                serde::ser::SerializeSeq::end(seq)
            }
        }
    }
}

#[cfg(feature = "serde")]
macro_rules! impl_de_number {
    ($deserializser_fn:ident, $visit_fn:ident) => {
        fn $deserializser_fn<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: serde::de::Visitor<'de>,
        {
            if let Amf0Value::Number(n) = self {
                let n = *(&n as &f64);
                if let Some(n) = ::num_traits::cast(n) {
                    visitor.$visit_fn(n)
                } else {
                    visitor.visit_f64(n)
                }
            } else {
                self.deserialize_any(visitor)
            }
        }
    };
}

#[cfg(feature = "serde")]
macro_rules! impl_deserializer {
    ($ty:ty) => {
        impl<'de> serde::Deserializer<'de> for $ty {
            type Error = Amf0Error;

            serde::forward_to_deserialize_any! {
                bool f64 str string unit
                seq map newtype_struct tuple
                struct enum ignored_any identifier
            }

            impl_de_number!(deserialize_i8, visit_i8);

            impl_de_number!(deserialize_i16, visit_i16);

            impl_de_number!(deserialize_i32, visit_i32);

            impl_de_number!(deserialize_i64, visit_i64);

            impl_de_number!(deserialize_u8, visit_u8);

            impl_de_number!(deserialize_u16, visit_u16);

            impl_de_number!(deserialize_u32, visit_u32);

            impl_de_number!(deserialize_u64, visit_u64);

            impl_de_number!(deserialize_f32, visit_f32);

            fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                if let Amf0Value::String(s) = self {
                    s.into_deserializer().deserialize_any(visitor)
                } else {
                    self.deserialize_any(visitor)
                }
            }

            fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                match self {
                    Amf0Value::Null => visitor.visit_none(),
                    _ => visitor.visit_some(self),
                }
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

            fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                self.deserialize_unit(visitor)
            }

            fn deserialize_tuple_struct<V>(
                self,
                _name: &'static str,
                len: usize,
                visitor: V,
            ) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                self.deserialize_tuple(len, visitor)
            }

            fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                match self {
                    Amf0Value::Null => visitor.visit_unit(),
                    Amf0Value::Boolean(b) => visitor.visit_bool(*(&b as &bool)),
                    Amf0Value::Number(n) => visitor.visit_f64(*(&n as &f64)),
                    Amf0Value::String(s) => s.into_deserializer().deserialize_any(visitor),
                    Amf0Value::Array(a) => visitor.visit_seq(Amf0SeqAccess { iter: a.into_iter() }),
                    Amf0Value::Object(o) => o.into_deserializer().deserialize_any(visitor),
                }
            }
        }
    };
}

#[cfg(feature = "serde")]
impl_deserializer!(Amf0Value<'de>);
#[cfg(feature = "serde")]
impl_deserializer!(&'de Amf0Value<'de>);

#[cfg(feature = "serde")]
impl<'de> serde::de::IntoDeserializer<'de, Amf0Error> for Amf0Value<'de> {
    type Deserializer = Amf0Value<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::de::IntoDeserializer<'de, Amf0Error> for &'de Amf0Value<'de> {
    type Deserializer = &'de Amf0Value<'de>;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

#[cfg(feature = "serde")]
struct Amf0SeqAccess<I> {
    iter: I,
}

#[cfg(feature = "serde")]
impl<'de, I> serde::de::SeqAccess<'de> for Amf0SeqAccess<I>
where
    I: Iterator,
    I::Item: serde::de::IntoDeserializer<'de, Amf0Error>,
{
    type Error = Amf0Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value.into_deserializer()).map(Some),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use scuffle_bytes_util::StringCow;
    #[cfg(feature = "serde")]
    use serde::Deserialize;

    use super::Amf0Value;
    use crate::{Amf0Decoder, Amf0Encoder, Amf0Error, Amf0Marker, Amf0Object};

    #[test]
    fn from() {
        let value: Amf0Value = 1.0.into();
        assert_eq!(value, Amf0Value::Number(1.0));

        let value: Amf0Value = true.into();
        assert_eq!(value, Amf0Value::Boolean(true));

        let value: Amf0Value = StringCow::from("abc").into();
        assert_eq!(value, Amf0Value::String("abc".into()));

        let object: Amf0Object = [("a".into(), Amf0Value::Boolean(true))].into_iter().collect();
        let value: Amf0Value = object.clone().into();
        assert_eq!(value, Amf0Value::Object(object));

        let array: Vec<Amf0Value> = vec![Amf0Value::Boolean(true)];
        let value: Amf0Value = array.clone().into();
        assert_eq!(value, Amf0Value::Array(array));

        let iter = std::iter::once(Amf0Value::Boolean(true));
        let value: Amf0Value = iter.collect();
        assert_eq!(value, Amf0Value::Array(vec![Amf0Value::Boolean(true)]));
    }

    #[test]
    fn unsupported_marker() {
        let bytes = [Amf0Marker::MovieClipMarker as u8];

        let err = Amf0Decoder::from_slice(&bytes).decode_value().unwrap_err();
        assert!(matches!(err, Amf0Error::UnsupportedMarker(Amf0Marker::MovieClipMarker)));
    }

    #[test]
    fn string() {
        use crate::Amf0Decoder;

        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::String as u8,
            0, 3, // length
            b'a', b'b', b'c',
        ];

        let value = Amf0Decoder::from_slice(&bytes).decode_value().unwrap();
        assert_eq!(value, Amf0Value::String("abc".into()));
    }

    #[test]
    fn bool() {
        let bytes = [Amf0Marker::Boolean as u8, 0];

        let value = Amf0Decoder::from_slice(&bytes).decode_value().unwrap();
        assert_eq!(value, Amf0Value::Boolean(false));
    }

    #[test]
    fn object() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::Object as u8,
            0, 1,
            b'a',
            Amf0Marker::Boolean as u8,
            1,
            0, 0, Amf0Marker::ObjectEnd as u8,
        ];

        let value = Amf0Decoder::from_slice(&bytes).decode_value().unwrap();
        assert_eq!(
            value,
            Amf0Value::Object([("a".into(), Amf0Value::Boolean(true))].into_iter().collect())
        );
    }

    #[test]
    fn array() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::StrictArray as u8,
            0, 0, 0, 1,
            Amf0Marker::Boolean as u8,
            1,
        ];

        let value = Amf0Decoder::from_slice(&bytes).decode_value().unwrap();
        assert_eq!(value, Amf0Value::Array(vec![Amf0Value::Boolean(true)]));

        let mut serialized = vec![];
        value.encode(&mut Amf0Encoder::new(&mut serialized)).unwrap();
        assert_eq!(serialized, bytes);
    }

    #[test]
    fn null() {
        let bytes = [Amf0Marker::Null as u8];

        let value = Amf0Decoder::from_slice(&bytes).decode_value().unwrap();
        assert_eq!(value, Amf0Value::Null);

        let mut serialized = vec![];
        value.encode(&mut Amf0Encoder::new(&mut serialized)).unwrap();
        assert_eq!(serialized, bytes);
    }

    #[test]
    fn into_owned() {
        let value = Amf0Value::Number(1.0);
        let owned_value = value.clone().into_owned();
        assert_eq!(owned_value, value);

        let value = Amf0Value::Boolean(true);
        let owned_value = value.clone().into_owned();
        assert_eq!(owned_value, value);

        let value = Amf0Value::String("abc".into());
        let owned_value = value.clone().into_owned();
        assert_eq!(owned_value, value);

        let value = Amf0Value::Object([("a".into(), Amf0Value::Boolean(true))].into_iter().collect());
        let owned_value = value.clone().into_owned();
        assert_eq!(owned_value, value,);

        let value = Amf0Value::Null;
        let owned_value = value.clone().into_owned();
        assert_eq!(owned_value, value);

        let value = Amf0Value::Array(vec![Amf0Value::Boolean(true)]);
        let owned_value = value.clone().into_owned();
        assert_eq!(owned_value, value);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize() {
        use std::fmt::Display;

        use serde::Deserialize;
        use serde::de::{IntoDeserializer, MapAccess, SeqAccess};

        #[derive(Debug)]
        struct TestError;

        impl Display for TestError {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "Test error")
            }
        }

        impl std::error::Error for TestError {}

        impl serde::de::Error for TestError {
            fn custom<T: std::fmt::Display>(msg: T) -> Self {
                assert_eq!(msg.to_string(), "invalid type: Option value, expected a byte slice");
                Self
            }
        }

        enum Mode {
            Bool,
            I64,
            U64,
            F64,
            Str,
            String,
            BorrowedStr,
            Unit,
            None,
            Some,
            Seq,
            Map,
            Bytes,
            End,
        }

        struct TestDeserializer {
            mode: Mode,
        }

        impl<'de> SeqAccess<'de> for TestDeserializer {
            type Error = TestError;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
            where
                T: serde::de::DeserializeSeed<'de>,
            {
                match self.mode {
                    Mode::Seq => {
                        self.mode = Mode::End;
                        Ok(Some(seed.deserialize(TestDeserializer { mode: Mode::I64 })?))
                    }
                    Mode::End => Ok(None),
                    _ => Err(TestError),
                }
            }
        }

        impl<'de> MapAccess<'de> for TestDeserializer {
            type Error = TestError;

            fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
            where
                K: serde::de::DeserializeSeed<'de>,
            {
                match self.mode {
                    Mode::Map => Ok(Some(seed.deserialize(TestDeserializer { mode: Mode::Str })?)),
                    Mode::End => Ok(None),
                    _ => Err(TestError),
                }
            }

            fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::DeserializeSeed<'de>,
            {
                match self.mode {
                    Mode::Map => {
                        self.mode = Mode::End;
                        Ok(seed.deserialize(TestDeserializer { mode: Mode::I64 })?)
                    }
                    _ => Err(TestError),
                }
            }
        }

        impl<'de> serde::Deserializer<'de> for TestDeserializer {
            type Error = TestError;

            serde::forward_to_deserialize_any! {
                bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes byte_buf
                option unit unit_struct newtype_struct seq tuple tuple_struct
                map struct enum identifier ignored_any
            }

            fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                match self.mode {
                    Mode::Bool => visitor.visit_bool(true),
                    Mode::I64 => visitor.visit_i64(1),
                    Mode::U64 => visitor.visit_u64(1),
                    Mode::F64 => visitor.visit_f64(1.0),
                    Mode::Str => visitor.visit_str("hello"),
                    Mode::String => visitor.visit_string("hello".to_owned()),
                    Mode::BorrowedStr => visitor.visit_borrowed_str("hello"),
                    Mode::Unit => visitor.visit_unit(),
                    Mode::None => visitor.visit_none(),
                    Mode::Some => visitor.visit_some(1.into_deserializer()),
                    Mode::Seq => visitor.visit_seq(self),
                    Mode::Map => visitor.visit_map(self),
                    Mode::Bytes => visitor.visit_bytes(b"hello"),
                    Mode::End => unreachable!(),
                }
            }
        }

        fn test_de(mode: Mode, expected: Amf0Value) {
            let deserializer = TestDeserializer { mode };
            let deserialized_value: Amf0Value = Amf0Value::deserialize(deserializer).unwrap();
            assert_eq!(deserialized_value, expected);
        }

        test_de(Mode::Bool, Amf0Value::Boolean(true));
        test_de(Mode::I64, Amf0Value::Number(1.0));
        test_de(Mode::U64, Amf0Value::Number(1.0));
        test_de(Mode::F64, Amf0Value::Number(1.0));
        test_de(Mode::Str, Amf0Value::String("hello".into()));
        test_de(Mode::String, Amf0Value::String("hello".into()));
        test_de(Mode::BorrowedStr, Amf0Value::String("hello".into()));
        test_de(Mode::Unit, Amf0Value::Null);
        test_de(Mode::None, Amf0Value::Null);
        test_de(Mode::Some, Amf0Value::Number(1.0));
        test_de(Mode::Seq, Amf0Value::Array(vec![Amf0Value::Number(1.0)]));
        test_de(
            Mode::Map,
            Amf0Value::Object([("hello".into(), Amf0Value::Number(1.0))].into_iter().collect()),
        );
        test_de(
            Mode::Bytes,
            Amf0Value::Array(vec![
                Amf0Value::Number(104.0),
                Amf0Value::Number(101.0),
                Amf0Value::Number(108.0),
                Amf0Value::Number(108.0),
                Amf0Value::Number(111.0),
            ]),
        );
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_bool() {
        let value = Amf0Value::Boolean(true);
        let deserialized: bool = Deserialize::deserialize(value).unwrap();
        assert!(deserialized);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_number() {
        let value = Amf0Value::Number(42.0);
        let deserialized: f64 = Deserialize::deserialize(value).unwrap();
        assert_eq!(deserialized, 42.0);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_string() {
        let value = Amf0Value::String(StringCow::from("hello"));
        let deserialized: String = Deserialize::deserialize(value).unwrap();
        assert_eq!(deserialized, "hello");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_null() {
        let value = Amf0Value::Null;
        let deserialized: Option<i32> = Deserialize::deserialize(value).unwrap();
        assert_eq!(deserialized, None);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_array() {
        let value = Amf0Value::Array(vec![Amf0Value::Number(1.0), Amf0Value::Number(2.0), Amf0Value::Number(3.0)]);

        let deserialized: Vec<f64> = Deserialize::deserialize(value).unwrap();
        assert_eq!(deserialized, vec![1.0, 2.0, 3.0]);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_object() {
        use std::collections::HashMap;

        let mut map = Amf0Object::new();
        map.insert(StringCow::from("key"), Amf0Value::String(StringCow::from("value")));
        let value = Amf0Value::Object(map);

        let deserialized: HashMap<String, String> = Deserialize::deserialize(value).unwrap();
        assert_eq!(deserialized.get("key"), Some(&"value".to_string()));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn deserialize_complex_structure() {
        let value = Amf0Value::Object(Amf0Object::from_iter([
            (
                StringCow::from("numbers"),
                Amf0Value::Array(vec![Amf0Value::Number(1.0), Amf0Value::Number(2.0)]),
            ),
            (StringCow::from("flag"), Amf0Value::Boolean(true)),
        ]));

        #[derive(Debug, Deserialize, PartialEq)]
        struct Complex {
            numbers: Vec<f64>,
            flag: bool,
        }

        let deserialized: Complex = Deserialize::deserialize(value).unwrap();

        assert_eq!(
            deserialized,
            Complex {
                numbers: vec![1.0, 2.0],
                flag: true
            }
        );
    }
}

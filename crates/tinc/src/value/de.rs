use core::fmt;
use std::borrow::Cow;

use ordered_float::OrderedFloat;
use serde::de::{self, IntoDeserializer, SeqAccess};
use serde::{Deserialize, Deserializer};

use super::{Object, ObjectOwned, Value, ValueError};
use crate::value::{Map, ValueOwned};

macro_rules! impl_deserialize_number {
    ($ty:ty, $deserialize_fn:ident, $visit_fn:ident) => {
        #[inline]
        fn $deserialize_fn<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            let v = match self.value {
                Value::String(ref s) => Some(std::str::FromStr::from_str(&s).map_err(serde::de::Error::custom)?),
                Value::U8(v) => num_traits::cast::cast(v),
                Value::U16(v) => num_traits::cast::cast(v),
                Value::U32(v) => num_traits::cast::cast(v),
                Value::U64(v) => num_traits::cast::cast(v),
                Value::U128(v) => num_traits::cast::cast(v),
                Value::I8(v) => num_traits::cast::cast(v),
                Value::I16(v) => num_traits::cast::cast(v),
                Value::I32(v) => num_traits::cast::cast(v),
                Value::I64(v) => num_traits::cast::cast(v),
                Value::I128(v) => num_traits::cast::cast(v),
                Value::F32(v) => num_traits::cast::cast(v),
                Value::F64(v) => num_traits::cast::cast(v),
                Value::Bool(v) => num_traits::cast::cast(v as u8),
                _ => None,
            };

            if let Some(v) = v {
                visitor.$visit_fn(v)
            } else {
                Err(serde::de::Error::invalid_type(
                    self.value.unexpected(),
                    &stringify!($ty),
                ))
            }
        }
    };
}

/// Try to parse a bool from a string representation.
fn parse_bool_primitive(s: &str) -> Option<bool> {
    match s {
        "true" | "1" | "yes" | "on" | "enable" | "enabled" | "t" | "y" => Some(true),
        "false" | "0" | "no" | "off" | "disable" | "disabled" | "f" | "n" => Some(false),
        _ => None,
    }
}

/// Parses a string into a single character, returning an error if the string is empty or too long.
fn parse_char_primitive(s: &str) -> Result<char, ValueError> {
    let mut chars = s.chars();
    let Some(c) = chars.next() else {
        return Err(ValueError::Custom("expected 1 character, got empty string".to_string()));
    };

    if chars.next().is_some() {
        return Err(ValueError::Custom(
            "expected 1 character, got multiple characters".to_string(),
        ));
    }

    Ok(c)
}

/// Deserialize implementation for `Object<'de>`.
impl<'de> de::Deserialize<'de> for Object<'de> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        de::Deserialize::deserialize(deserializer).map(Object)
    }
}

/// Deserialize implementation for `ObjectOwned`.
impl<'de> de::Deserialize<'de> for ObjectOwned {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let object = Object::deserialize(deserializer)?;
        Ok(object.into_owned())
    }
}

/// Deserialize implementation for `Map<K, V>`.
impl<'de, K, V> serde::Deserialize<'de> for Map<K, V>
where
    K: serde::Deserialize<'de>,
    V: serde::Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MapVisitor<K, V> {
            marker: std::marker::PhantomData<Map<K, V>>,
        }

        impl<'de, K, V> de::Visitor<'de> for MapVisitor<K, V>
        where
            K: serde::Deserialize<'de>,
            V: serde::Deserialize<'de>,
        {
            type Value = Map<K, V>;

            #[inline]
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a map")
            }

            fn visit_map<D>(self, mut visitor: D) -> Result<Self::Value, D::Error>
            where
                D: de::MapAccess<'de>,
            {
                let mut map = Map::with_capacity(visitor.size_hint().unwrap_or(0));
                while let Some((key, value)) = visitor.next_entry()? {
                    map.push(key, value);
                }
                Ok(map)
            }
        }

        deserializer.deserialize_map(MapVisitor {
            marker: std::marker::PhantomData,
        })
    }
}

/// Implement `Deserializer` for `Object<'de>`.
impl<'de> de::Deserializer<'de> for Object<'de> {
    type Error = ValueError;

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(MapDeserializer::new(self.0.into_iter().map(|(k, v)| (k.into(), v))))
    }
}

/// Allow `Object<'de>` to be used as a deserializer.
impl<'de> de::IntoDeserializer<'de, ValueError> for Object<'de> {
    type Deserializer = Self;

    #[inline]
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

/// Allow `ObjectOwned` to be used as a deserializer.
impl de::IntoDeserializer<'_, ValueError> for ObjectOwned {
    type Deserializer = ObjectOwned;

    #[inline]
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> de::Deserializer<'de> for ObjectOwned {
    type Error = ValueError;

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.0.deserialize_any(visitor)
    }
}

/// Deserialize implementation for `Value<'de>`.
impl<'de> de::Deserialize<'de> for Value<'de> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ValueKindVisitor;

        impl<'de> de::Visitor<'de> for ValueKindVisitor {
            type Value = Value<'de>;

            #[inline]
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a value")
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Bool(v))
            }

            #[inline]
            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::I8(v))
            }

            #[inline]
            fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Char(v))
            }

            #[inline]
            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::U8(v))
            }

            #[inline]
            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::F32(OrderedFloat(v)))
            }

            #[inline]
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::F64(OrderedFloat(v)))
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::String(Cow::Owned(v.to_owned())))
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::String(Cow::Borrowed(v)))
            }

            #[inline]
            fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::I128(v))
            }

            #[inline]
            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::I16(v))
            }

            #[inline]
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::I32(v))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::I64(v))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Null)
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let mut values = Vec::with_capacity(visitor.size_hint().unwrap_or(0));
                while let Some(value) = visitor.next_element()? {
                    values.push(value);
                }
                Ok(Value::Array(values))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut map = Map::with_capacity(visitor.size_hint().unwrap_or(0));
                while let Some((key, value)) = visitor.next_entry()? {
                    map.push(key, value);
                }
                Ok(Value::Map(map))
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Unit)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                deserializer.deserialize_any(self)
            }

            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Bytes(Cow::Owned(v.to_vec())))
            }

            #[inline]
            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Bytes(Cow::Borrowed(v)))
            }

            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Bytes(Cow::Owned(v)))
            }

            #[inline]
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::String(Cow::Owned(v)))
            }

            #[inline]
            fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::U128(v))
            }

            #[inline]
            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::U16(v))
            }

            #[inline]
            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::U32(v))
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::U64(v))
            }
        }

        deserializer.deserialize_any(ValueKindVisitor)
    }
}

/// Deserialize implementation for `ValueOwned`.
impl<'de> Deserialize<'de> for ValueOwned {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Value::deserialize(deserializer).map(|value| value.into_owned())
    }
}

pub struct ValueDeserializer<'de> {
    value: Value<'de>,
}

impl<'de> de::IntoDeserializer<'de, ValueError> for Value<'de> {
    type Deserializer = ValueDeserializer<'de>;

    #[inline]
    fn into_deserializer(self) -> Self::Deserializer {
        ValueDeserializer { value: self }
    }
}

impl<'de> de::IntoDeserializer<'de, ValueError> for ValueOwned {
    type Deserializer = ValueDeserializer<'de>;

    #[inline]
    fn into_deserializer(self) -> Self::Deserializer {
        ValueDeserializer {
            value: self.into_inner(),
        }
    }
}

/// Implement `Deserializer` for `Value<'de>`.
impl<'de> de::Deserializer<'de> for ValueDeserializer<'de> {
    type Error = ValueError;

    serde::forward_to_deserialize_any! {
        newtype_struct
        tuple tuple_struct map struct
        identifier ignored_any
    }

    impl_deserialize_number!(u8, deserialize_u8, visit_u8);

    impl_deserialize_number!(u16, deserialize_u16, visit_u16);

    impl_deserialize_number!(u32, deserialize_u32, visit_u32);

    impl_deserialize_number!(u64, deserialize_u64, visit_u64);

    impl_deserialize_number!(u128, deserialize_u128, visit_u128);

    impl_deserialize_number!(i8, deserialize_i8, visit_i8);

    impl_deserialize_number!(i16, deserialize_i16, visit_i16);

    impl_deserialize_number!(i32, deserialize_i32, visit_i32);

    impl_deserialize_number!(i64, deserialize_i64, visit_i64);

    impl_deserialize_number!(i128, deserialize_i128, visit_i128);

    impl_deserialize_number!(f32, deserialize_f32, visit_f32);

    impl_deserialize_number!(f64, deserialize_f64, visit_f64);

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::String(Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
            Value::String(Cow::Owned(s)) => visitor.visit_string(s),
            Value::Bytes(Cow::Borrowed(b)) => visitor.visit_borrowed_bytes(b),
            Value::Bytes(Cow::Owned(b)) => visitor.visit_byte_buf(b),
            Value::BytesOwned(b) => visitor.visit_bytes(b.as_ref()),
            Value::U8(v) => visitor.visit_u8(v),
            Value::U16(v) => visitor.visit_u16(v),
            Value::U32(v) => visitor.visit_u32(v),
            Value::U64(v) => visitor.visit_u64(v),
            Value::U128(v) => visitor.visit_u128(v),
            Value::I8(v) => visitor.visit_i8(v),
            Value::I16(v) => visitor.visit_i16(v),
            Value::I32(v) => visitor.visit_i32(v),
            Value::I64(v) => visitor.visit_i64(v),
            Value::I128(v) => visitor.visit_i128(v),
            Value::F32(OrderedFloat(v)) => visitor.visit_f32(v),
            Value::F64(OrderedFloat(v)) => visitor.visit_f64(v),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::Char(v) => visitor.visit_char(v),
            Value::Null => visitor.visit_none(),
            Value::Unit => visitor.visit_unit(),
            Value::Array(a) => visitor.visit_seq(SeqDeserializer::new(a.into_iter())),
            Value::Map(m) => visitor.visit_map(MapDeserializer::new(m.into_iter())),
            Value::Object(o) => visitor.visit_map(MapDeserializer::new(o.into_iter().map(|(k, v)| (k.into(), v)))),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Array(a) => visitor.visit_seq(SeqDeserializer::new(a.into_iter())),
            v => visitor.visit_seq(SeqDeserializer::new(std::iter::once(v))),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Null | Value::Unit => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Map(m) => {
                let mut iter = m.into_iter();
                let (key, value) = iter
                    .next()
                    .ok_or(serde::de::Error::invalid_type(de::Unexpected::Map, &"map with single key"))?;

                if iter.next().is_some() {
                    return Err(serde::de::Error::invalid_type(de::Unexpected::Map, &"map with single key"));
                }

                visitor.visit_enum(EnumDeserializer {
                    variant: key,
                    value: Some(value),
                })
            }
            Value::Object(o) => {
                let mut iter = o.into_iter();
                let (key, value) = iter
                    .next()
                    .ok_or(serde::de::Error::invalid_type(de::Unexpected::Map, &"map with single key"))?;

                if iter.next().is_some() {
                    return Err(serde::de::Error::invalid_type(de::Unexpected::Map, &"map with single key"));
                }

                visitor.visit_enum(EnumDeserializer {
                    variant: key.into(),
                    value: Some(value),
                })
            }
            Value::String(_) => visitor.visit_enum(EnumDeserializer {
                variant: self.value,
                value: None,
            }),
            other => Err(serde::de::Error::invalid_type(other.unexpected(), &"map or string")),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let v = match self.value {
            Value::String(ref s) => parse_bool_primitive(s),
            Value::Bool(v) => Some(v),
            Value::U8(v) => Some(v != 0),
            Value::U16(v) => Some(v != 0),
            Value::U32(v) => Some(v != 0),
            Value::U64(v) => Some(v != 0),
            Value::U128(v) => Some(v != 0),
            Value::I8(v) => Some(v != 0),
            Value::I16(v) => Some(v != 0),
            Value::I32(v) => Some(v != 0),
            Value::I64(v) => Some(v != 0),
            Value::I128(v) => Some(v != 0),
            Value::F32(v) => Some(v != 0.0),
            Value::F64(v) => Some(v != 0.0),
            Value::Char('t' | 'T' | '1' | 'y' | 'Y') => Some(true),
            Value::Char('f' | 'F' | '0' | 'n' | 'N') => Some(false),
            _ => None,
        };

        if let Some(v) = v {
            visitor.visit_bool(v)
        } else {
            Err(serde::de::Error::invalid_type(self.value.unexpected(), &"bool"))
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let v = match self.value {
            Value::String(ref s) => Some(parse_char_primitive(s)?),
            Value::Bytes(ref b) => {
                let s = std::str::from_utf8(b).map_err(serde::de::Error::custom)?;
                Some(parse_char_primitive(s)?)
            }
            Value::Char(v) => Some(v),
            Value::U8(v) => num_traits::cast::cast::<_, u8>(v).map(Into::into),
            Value::U16(v) => num_traits::cast::cast::<_, u32>(v).and_then(|u| char::try_from(u).ok()),
            Value::U32(v) => num_traits::cast::cast::<_, u32>(v).and_then(|u| char::try_from(u).ok()),
            Value::U64(v) => num_traits::cast::cast::<_, u32>(v).and_then(|u| char::try_from(u).ok()),
            Value::U128(v) => num_traits::cast::cast::<_, u32>(v).and_then(|u| char::try_from(u).ok()),
            Value::I8(v) => num_traits::cast::cast::<_, u8>(v).map(Into::into),
            Value::I16(v) => num_traits::cast::cast::<_, u32>(v).and_then(|u| char::try_from(u).ok()),
            Value::I32(v) => num_traits::cast::cast::<_, u32>(v).and_then(|u| char::try_from(u).ok()),
            Value::I64(v) => num_traits::cast::cast::<_, u32>(v).and_then(|u| char::try_from(u).ok()),
            Value::I128(v) => num_traits::cast::cast::<_, u32>(v).and_then(|u| char::try_from(u).ok()),
            Value::F32(OrderedFloat(v)) => num_traits::cast::cast::<_, u32>(v).and_then(|u| char::try_from(u).ok()),
            Value::F64(OrderedFloat(v)) => num_traits::cast::cast::<_, u32>(v).and_then(|u| char::try_from(u).ok()),
            _ => None,
        };

        if let Some(v) = v {
            visitor.visit_char(v)
        } else {
            Err(serde::de::Error::invalid_type(self.value.unexpected(), &"char"))
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Bytes(Cow::Borrowed(b)) => visitor.visit_borrowed_bytes(b),
            Value::Bytes(Cow::Owned(b)) => visitor.visit_byte_buf(b),
            Value::Array(a) => {
                let mut seq = SeqDeserializer::new(a.into_iter());
                let mut bytes = Vec::new();
                while let Some(value) = seq.next_element::<u8>()? {
                    bytes.push(value);
                }
                visitor.visit_byte_buf(bytes)
            }
            Value::String(Cow::Borrowed(s)) => visitor.visit_borrowed_bytes(s.as_bytes()),
            Value::String(Cow::Owned(s)) => visitor.visit_byte_buf(s.into_bytes()),
            _ => Err(serde::de::Error::invalid_type(self.value.unexpected(), &"bytes")),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::String(Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
            Value::String(Cow::Owned(s)) => visitor.visit_string(s),
            Value::Bytes(Cow::Borrowed(b)) => {
                let s = std::str::from_utf8(b).map_err(serde::de::Error::custom)?;
                visitor.visit_borrowed_str(s)
            }
            Value::Bytes(Cow::Owned(b)) => {
                let s = String::from_utf8(b).map_err(serde::de::Error::custom)?;
                visitor.visit_string(s)
            }
            Value::Char(c) => visitor.visit_str(c.encode_utf8(&mut [0; 4])),
            Value::Bool(b) => visitor.visit_borrowed_str(if b { "true" } else { "false" }),
            Value::U8(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::U16(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::U32(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::U64(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::U128(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::I8(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::I16(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::I32(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::I64(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::I128(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::F32(OrderedFloat(v)) => visitor.visit_str(ryu::Buffer::new().format(v)),
            Value::F64(OrderedFloat(v)) => visitor.visit_str(ryu::Buffer::new().format(v)),
            _ => Err(serde::de::Error::invalid_type(self.value.unexpected(), &"string")),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Unit => visitor.visit_unit(),
            _ => Err(serde::de::Error::invalid_type(self.value.unexpected(), &"unit")),
        }
    }

    #[inline]
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    #[inline]
    fn deserialize_unit_struct<V>(self, _: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }
}

/// Helper deserializer for enums.
struct EnumDeserializer<'de> {
    variant: Value<'de>,
    value: Option<Value<'de>>,
}

impl<'de> de::EnumAccess<'de> for EnumDeserializer<'de> {
    type Error = ValueError;
    type Variant = VariantDeserializer<'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.variant.into_deserializer())
            .map(|v| (v, VariantDeserializer { value: self.value }))
    }
}

/// Helper for deserializing enum variants.
struct VariantDeserializer<'de> {
    value: Option<Value<'de>>,
}

impl<'de> de::VariantAccess<'de> for VariantDeserializer<'de> {
    type Error = ValueError;

    #[inline]
    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            Some(value) => de::Deserialize::deserialize(value.into_deserializer()),
            None => Ok(()),
        }
    }

    #[inline]
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value.into_deserializer()),
            None => Err(serde::de::Error::invalid_type(
                de::Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let Some(value) = self.value else {
            return Err(serde::de::Error::invalid_type(de::Unexpected::UnitVariant, &"tuple variant"));
        };

        match value {
            Value::Array(v) => {
                if v.len() != len {
                    return Err(serde::de::Error::invalid_length(v.len(), &"tuple variant"));
                }

                if v.is_empty() {
                    visitor.visit_unit()
                } else {
                    visitor.visit_seq(SeqDeserializer::new(v.into_iter()))
                }
            }
            other => Err(serde::de::Error::invalid_type(other.unexpected(), &"tuple variant")),
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let Some(value) = self.value else {
            return Err(serde::de::Error::invalid_type(de::Unexpected::UnitVariant, &"struct variant"));
        };

        if matches!(value, Value::Map(_)) {
            value.into_deserializer().deserialize_map(visitor)
        } else {
            Err(serde::de::Error::invalid_type(value.unexpected(), &"struct variant"))
        }
    }
}

/// A helper struct to deserialize sequences.
struct SeqDeserializer<I> {
    iter: I,
}

impl<I> SeqDeserializer<I> {
    #[inline]
    fn new(values: I) -> Self {
        Self { iter: values }
    }
}

impl<'de, I> de::SeqAccess<'de> for SeqDeserializer<I>
where
    I: Iterator<Item = Value<'de>>,
{
    type Error = ValueError;

    #[inline]
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let Some(value) = self.iter.next() else {
            return Ok(None);
        };

        seed.deserialize(value.into_deserializer()).map(Some)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.iter.size_hint().1
    }
}

/// A helper struct to deserialize maps.
struct MapDeserializer<'de, I> {
    iter: I,
    value: Option<Value<'de>>,
}

impl<I> MapDeserializer<'_, I> {
    #[inline]
    fn new(values: I) -> Self {
        Self {
            iter: values,
            value: None,
        }
    }
}

impl<'de, I> de::MapAccess<'de> for MapDeserializer<'de, I>
where
    I: Iterator<Item = (Value<'de>, Value<'de>)>,
{
    type Error = ValueError;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        let Some((key, value)) = self.iter.next() else {
            return Ok(None);
        };

        self.value = Some(value);
        seed.deserialize(key.into_deserializer()).map(Some)
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let Some(value) = self.value.take() else {
            return Err(serde::de::Error::invalid_value(de::Unexpected::Map, &"missing map value"));
        };

        seed.deserialize(value.into_deserializer())
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.iter.size_hint().1
    }
}

use core::fmt;
use std::borrow::Cow;

use ordered_float::OrderedFloat;
use serde::de::{self, SeqAccess};
use serde::{Deserialize, Deserializer};

use super::{Object, ObjectOwned, Value, ValueError};
use crate::value::{Map, ValueOwned};

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

        impl<'de, K, V> serde::de::Visitor<'de> for MapVisitor<K, V>
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
                D: serde::de::MapAccess<'de>,
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

impl<'de> serde::Deserializer<'de> for Object<'de> {
    type Error = ValueError;

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let count = self.0.len();
        visitor.visit_map(MapDeserializer::new(
            self.0.into_iter().map(|(k, v)| (k.into(), v)),
            Some(count),
        ))
    }
}

impl<'de> de::IntoDeserializer<'de, ValueError> for Object<'de> {
    type Deserializer = Self;

    #[inline]
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> serde::de::Deserialize<'de> for Value<'de> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ValueKindVisitor;

        impl<'de> serde::de::Visitor<'de> for ValueKindVisitor {
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
                let mut values = Vec::with_capacity(visitor.size_hint().map_or(0, |n| n));
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

impl<'de> Deserialize<'de> for ValueOwned {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Value::deserialize(deserializer).map(|value| ValueOwned(value.into_owned()))
    }
}

trait PrimParse: Sized {
    const NAME: &'static str;

    fn parse_prim(s: &str) -> Result<Self, impl std::fmt::Display>;
}

macro_rules! impl_prim_parse_number {
        ($parse_fn:path, $($ty:ty),*) => {
            $(
                impl PrimParse for $ty {
                    const NAME: &'static str = stringify!($ty);

                    fn parse_prim(s: &str) -> Result<Self, impl std::fmt::Display> {
                        $parse_fn(s)
                    }
                }
            )*
        };
    }

impl_prim_parse_number!(std::str::FromStr::from_str, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
impl_prim_parse_number!(std::str::FromStr::from_str, f32, f64);

macro_rules! impl_deserialize_number {
    ($ty:ty, $deserialize_fn:ident, $visit_fn:ident) => {
        #[inline]
        fn $deserialize_fn<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            let v = match self {
                Self::String(ref s) => Some(<$ty as PrimParse>::parse_prim(&s).map_err(serde::de::Error::custom)?),
                Self::U8(v) => num_traits::cast::cast(v),
                Self::U16(v) => num_traits::cast::cast(v),
                Self::U32(v) => num_traits::cast::cast(v),
                Self::U64(v) => num_traits::cast::cast(v),
                Self::U128(v) => num_traits::cast::cast(v),
                Self::I8(v) => num_traits::cast::cast(v),
                Self::I16(v) => num_traits::cast::cast(v),
                Self::I32(v) => num_traits::cast::cast(v),
                Self::I64(v) => num_traits::cast::cast(v),
                Self::I128(v) => num_traits::cast::cast(v),
                Self::F32(v) => num_traits::cast::cast(v),
                Self::F64(v) => num_traits::cast::cast(v),
                Self::Bool(v) => num_traits::cast::cast(v as u8),
                _ => None,
            };

            if let Some(v) = v {
                visitor.$visit_fn(v)
            } else {
                Err(serde::de::Error::invalid_type(
                    self.unexpected(),
                    &<$ty as PrimParse>::NAME,
                ))
            }
        }
    };
}

fn parse_bool_primitive(s: &str) -> Option<bool> {
    match s {
        "true" | "1" | "yes" | "on" | "enable" | "enabled" | "t" | "y" => Some(true),
        "false" | "0" | "no" | "off" | "disable" | "disabled" | "f" | "n" => Some(false),
        _ => None,
    }
}

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

impl<'de> de::Deserializer<'de> for Value<'de> {
    type Error = ValueError;

    serde::forward_to_deserialize_any! {
        newtype_struct seq
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

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Null | Self::Unit => visitor.visit_none(),
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
        match self {
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
            Value::String(_) => visitor.visit_enum(EnumDeserializer {
                variant: self,
                value: None,
            }),
            other => Err(serde::de::Error::invalid_type(other.unexpected(), &"map or string")),
        }
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::String(Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
            Self::String(Cow::Owned(s)) => visitor.visit_string(s),
            Self::Bytes(Cow::Borrowed(b)) => visitor.visit_borrowed_bytes(b),
            Self::Bytes(Cow::Owned(b)) => visitor.visit_byte_buf(b),
            Self::U8(v) => visitor.visit_u8(v),
            Self::U16(v) => visitor.visit_u16(v),
            Self::U32(v) => visitor.visit_u32(v),
            Self::U64(v) => visitor.visit_u64(v),
            Self::U128(v) => visitor.visit_u128(v),
            Self::I8(v) => visitor.visit_i8(v),
            Self::I16(v) => visitor.visit_i16(v),
            Self::I32(v) => visitor.visit_i32(v),
            Self::I64(v) => visitor.visit_i64(v),
            Self::I128(v) => visitor.visit_i128(v),
            Self::F32(OrderedFloat(v)) => visitor.visit_f32(v),
            Self::F64(OrderedFloat(v)) => visitor.visit_f64(v),
            Self::Bool(v) => visitor.visit_bool(v),
            Self::Char(v) => visitor.visit_char(v),
            Self::Null => visitor.visit_unit(),
            Self::Unit => visitor.visit_unit(),
            Self::Array(a) => {
                let count = a.len();
                visitor.visit_seq(SeqDeserializer::new(a.into_iter(), Some(count)))
            }
            Self::Map(m) => {
                let count = m.len();
                visitor.visit_map(MapDeserializer::new(m.into_iter(), Some(count)))
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let v = match self {
            Value::String(ref s) => parse_bool_primitive(s),
            Self::Bool(v) => Some(v),
            Self::U8(v) => Some(v != 0),
            Self::U16(v) => Some(v != 0),
            Self::U32(v) => Some(v != 0),
            Self::U64(v) => Some(v != 0),
            Self::U128(v) => Some(v != 0),
            Self::I8(v) => Some(v != 0),
            Self::I16(v) => Some(v != 0),
            Self::I32(v) => Some(v != 0),
            Self::I64(v) => Some(v != 0),
            Self::I128(v) => Some(v != 0),
            Self::F32(v) => Some(v != 0.0),
            Self::F64(v) => Some(v != 0.0),
            Self::Char('t' | 'T' | '1' | 'y' | 'Y') => Some(true),
            Self::Char('f' | 'F' | '0' | 'n' | 'N') => Some(false),
            _ => None,
        };

        if let Some(v) = v {
            visitor.visit_bool(v)
        } else {
            Err(serde::de::Error::invalid_type(self.unexpected(), &"bool"))
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        fn try_char<U, T>(v: U) -> Result<char, ValueError>
        where
            U: num_traits::cast::FromPrimitive + num_traits::cast::NumCast,
            T: num_traits::cast::FromPrimitive + num_traits::cast::NumCast,
            char: std::convert::TryFrom<T>,
            <char as std::convert::TryFrom<T>>::Error: std::fmt::Display,
        {
            let u = num_traits::cast::cast::<U, T>(v).ok_or_else(|| serde::de::Error::custom("expected char"))?;
            char::try_from(u).map_err(serde::de::Error::custom)
        }

        let v = match self {
            Value::String(ref s) => Some(parse_char_primitive(s)?),
            Value::Bytes(ref b) => {
                let s = std::str::from_utf8(b).map_err(serde::de::Error::custom)?;
                Some(parse_char_primitive(s)?)
            }
            Self::Char(v) => Some(v),
            Self::U8(v) => Some(try_char::<_, u8>(v)?),
            Self::U16(v) => Some(try_char::<_, u32>(v)?),
            Self::U32(v) => Some(try_char::<_, u32>(v)?),
            Self::U64(v) => Some(try_char::<_, u32>(v)?),
            Self::U128(v) => Some(try_char::<_, u32>(v)?),
            Self::I8(v) => Some(try_char::<_, u8>(v)?),
            Self::I16(v) => Some(try_char::<_, u32>(v)?),
            Self::I32(v) => Some(try_char::<_, u32>(v)?),
            Self::I64(v) => Some(try_char::<_, u32>(v)?),
            Self::I128(v) => Some(try_char::<_, u32>(v)?),
            Self::F32(v) => Some(try_char::<_, u32>(v)?),
            Self::F64(v) => Some(try_char::<_, u32>(v)?),
            _ => None,
        };

        if let Some(v) = v {
            visitor.visit_char(v)
        } else {
            Err(serde::de::Error::invalid_type(self.unexpected(), &"char"))
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::String(Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
            Self::String(Cow::Owned(s)) => visitor.visit_string(s),
            Self::Bytes(Cow::Borrowed(b)) => {
                let s = std::str::from_utf8(b).map_err(serde::de::Error::custom)?;
                visitor.visit_borrowed_str(s)
            }
            Self::Bytes(Cow::Owned(b)) => {
                let s = String::from_utf8(b).map_err(serde::de::Error::custom)?;
                visitor.visit_string(s)
            }
            Self::Char(c) => visitor.visit_str(c.encode_utf8(&mut [0; 4])),
            Self::Bool(b) => visitor.visit_borrowed_str(if b { "true" } else { "false" }),
            Self::U8(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::U16(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::U32(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::U64(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::U128(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::I8(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::I16(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::I32(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::I64(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::I128(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::F32(OrderedFloat(v)) => visitor.visit_str(ryu::Buffer::new().format(v)),
            Self::F64(OrderedFloat(v)) => visitor.visit_str(ryu::Buffer::new().format(v)),
            _ => Err(serde::de::Error::invalid_type(self.unexpected(), &"string")),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::Bytes(Cow::Borrowed(b)) => visitor.visit_borrowed_bytes(b),
            Value::Bytes(Cow::Owned(b)) => visitor.visit_byte_buf(b),
            Value::Array(a) => {
                let count = a.len();
                let mut seq = SeqDeserializer::new(a.into_iter(), Some(count));
                let mut bytes = Vec::new();
                while let Some(value) = seq.next_element::<u8>()? {
                    bytes.push(value);
                }

                visitor.visit_byte_buf(bytes)
            }
            _ => Err(serde::de::Error::invalid_type(self.unexpected(), &"bytes")),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::Bytes(Cow::Borrowed(b)) => visitor.visit_borrowed_bytes(b),
            Value::Bytes(Cow::Owned(b)) => visitor.visit_byte_buf(b),
            Value::Array(a) => {
                let count = a.len();
                let mut seq = SeqDeserializer::new(a.into_iter(), Some(count));
                let mut bytes = Vec::new();
                while let Some(value) = seq.next_element::<u8>()? {
                    bytes.push(value);
                }
                visitor.visit_byte_buf(bytes)
            }
            _ => Err(serde::de::Error::invalid_type(self.unexpected(), &"bytes")),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
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
            Self::Char(c) => visitor.visit_str(c.encode_utf8(&mut [0; 4])),
            Self::Bool(b) => visitor.visit_borrowed_str(if b { "true" } else { "false" }),
            Self::U8(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::U16(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::U32(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::U64(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::U128(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::I8(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::I16(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::I32(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::I64(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::I128(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Self::F32(OrderedFloat(v)) => visitor.visit_str(ryu::Buffer::new().format(v)),
            Self::F64(OrderedFloat(v)) => visitor.visit_str(ryu::Buffer::new().format(v)),
            _ => Err(serde::de::Error::invalid_type(self.unexpected(), &"string")),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::Unit => visitor.visit_unit(),
            _ => Err(serde::de::Error::invalid_type(self.unexpected(), &"unit")),
        }
    }

    #[inline]
    fn deserialize_unit_struct<V>(self, _: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }
}

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
        seed.deserialize(self.variant)
            .map(|v| (v, VariantDeserializer { value: self.value }))
    }
}

struct VariantDeserializer<'de> {
    value: Option<Value<'de>>,
}

impl<'de> de::VariantAccess<'de> for VariantDeserializer<'de> {
    type Error = ValueError;

    #[inline]
    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            Some(value) => de::Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    #[inline]
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
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
                    let count = v.len();
                    visitor.visit_seq(SeqDeserializer::new(v.into_iter(), Some(count)))
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
            value.deserialize_map(visitor)
        } else {
            Err(serde::de::Error::invalid_type(value.unexpected(), &"struct variant"))
        }
    }
}

struct SeqDeserializer<I> {
    iter: I,
    size: Option<usize>,
}

impl<I> SeqDeserializer<I> {
    #[inline]
    fn new(values: I, size: Option<usize>) -> Self {
        Self { iter: values, size }
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

        seed.deserialize(value).map(Some)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.size.or_else(|| self.iter.size_hint().1)
    }
}

struct MapDeserializer<'de, I> {
    iter: I,
    value: Option<Value<'de>>,
    size: Option<usize>,
}

impl<I> MapDeserializer<'_, I> {
    #[inline]
    fn new(values: I, size: Option<usize>) -> Self {
        Self {
            iter: values,
            value: None,
            size,
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

        seed.deserialize(key).map(Some)
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let Some(value) = self.value.take() else {
            return Err(serde::de::Error::invalid_value(de::Unexpected::Map, &"missing map value"));
        };

        seed.deserialize(value)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.size.or_else(|| self.iter.size_hint().1.map(|s| s / 2))
    }
}

impl<'de> de::Deserialize<'de> for Object<'de> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        serde::de::Deserialize::deserialize(deserializer).map(Object)
    }
}

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

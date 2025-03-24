use core::fmt;
use std::borrow::Cow;

use ordered_float::OrderedFloat;
use serde::de::{self, IntoDeserializer, SeqAccess};
use serde::{Deserialize, Deserializer};

use super::{Object, Value, ValueError};
use crate::value::{Map, ValueOwned, ValuePrimitive};

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

impl<'de> de::IntoDeserializer<'de, ValueError> for Value<'de> {
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
                Ok(Value::Primitive(ValuePrimitive::Bool(v)))
            }

            #[inline]
            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Primitive(ValuePrimitive::I8(v)))
            }

            #[inline]
            fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Primitive(ValuePrimitive::Char(v)))
            }

            #[inline]
            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Primitive(ValuePrimitive::U8(v)))
            }

            #[inline]
            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Primitive(ValuePrimitive::F32(OrderedFloat(v))))
            }

            #[inline]
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Primitive(ValuePrimitive::F64(OrderedFloat(v))))
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
                Ok(Value::Primitive(ValuePrimitive::I128(v)))
            }

            #[inline]
            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Primitive(ValuePrimitive::I16(v)))
            }

            #[inline]
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Primitive(ValuePrimitive::I32(v)))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Primitive(ValuePrimitive::I64(v)))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Primitive(ValuePrimitive::Null))
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
                Ok(Value::Primitive(ValuePrimitive::Unit))
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
                Ok(Value::Primitive(ValuePrimitive::U128(v)))
            }

            #[inline]
            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Primitive(ValuePrimitive::U16(v)))
            }

            #[inline]
            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Primitive(ValuePrimitive::U32(v)))
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Primitive(ValuePrimitive::U64(v)))
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
        Value::deserialize(deserializer).map(|v| match v {
            Value::String(s) => ValueOwned::String(s.into_owned()),
            Value::Bytes(b) => ValueOwned::Bytes(b.into_owned()),
            Value::Primitive(p) => ValueOwned::Primitive(p),
            Value::Array(a) => ValueOwned::Array(a.into_iter().map(|v| v.into_owned()).collect()),
            Value::Map(m) => ValueOwned::Map(m.into_iter().map(|(k, v)| (k.into_owned(), v.into_owned())).collect()),
        })
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
            match self {
                Self::Primitive(p) => p.$deserialize_fn(visitor),
                Self::String(s) => visitor.$visit_fn(<$ty as PrimParse>::parse_prim(&s).map_err(serde::de::Error::custom)?),
                _ => Err(serde::de::Error::invalid_type(
                    self.unexpected(),
                    &<$ty as PrimParse>::NAME,
                )),
            }
        }
    };
}

macro_rules! impl_deserialize_number_primitive {
    ($ty:ty, $deserialize_fn:ident, $visit_fn:ident) => {
        #[inline]
        fn $deserialize_fn<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            let v = match self {
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

impl<'de> de::Deserialize<'de> for ValuePrimitive {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ValuePrimitiveVisitor;

        impl de::Visitor<'_> for ValuePrimitiveVisitor {
            type Value = ValuePrimitive;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a primitive value")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValuePrimitive::Bool(v))
            }

            fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValuePrimitive::Char(v))
            }

            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValuePrimitive::U8(v))
            }

            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValuePrimitive::U16(v))
            }

            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValuePrimitive::U32(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValuePrimitive::U64(v))
            }

            fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValuePrimitive::U128(v))
            }

            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValuePrimitive::I8(v))
            }

            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValuePrimitive::I16(v))
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValuePrimitive::I32(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValuePrimitive::I64(v))
            }

            fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValuePrimitive::I128(v))
            }

            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValuePrimitive::F32(v.into()))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValuePrimitive::F64(v.into()))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValuePrimitive::Unit)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValuePrimitive::Null)
            }
        }

        deserializer.deserialize_any(ValuePrimitiveVisitor)
    }
}

impl<'de> de::Deserializer<'de> for ValuePrimitive {
    type Error = ValueError;

    serde::forward_to_deserialize_any! {
        newtype_struct seq byte_buf
        tuple tuple_struct map struct
        identifier ignored_any bytes
        enum
    }

    impl_deserialize_number_primitive!(u8, deserialize_u8, visit_u8);

    impl_deserialize_number_primitive!(u16, deserialize_u16, visit_u16);

    impl_deserialize_number_primitive!(u32, deserialize_u32, visit_u32);

    impl_deserialize_number_primitive!(u64, deserialize_u64, visit_u64);

    impl_deserialize_number_primitive!(u128, deserialize_u128, visit_u128);

    impl_deserialize_number_primitive!(i8, deserialize_i8, visit_i8);

    impl_deserialize_number_primitive!(i16, deserialize_i16, visit_i16);

    impl_deserialize_number_primitive!(i32, deserialize_i32, visit_i32);

    impl_deserialize_number_primitive!(i64, deserialize_i64, visit_i64);

    impl_deserialize_number_primitive!(i128, deserialize_i128, visit_i128);

    impl_deserialize_number_primitive!(f32, deserialize_f32, visit_f32);

    impl_deserialize_number_primitive!(f64, deserialize_f64, visit_f64);

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
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
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let v = match self {
            Self::Bool(v) => v,
            Self::U8(v) => v != 0,
            Self::U16(v) => v != 0,
            Self::U32(v) => v != 0,
            Self::U64(v) => v != 0,
            Self::U128(v) => v != 0,
            Self::I8(v) => v != 0,
            Self::I16(v) => v != 0,
            Self::I32(v) => v != 0,
            Self::I64(v) => v != 0,
            Self::I128(v) => v != 0,
            Self::F32(v) => v != 0.0,
            Self::F64(v) => v != 0.0,
            Self::Char('t' | 'T' | '1' | 'y' | 'Y') => true,
            Self::Char('f' | 'F' | '0' | 'n' | 'N') => false,
            _ => return Err(serde::de::Error::invalid_type(self.unexpected(), &"bool")),
        };

        visitor.visit_bool(v)
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
            Self::Char(v) => v,
            Self::U8(v) => try_char::<_, u8>(v)?,
            Self::U16(v) => try_char::<_, u32>(v)?,
            Self::U32(v) => try_char::<_, u32>(v)?,
            Self::U64(v) => try_char::<_, u32>(v)?,
            Self::U128(v) => try_char::<_, u32>(v)?,
            Self::I8(v) => try_char::<_, u8>(v)?,
            Self::I16(v) => try_char::<_, u32>(v)?,
            Self::I32(v) => try_char::<_, u32>(v)?,
            Self::I64(v) => try_char::<_, u32>(v)?,
            Self::I128(v) => try_char::<_, u32>(v)?,
            Self::F32(v) => try_char::<_, u32>(v)?,
            Self::F64(v) => try_char::<_, u32>(v)?,
            _ => return Err(serde::de::Error::invalid_type(self.unexpected(), &"char")),
        };

        visitor.visit_char(v)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
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
            _ => Err(serde::de::Error::invalid_type(self.unexpected(), &"str")),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Self::Null | Self::Unit => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }
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
            Value::Primitive(p) => p.deserialize_option(visitor),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
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
            Value::Primitive(p) => p.deserialize_enum(name, variants, visitor),
            other => Err(serde::de::Error::invalid_type(other.unexpected(), &"map or string")),
        }
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::String(Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
            Value::String(Cow::Owned(s)) => visitor.visit_string(s),
            Value::Bytes(Cow::Borrowed(b)) => visitor.visit_borrowed_bytes(b),
            Value::Bytes(Cow::Owned(b)) => visitor.visit_byte_buf(b),
            Value::Primitive(primitive) => primitive.deserialize_any(visitor),
            Value::Array(a) => {
                let count = a.len();
                visitor.visit_seq(SeqDeserializer::new(a.into_iter(), Some(count)))
            }
            Value::Map(m) => {
                let count = m.len();
                visitor.visit_map(MapDeserializer::new(m.into_iter(), Some(count)))
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::String(ref s) => visitor.visit_bool(
                parse_bool_primitive(s).ok_or_else(|| serde::de::Error::invalid_type(self.unexpected(), &"bool"))?,
            ),
            Value::Primitive(p) => p.deserialize_bool(visitor),
            _ => Err(serde::de::Error::invalid_type(self.unexpected(), &"bool")),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::String(s) => visitor.visit_char(parse_char_primitive(&s).map_err(serde::de::Error::custom)?),
            Value::Bytes(b) => {
                let s = std::str::from_utf8(&b).map_err(serde::de::Error::custom)?;
                visitor.visit_char(parse_char_primitive(s).map_err(serde::de::Error::custom)?)
            }
            Value::Primitive(p) => p.deserialize_char(visitor),
            _ => Err(serde::de::Error::invalid_type(self.unexpected(), &"char")),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
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
            Value::Primitive(p) => p.deserialize_str(visitor),
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
            Value::Primitive(p) => p.deserialize_bytes(visitor),
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
            Value::Primitive(p) => p.deserialize_bytes(visitor),
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
            Value::Primitive(p) => p.deserialize_string(visitor),
            _ => Err(serde::de::Error::invalid_type(self.unexpected(), &"string")),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self {
            Value::Primitive(p) => p.deserialize_unit(visitor),
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
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
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

use core::fmt;
use std::borrow::Cow;

use ordered_float::OrderedFloat;
use serde::Deserializer;
use serde::de::{self, IntoDeserializer, SeqAccess};

use super::{Object, Value, ValueError, ValueKind};
use crate::value::Map;

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

impl<'de> serde::Deserializer<'de> for Object<'de, '_> {
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
            self.0.into_iter().flat_map(|(k, v)| {
                [
                    Value::new(
                        match k {
                            Cow::Borrowed(k) => ValueKind::StringRef(k),
                            Cow::Owned(k) => ValueKind::String(k),
                        },
                        match &v.config {
                            Cow::Borrowed(config) => Cow::Borrowed(*config),
                            Cow::Owned(config) => Cow::Owned(config.clone()),
                        },
                    ),
                    v,
                ]
            }),
            Some(count),
        ))
    }
}

impl<'de> de::IntoDeserializer<'de, ValueError> for Value<'de, '_> {
    type Deserializer = Self;

    #[inline]
    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> serde::de::Deserialize<'de> for ValueKind<'de> {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct ValueKindVisitor;

        impl<'de> serde::de::Visitor<'de> for ValueKindVisitor {
            type Value = ValueKind<'de>;

            #[inline]
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a value")
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::Bool(v))
            }

            #[inline]
            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::I8(v))
            }

            #[inline]
            fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::Char(v))
            }

            #[inline]
            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::U8(v))
            }

            #[inline]
            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::F32(OrderedFloat(v)))
            }

            #[inline]
            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::F64(OrderedFloat(v)))
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::String(v.to_string()))
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::StringRef(v))
            }

            #[inline]
            fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::I128(v))
            }

            #[inline]
            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::I16(v))
            }

            #[inline]
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::I32(v))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::I64(v))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::Null)
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::SeqAccess<'de>,
            {
                let mut values = Vec::with_capacity(visitor.size_hint().map_or(0, |n| n));
                while let Some(value) = visitor.next_element()? {
                    values.push(value);
                }
                Ok(ValueKind::Array(values))
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut map = Map::with_capacity(visitor.size_hint().unwrap_or(0));
                while let Some((key, value)) = visitor.next_entry()? {
                    map.push(key, value);
                }
                Ok(ValueKind::Map(map))
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::Unit)
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
                Ok(ValueKind::Bytes(v.to_vec()))
            }

            #[inline]
            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::BytesRef(v))
            }

            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::Bytes(v))
            }

            #[inline]
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::String(v))
            }

            #[inline]
            fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::U128(v))
            }

            #[inline]
            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::U16(v))
            }

            #[inline]
            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::U32(v))
            }

            #[inline]
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ValueKind::U64(v))
            }
        }

        deserializer.deserialize_any(ValueKindVisitor)
    }
}

macro_rules! impl_deserialize_primitive {
    ($variant:ident, $deserialize_fn:ident, $visit_fn:ident, $parse_fn:path) => {
        fn $deserialize_fn<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: de::Visitor<'de>,
        {
            let v = match self.kind {
                ValueKind::U8(v) => num_traits::cast::cast(v),
                ValueKind::U16(v) => num_traits::cast::cast(v),
                ValueKind::U32(v) => num_traits::cast::cast(v),
                ValueKind::U64(v) => num_traits::cast::cast(v),
                ValueKind::U128(v) => num_traits::cast::cast(v),
                ValueKind::I8(v) => num_traits::cast::cast(v),
                ValueKind::I16(v) => num_traits::cast::cast(v),
                ValueKind::I32(v) => num_traits::cast::cast(v),
                ValueKind::I64(v) => num_traits::cast::cast(v),
                ValueKind::I128(v) => num_traits::cast::cast(v),
                ValueKind::F32(OrderedFloat(v)) => num_traits::cast::cast(v),
                ValueKind::F64(OrderedFloat(v)) => num_traits::cast::cast(v),
                ValueKind::StringRef(s) if self.config.parse_string_primitive => {
                    Some($parse_fn(s).map_err(serde::de::Error::custom)?)
                }
                ValueKind::String(ref s) if self.config.parse_string_primitive => {
                    Some($parse_fn(s.as_str()).map_err(serde::de::Error::custom)?)
                }
                _ => {
                    return Err(serde::de::Error::invalid_type(
                        self.kind.unexpected(),
                        &stringify!($variant),
                    ))
                }
            };

            let v = v.ok_or_else(|| serde::de::Error::invalid_value(self.kind.unexpected(), &stringify!($variant)))?;

            visitor.$visit_fn(v)
        }
    };
}

fn parse_bool_primitive(s: &str) -> Result<bool, ValueError> {
    const TRUE_VALUES: &[&str] = &["true", "1", "yes", "on", "enable", "enabled", "t", "y"];
    const FALSE_VALUES: &[&str] = &["false", "0", "no", "off", "disable", "disabled", "f", "n"];
    if TRUE_VALUES.contains(&s) {
        Ok(true)
    } else if FALSE_VALUES.contains(&s) {
        Ok(false)
    } else {
        Err(ValueError::Custom(format!("expected bool, got {}", s)))
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

impl<'de> de::Deserializer<'de> for Value<'de, '_> {
    type Error = ValueError;

    serde::forward_to_deserialize_any! {
        newtype_struct seq
        tuple tuple_struct map struct
        identifier ignored_any
    }

    impl_deserialize_primitive!(u8, deserialize_u8, visit_u8, parse_int::parse);

    impl_deserialize_primitive!(u16, deserialize_u16, visit_u16, parse_int::parse);

    impl_deserialize_primitive!(u32, deserialize_u32, visit_u32, parse_int::parse);

    impl_deserialize_primitive!(u64, deserialize_u64, visit_u64, parse_int::parse);

    impl_deserialize_primitive!(u128, deserialize_u128, visit_u128, parse_int::parse);

    impl_deserialize_primitive!(i8, deserialize_i8, visit_i8, parse_int::parse);

    impl_deserialize_primitive!(i16, deserialize_i16, visit_i16, parse_int::parse);

    impl_deserialize_primitive!(i32, deserialize_i32, visit_i32, parse_int::parse);

    impl_deserialize_primitive!(i64, deserialize_i64, visit_i64, parse_int::parse);

    impl_deserialize_primitive!(i128, deserialize_i128, visit_i128, parse_int::parse);

    impl_deserialize_primitive!(f32, deserialize_f32, visit_f32, fast_float2::parse);

    impl_deserialize_primitive!(f64, deserialize_f64, visit_f64, fast_float2::parse);

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.kind {
            ValueKind::Null | ValueKind::Unit => visitor.visit_none(),
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
        match self.kind {
            ValueKind::Map(m) => {
                let mut iter = m.into_iter();
                let (key, value) = iter
                    .next()
                    .ok_or(serde::de::Error::invalid_type(de::Unexpected::Map, &"map with single key"))?;

                if iter.next().is_some() {
                    return Err(serde::de::Error::invalid_type(de::Unexpected::Map, &"map with single key"));
                }

                visitor.visit_enum(EnumDeserializer {
                    variant: Value::new(key, Cow::Borrowed(self.config.as_ref())),
                    value: Some(Value::new(value, Cow::Borrowed(self.config.as_ref()))),
                })
            }
            ValueKind::String(s) => visitor.visit_enum(EnumDeserializer {
                variant: Value::new(ValueKind::String(s), Cow::Borrowed(self.config.as_ref())),
                value: None,
            }),
            ValueKind::StringRef(s) => visitor.visit_enum(EnumDeserializer {
                variant: Value::new(ValueKind::StringRef(s), Cow::Borrowed(self.config.as_ref())),
                value: None,
            }),
            other => Err(serde::de::Error::invalid_type(other.unexpected(), &"map or string")),
        }
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.kind {
            ValueKind::String(s) => visitor.visit_string(s),
            ValueKind::StringRef(s) => visitor.visit_borrowed_str(s),
            ValueKind::F64(f) => visitor.visit_f64(f.into_inner()),
            ValueKind::F32(f) => visitor.visit_f32(f.into_inner()),
            ValueKind::U8(u) => visitor.visit_u8(u),
            ValueKind::U16(u) => visitor.visit_u16(u),
            ValueKind::U32(u) => visitor.visit_u32(u),
            ValueKind::U64(u) => visitor.visit_u64(u),
            ValueKind::U128(u) => visitor.visit_u128(u),
            ValueKind::I8(i) => visitor.visit_i8(i),
            ValueKind::I16(i) => visitor.visit_i16(i),
            ValueKind::I32(i) => visitor.visit_i32(i),
            ValueKind::I64(i) => visitor.visit_i64(i),
            ValueKind::I128(i) => visitor.visit_i128(i),
            ValueKind::Bool(b) => visitor.visit_bool(b),
            ValueKind::Char(c) => visitor.visit_char(c),
            ValueKind::Array(a) => {
                let count = a.len();
                visitor.visit_seq(SeqDeserializer::new(
                    a.into_iter().map(|v| Value::new(v, Cow::Borrowed(self.config.as_ref()))),
                    Some(count),
                ))
            }
            ValueKind::Map(m) => {
                let count = m.len();
                visitor.visit_map(MapDeserializer::new(
                    m.into_iter().flat_map(|(k, v)| {
                        [
                            Value::new(k, Cow::Borrowed(self.config.as_ref())),
                            Value::new(v, Cow::Borrowed(self.config.as_ref())),
                        ]
                    }),
                    Some(count),
                ))
            }
            ValueKind::Bytes(b) => visitor.visit_byte_buf(b),
            ValueKind::BytesRef(b) => visitor.visit_borrowed_bytes(b),
            ValueKind::Null => visitor.visit_none(),
            ValueKind::Unit => visitor.visit_unit(),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let v = match self.kind {
            ValueKind::Bool(b) => b,
            ValueKind::String(s) => parse_bool_primitive(s.as_str()).map_err(serde::de::Error::custom)?,
            ValueKind::StringRef(s) => parse_bool_primitive(s).map_err(serde::de::Error::custom)?,
            ValueKind::U8(u) => u != 0,
            ValueKind::U16(u) => u != 0,
            ValueKind::U32(u) => u != 0,
            ValueKind::U64(u) => u != 0,
            ValueKind::U128(u) => u != 0,
            ValueKind::I8(i) => i != 0,
            ValueKind::I16(i) => i != 0,
            ValueKind::I32(i) => i != 0,
            ValueKind::I64(i) => i != 0,
            ValueKind::I128(i) => i != 0,
            ValueKind::F32(OrderedFloat(f)) => f != 0.0,
            ValueKind::F64(OrderedFloat(f)) => f != 0.0,
            _ => return Err(serde::de::Error::custom("expected bool")),
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
            let u = num_traits::cast::cast::<U, T>(v).ok_or(serde::de::Error::custom("expected char"))?;
            char::try_from(u).map_err(serde::de::Error::custom)
        }

        let v = match self.kind {
            ValueKind::Char(c) => c,
            ValueKind::String(s) => parse_char_primitive(s.as_str()).map_err(serde::de::Error::custom)?,
            ValueKind::StringRef(s) => parse_char_primitive(s).map_err(serde::de::Error::custom)?,
            ValueKind::Bytes(b) => {
                let s = String::from_utf8(b).map_err(serde::de::Error::custom)?;
                parse_char_primitive(s.as_str()).map_err(serde::de::Error::custom)?
            }
            ValueKind::BytesRef(b) => {
                let s = std::str::from_utf8(b).map_err(serde::de::Error::custom)?;
                parse_char_primitive(s).map_err(serde::de::Error::custom)?
            }
            ValueKind::U8(u) => try_char::<_, u8>(u)?,
            ValueKind::U16(u) => try_char::<_, u32>(u)?,
            ValueKind::U32(u) => try_char::<_, u32>(u)?,
            ValueKind::U64(u) => try_char::<_, u32>(u)?,
            ValueKind::U128(u) => try_char::<_, u32>(u)?,
            ValueKind::I8(u) => try_char::<_, u8>(u)?,
            ValueKind::I16(u) => try_char::<_, u32>(u)?,
            ValueKind::I32(u) => try_char::<_, u32>(u)?,
            ValueKind::I64(u) => try_char::<_, u32>(u)?,
            ValueKind::I128(u) => try_char::<_, u32>(u)?,
            ValueKind::F32(OrderedFloat(f)) => try_char::<_, u32>(f)?,
            ValueKind::F64(OrderedFloat(f)) => try_char::<_, u32>(f)?,
            _ => return Err(serde::de::Error::custom("expected char")),
        };
        visitor.visit_char(v)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.kind {
            ValueKind::String(s) => visitor.visit_string(s),
            ValueKind::StringRef(s) => visitor.visit_borrowed_str(s),
            ValueKind::Bytes(b) => {
                let s = String::from_utf8(b).map_err(serde::de::Error::custom)?;
                visitor.visit_string(s)
            }
            ValueKind::BytesRef(b) => {
                let s = std::str::from_utf8(b).map_err(serde::de::Error::custom)?;
                visitor.visit_borrowed_str(s)
            }
            _ => Err(serde::de::Error::custom("expected string")),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.kind {
            ValueKind::Bytes(b) => visitor.visit_byte_buf(b),
            ValueKind::BytesRef(b) => visitor.visit_borrowed_bytes(b),
            ValueKind::String(s) => visitor.visit_byte_buf(s.into_bytes()),
            ValueKind::StringRef(s) => visitor.visit_borrowed_bytes(s.as_bytes()),
            ValueKind::Array(a) => {
                let count = a.len();
                let mut seq = SeqDeserializer::new(
                    a.into_iter().map(|v| Value::new(v, Cow::Borrowed(self.config.as_ref()))),
                    Some(count),
                );
                let mut bytes = Vec::new();
                while let Some(value) = seq.next_element::<u8>()? {
                    bytes.push(value);
                }
                visitor.visit_byte_buf(bytes)
            }
            _ => Err(serde::de::Error::custom("expected bytes")),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.kind {
            ValueKind::Bytes(b) => visitor.visit_byte_buf(b),
            ValueKind::BytesRef(b) => visitor.visit_borrowed_bytes(b),
            ValueKind::String(s) => visitor.visit_bytes(s.as_bytes()),
            ValueKind::StringRef(s) => visitor.visit_borrowed_bytes(s.as_bytes()),
            ValueKind::Array(a) => {
                let count = a.len();
                let mut seq = SeqDeserializer::new(
                    a.into_iter().map(|v| Value::new(v, Cow::Borrowed(self.config.as_ref()))),
                    Some(count),
                );
                let mut bytes = Vec::new();
                while let Some(value) = seq.next_element::<u8>()? {
                    bytes.push(value);
                }
                visitor.visit_byte_buf(bytes)
            }
            _ => Err(serde::de::Error::custom("expected bytes")),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.kind {
            ValueKind::String(s) => visitor.visit_string(s),
            ValueKind::StringRef(s) => visitor.visit_borrowed_str(s),
            ValueKind::Bytes(b) => {
                let s = String::from_utf8(b).map_err(serde::de::Error::custom)?;
                visitor.visit_string(s)
            }
            ValueKind::BytesRef(b) => {
                let s = std::str::from_utf8(b).map_err(serde::de::Error::custom)?;
                visitor.visit_borrowed_str(s)
            }
            _ => Err(serde::de::Error::custom("expected string")),
        }
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.kind {
            ValueKind::Unit | ValueKind::Null => visitor.visit_unit(),
            _ => Err(serde::de::Error::custom("expected unit")),
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

struct EnumDeserializer<'de, 'b> {
    variant: Value<'de, 'b>,
    value: Option<Value<'de, 'b>>,
}

impl<'de, 'b> de::EnumAccess<'de> for EnumDeserializer<'de, 'b> {
    type Error = ValueError;
    type Variant = VariantDeserializer<'de, 'b>;

    #[inline]
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

struct VariantDeserializer<'de, 'b> {
    value: Option<Value<'de, 'b>>,
}

impl<'de> de::VariantAccess<'de> for VariantDeserializer<'de, '_> {
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

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        let Some(value) = self.value else {
            return Err(serde::de::Error::invalid_type(de::Unexpected::UnitVariant, &"tuple variant"));
        };

        match value.kind {
            ValueKind::Array(v) => {
                if v.is_empty() {
                    visitor.visit_unit()
                } else {
                    let count = v.len();
                    visitor.visit_seq(SeqDeserializer::new(
                        v.into_iter().map(|v| Value::new(v, Cow::Borrowed(value.config.as_ref()))),
                        Some(count),
                    ))
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

        if matches!(value.kind, ValueKind::Map(_)) {
            value.deserialize_any(visitor)
        } else {
            Err(serde::de::Error::invalid_type(value.kind.unexpected(), &"struct variant"))
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

impl<'de, 'b, I> de::SeqAccess<'de> for SeqDeserializer<I>
where
    I: Iterator<Item = Value<'de, 'b>>,
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

struct MapDeserializer<I> {
    iter: I,
    size: Option<usize>,
}

impl<I> MapDeserializer<I> {
    #[inline]
    fn new(values: I, size: Option<usize>) -> Self {
        Self { iter: values, size }
    }
}

impl<'de, 'b, I> de::MapAccess<'de> for MapDeserializer<I>
where
    I: Iterator<Item = Value<'de, 'b>>,
{
    type Error = ValueError;

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        let Some(value) = self.iter.next() else {
            return Ok(None);
        };

        seed.deserialize(value).map(Some)
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let Some(value) = self.iter.next() else {
            return Err(serde::de::Error::invalid_value(de::Unexpected::Map, &"missing map value"));
        };

        seed.deserialize(value)
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.size.or_else(|| self.iter.size_hint().1.map(|s| s / 2))
    }
}

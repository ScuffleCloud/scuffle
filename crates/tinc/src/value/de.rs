use core::fmt;
use std::collections::HashMap;

use ordered_float::OrderedFloat;
use scuffle_bytes_util::{BytesCow, BytesCowVisitor, StringCow, StringCowVisitor};
use serde::de::value::{MapDeserializer, SeqDeserializer};
use serde::de::{self, IntoDeserializer, SeqAccess};
use serde::{Deserialize, Deserializer};

use super::{Key, Value};
use crate::value::ValueOwned;

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
                Value::I8(v) => num_traits::cast::cast(v),
                Value::I16(v) => num_traits::cast::cast(v),
                Value::I32(v) => num_traits::cast::cast(v),
                Value::I64(v) => num_traits::cast::cast(v),
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
                Ok(Value::String(StringCow::from_string(v.to_string())))
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
                Ok(Value::String(StringCow::from_string(v.to_owned())))
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::String(StringCow::from_ref(v)))
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
                let mut map = HashMap::with_capacity(visitor.size_hint().unwrap_or(0));
                while let Some((key, value)) = visitor.next_entry()? {
                    map.insert(key, value);
                }
                Ok(Value::Map(map))
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
                Ok(Value::Bytes(BytesCow::from_vec(v.to_vec())))
            }

            #[inline]
            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Bytes(BytesCow::from_slice(v)))
            }

            #[inline]
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Bytes(BytesCow::from_vec(v)))
            }

            #[inline]
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Value::String(StringCow::from_string(v)))
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

pub struct ValueDeserializer<'de, E> {
    value: Value<'de>,
    _marker: std::marker::PhantomData<E>,
}

impl<'de, E> de::IntoDeserializer<'de, E> for Value<'de>
where
    E: de::Error,
{
    type Deserializer = ValueDeserializer<'de, E>;

    #[inline]
    fn into_deserializer(self) -> Self::Deserializer {
        ValueDeserializer {
            value: self,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'de, E> de::IntoDeserializer<'de, E> for ValueOwned
where
    E: de::Error,
{
    type Deserializer = ValueDeserializer<'de, E>;

    #[inline]
    fn into_deserializer(self) -> Self::Deserializer {
        ValueDeserializer {
            value: self.0,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Implement `Deserializer` for `Value<'de>`.
impl<'de, E> de::Deserializer<'de> for ValueDeserializer<'de, E>
where
    E: de::Error,
{
    type Error = E;

    serde::forward_to_deserialize_any! {
        unit_struct tuple tuple_struct map struct
        identifier ignored_any unit
        char
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
            Value::String(StringCow::Ref(s) | StringCow::StaticRef(s)) => visitor.visit_borrowed_str(s),
            Value::String(StringCow::Bytes(s)) => visitor.visit_str(&s),
            Value::String(StringCow::String(s)) => visitor.visit_string(s),
            Value::Bytes(BytesCow::Slice(b) | BytesCow::StaticSlice(b)) => visitor.visit_borrowed_bytes(b),
            Value::Bytes(BytesCow::Vec(b)) => visitor.visit_byte_buf(b),
            Value::Bytes(BytesCow::Bytes(b)) => visitor.visit_bytes(b.as_ref()),
            Value::U8(v) => visitor.visit_u8(v),
            Value::U16(v) => visitor.visit_u16(v),
            Value::U32(v) => visitor.visit_u32(v),
            Value::U64(v) => visitor.visit_u64(v),
            Value::I8(v) => visitor.visit_i8(v),
            Value::I16(v) => visitor.visit_i16(v),
            Value::I32(v) => visitor.visit_i32(v),
            Value::I64(v) => visitor.visit_i64(v),
            Value::F32(OrderedFloat(v)) => visitor.visit_f32(v),
            Value::F64(OrderedFloat(v)) => visitor.visit_f64(v),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::Null => visitor.visit_none(),
            Value::Array(a) => visitor.visit_seq(SeqDeserializer::new(a.into_iter())),
            Value::Map(m) => visitor.visit_map(MapDeserializer::new(m.into_iter().map(|(k, v)| (k.into_value(), v)))),
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
            Value::Null => visitor.visit_none(),
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
                    _marker: std::marker::PhantomData,
                })
            }
            Value::String(string) => visitor.visit_enum(EnumDeserializer {
                variant: Key::String(string),
                value: None,
                _marker: std::marker::PhantomData,
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
            Value::Bytes(ref b) => b.as_str().map(parse_bool_primitive).ok().flatten(),
            Value::Bool(v) => Some(v),
            Value::U8(v) => Some(v != 0),
            Value::U16(v) => Some(v != 0),
            Value::U32(v) => Some(v != 0),
            Value::U64(v) => Some(v != 0),
            Value::I8(v) => Some(v != 0),
            Value::I16(v) => Some(v != 0),
            Value::I32(v) => Some(v != 0),
            Value::I64(v) => Some(v != 0),
            Value::F32(v) => Some(v != 0.0),
            Value::F64(v) => Some(v != 0.0),
            _ => None,
        };

        if let Some(v) = v {
            visitor.visit_bool(v)
        } else {
            Err(serde::de::Error::invalid_type(self.value.unexpected(), &"bool"))
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::Bytes(bytes) => bytes.handle_visitor(visitor),
            Value::String(string) => string.into_bytes_cow().handle_visitor(visitor),
            Value::Array(a) => {
                let mut seq = SeqDeserializer::new(a.into_iter());
                let mut bytes = Vec::new();
                while let Some(value) = seq.next_element::<u8>()? {
                    bytes.push(value);
                }
                visitor.visit_byte_buf(bytes)
            }
            _ => Err(serde::de::Error::invalid_type(self.value.unexpected(), &"bytes")),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Value::String(string) => string.handle_visitor(visitor),
            Value::Bytes(bytes) => bytes
                .into_string_cow()
                .map(|string| string.handle_visitor(visitor))
                .map_err(|e| serde::de::Error::invalid_type(de::Unexpected::Bytes(&e), &"string"))?,
            Value::Bool(b) => visitor.visit_borrowed_str(if b { "true" } else { "false" }),
            Value::U8(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::U16(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::U32(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::U64(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::I8(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::I16(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::I32(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::I64(v) => visitor.visit_str(itoa::Buffer::new().format(v)),
            Value::F32(OrderedFloat(v)) => visitor.visit_str(ryu::Buffer::new().format(v)),
            Value::F64(OrderedFloat(v)) => visitor.visit_str(ryu::Buffer::new().format(v)),
            _ => Err(serde::de::Error::invalid_type(self.value.unexpected(), &"string")),
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

    fn deserialize_newtype_struct<V>(mut self, name: &'static str, mut visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match (name, self.value.take()) {
            (BytesCowVisitor::NEW_TYPE_NAME, Value::Bytes(bytes)) => unsafe {
                let visitor = &mut *(&raw mut visitor as *mut BytesCowVisitor<'de>);
                visitor.set(bytes);
            },
            (StringCowVisitor::NEW_TYPE_NAME, Value::String(string)) => unsafe {
                let visitor = &mut *(&raw mut visitor as *mut StringCowVisitor<'de>);
                visitor.set(string);
            },
            (_, value) => self.value = value,
        }

        visitor.visit_newtype_struct(self)
    }
}

/// Helper deserializer for enums.
struct EnumDeserializer<'de, E> {
    variant: Key<'de>,
    value: Option<Value<'de>>,
    _marker: std::marker::PhantomData<E>,
}

impl<'de, E> de::EnumAccess<'de> for EnumDeserializer<'de, E>
where
    E: de::Error,
{
    type Error = E;
    type Variant = VariantDeserializer<'de, E>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.variant.into_value().into_deserializer()).map(|v| {
            (
                v,
                VariantDeserializer {
                    value: self.value,
                    _marker: std::marker::PhantomData,
                },
            )
        })
    }
}

impl<'de> de::Deserialize<'de> for Key<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeyVisitor;

        macro_rules! impl_visit {
            ($($fn:ident $var:ident $ty:ty),*$(,)?) => {
                $(
                    fn $fn<E>(self, v: $ty) -> Result<Self::Value, E>
                        where
                            E: de::Error, {
                        Ok(Key::$var(v.into()))
                    }
                )*
            };
        }

        impl<'de> de::Visitor<'de> for KeyVisitor {
            type Value = Key<'de>;

            impl_visit!(
                visit_bool Bool bool,
                visit_borrowed_str String &'de str,
                visit_string String String,
                visit_i8 I8 i8,
                visit_i16 I16 i16,
                visit_i32 I32 i32,
                visit_i64 I64 i64,
                visit_u8 U8 u8,
                visit_u16 U16 u16,
                visit_u32 U32 u32,
                visit_u64 U64 u64,
            );

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a key")
            }

            fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Key::String(v.to_string().into()))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Key::String(v.to_owned().into()))
            }
        }

        deserializer.deserialize_any(KeyVisitor)
    }
}

/// Helper for deserializing enum variants.
struct VariantDeserializer<'de, E> {
    value: Option<Value<'de>>,
    _marker: std::marker::PhantomData<E>,
}

impl<'de, E> de::VariantAccess<'de> for VariantDeserializer<'de, E>
where
    E: de::Error,
{
    type Error = E;

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

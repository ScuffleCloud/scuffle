use std::borrow::Cow;
use std::collections::HashMap;

use ordered_float::OrderedFloat;
use scuffle_bytes_util::{BytesCow, StringCow};

pub mod de;
pub mod ser;

#[derive(Debug)]
#[repr(transparent)]
pub struct ValueOwned(pub Value<'static>);

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Key<'a> {
    String(StringCow<'a>),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Bool(bool),
}

impl<'a> Key<'a> {
    pub fn into_static(self) -> Key<'static> {
        match self {
            Self::String(s) => Key::String(s.to_owned()),
            Self::U8(u) => Key::U8(u),
            Self::U16(u) => Key::U16(u),
            Self::U32(u) => Key::U32(u),
            Self::U64(u) => Key::U64(u),
            Self::I8(i) => Key::I8(i),
            Self::I16(i) => Key::I16(i),
            Self::I32(i) => Key::I32(i),
            Self::I64(i) => Key::I64(i),
            Self::Bool(b) => Key::Bool(b),
        }
    }

    pub fn into_value(self) -> Value<'a> {
        match self {
            Self::String(s) => Value::String(s),
            Self::U8(u) => Value::U8(u),
            Self::U16(u) => Value::U16(u),
            Self::U32(u) => Value::U32(u),
            Self::U64(u) => Value::U64(u),
            Self::I8(i) => Value::I8(i),
            Self::I16(i) => Value::I16(i),
            Self::I32(i) => Value::I32(i),
            Self::I64(i) => Value::I64(i),
            Self::Bool(b) => Value::Bool(b),
        }
    }
}

#[derive(Debug, Default)]
pub enum Value<'a> {
    String(StringCow<'a>),
    Bytes(BytesCow<'a>),
    F64(OrderedFloat<f64>),
    F32(OrderedFloat<f32>),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Bool(bool),
    #[default]
    Null,
    Array(Vec<Value<'a>>),
    Map(HashMap<Key<'a>, Value<'a>>),
}

impl Value<'_> {
    pub fn into_static(self) -> Value<'static> {
        match self {
            Self::String(s) => Value::String(s.to_owned()),
            Self::Bytes(b) => Value::Bytes(b.to_owned()),
            Self::F64(f) => Value::F64(f),
            Self::F32(f) => Value::F32(f),
            Self::U8(u) => Value::U8(u),
            Self::U16(u) => Value::U16(u),
            Self::U32(u) => Value::U32(u),
            Self::U64(u) => Value::U64(u),
            Self::I8(i) => Value::I8(i),
            Self::I16(i) => Value::I16(i),
            Self::I32(i) => Value::I32(i),
            Self::I64(i) => Value::I64(i),
            Self::Bool(b) => Value::Bool(b),
            Self::Null => Value::Null,
            Self::Array(a) => Value::Array(a.into_iter().map(|v| v.into_static()).collect()),
            Self::Map(m) => Value::Map(m.into_iter().map(|(k, v)| (k.into_static(), v.into_static())).collect()),
        }
    }

    pub fn into_owned(self) -> ValueOwned {
        ValueOwned(self.into_static())
    }

    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}

macro_rules! impl_from_primitive {
    ($variant:ident, $ty:ty) => {
        impl From<$ty> for Value<'static> {
            #[inline]
            fn from(value: $ty) -> Self {
                Self::$variant(value.into())
            }
        }
    };
}

impl_from_primitive!(U8, u8);
impl_from_primitive!(U16, u16);
impl_from_primitive!(U32, u32);
impl_from_primitive!(U64, u64);
impl_from_primitive!(I8, i8);
impl_from_primitive!(I16, i16);
impl_from_primitive!(I32, i32);
impl_from_primitive!(I64, i64);
impl_from_primitive!(F32, f32);
impl_from_primitive!(F64, f64);
impl_from_primitive!(Bool, bool);
impl_from_primitive!(String, String);
impl_from_primitive!(Bytes, Vec<u8>);

impl<'a> From<&'a str> for Value<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        Value::String(StringCow::from_ref(value))
    }
}

impl<'a> From<&'a [u8]> for Value<'a> {
    #[inline]
    fn from(value: &'a [u8]) -> Self {
        Value::Bytes(BytesCow::from_slice(value))
    }
}

impl<'a> From<Vec<Value<'a>>> for Value<'a> {
    #[inline]
    fn from(value: Vec<Value<'a>>) -> Self {
        Value::Array(value)
    }
}

impl<'a> From<HashMap<Key<'a>, Value<'a>>> for Value<'a> {
    #[inline]
    fn from(value: HashMap<Key<'a>, Value<'a>>) -> Self {
        Value::Map(value)
    }
}

impl<'a> From<Cow<'a, str>> for Value<'a> {
    #[inline]
    fn from(value: Cow<'a, str>) -> Self {
        Value::String(StringCow::from_cow(value))
    }
}

impl From<ValueOwned> for Value<'static> {
    #[inline]
    fn from(value: ValueOwned) -> Self {
        value.0
    }
}

impl From<Value<'_>> for ValueOwned {
    #[inline]
    fn from(value: Value<'_>) -> Self {
        value.into_owned()
    }
}

impl Value<'_> {
    fn unexpected(&self) -> serde::de::Unexpected<'_> {
        match self {
            Value::String(s) => serde::de::Unexpected::Str(s.as_ref()),
            Value::Bytes(b) => serde::de::Unexpected::Bytes(b.as_ref()),
            Value::Array(_) => serde::de::Unexpected::Seq,
            Value::Map(_) => serde::de::Unexpected::Map,
            Value::Null => serde::de::Unexpected::Option,
            Value::F64(OrderedFloat(f)) => serde::de::Unexpected::Float(*f as _),
            Value::F32(OrderedFloat(f)) => serde::de::Unexpected::Float(*f as _),
            Value::U8(u) => serde::de::Unexpected::Unsigned(*u as _),
            Value::U16(u) => serde::de::Unexpected::Unsigned(*u as _),
            Value::U32(u) => serde::de::Unexpected::Unsigned(*u as _),
            Value::U64(u) => serde::de::Unexpected::Unsigned(*u as _),
            Value::I8(i) => serde::de::Unexpected::Signed(*i as _),
            Value::I16(i) => serde::de::Unexpected::Signed(*i as _),
            Value::I32(i) => serde::de::Unexpected::Signed(*i as _),
            Value::I64(i) => serde::de::Unexpected::Signed(*i as _),
            Value::Bool(b) => serde::de::Unexpected::Bool(*b),
        }
    }
}

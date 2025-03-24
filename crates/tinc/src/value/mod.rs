use std::borrow::Cow;

use ordered_float::OrderedFloat;

pub mod de;
pub mod ser;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ValuePrimitive {
    F64(OrderedFloat<f64>),
    F32(OrderedFloat<f32>),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Bool(bool),
    Char(char),
    Null,
    Unit,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Value<'de> {
    String(Cow<'de, str>),
    Bytes(Cow<'de, [u8]>),
    Primitive(ValuePrimitive),
    Array(Vec<Value<'de>>),
    Map(Map<Value<'de>>),
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ValueOwned {
    String(String),
    Bytes(Vec<u8>),
    Primitive(ValuePrimitive),
    Array(Vec<ValueOwned>),
    Map(Map<ValueOwned>),
}

impl ValueOwned {
    pub fn into_value(self) -> Value<'static> {
        match self {
            Self::String(s) => Value::String(Cow::Owned(s)),
            Self::Bytes(b) => Value::Bytes(Cow::Owned(b)),
            Self::Primitive(p) => Value::Primitive(p),
            Self::Array(a) => Value::Array(a.into_iter().map(|v| v.into_value()).collect()),
            Self::Map(m) => Value::Map(m.into_iter().map(|(k, v)| (k.into_value(), v.into_value())).collect()),
        }
    }
}

impl<'de> Value<'de> {
    pub fn into_owned(self) -> ValueOwned {
        match self {
            Self::String(s) => ValueOwned::String(s.into_owned()),
            Self::Bytes(b) => ValueOwned::Bytes(b.into_owned()),
            Self::Primitive(p) => ValueOwned::Primitive(p),
            Self::Array(a) => ValueOwned::Array(a.into_iter().map(|v| v.into_owned()).collect()),
            Self::Map(m) => ValueOwned::Map(m.into_iter().map(|(k, v)| (k.into_owned(), v.into_owned())).collect()),
        }
    }
}

macro_rules! impl_from_primitive {
    ($variant:ident, $ty:ty) => {
        impl From<$ty> for ValuePrimitive {
            #[inline]
            fn from(value: $ty) -> Self {
                Self::$variant(value.into())
            }
        }

        impl From<$ty> for ValueOwned {
            #[inline]
            fn from(value: $ty) -> Self {
                Self::Primitive(value.into())
            }
        }

        impl From<$ty> for Value<'static> {
            #[inline]
            fn from(value: $ty) -> Self {
                Self::Primitive(value.into())
            }
        }
    };
}

impl_from_primitive!(U8, u8);
impl_from_primitive!(U16, u16);
impl_from_primitive!(U32, u32);
impl_from_primitive!(U64, u64);
impl_from_primitive!(U128, u128);
impl_from_primitive!(I8, i8);
impl_from_primitive!(I16, i16);
impl_from_primitive!(I32, i32);
impl_from_primitive!(I64, i64);
impl_from_primitive!(I128, i128);
impl_from_primitive!(F32, f32);
impl_from_primitive!(F64, f64);
impl_from_primitive!(Bool, bool);
impl_from_primitive!(Char, char);

impl From<String> for ValueOwned {
    #[inline]
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<Vec<u8>> for ValueOwned {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(value)
    }
}

impl From<String> for Value<'static> {
    #[inline]
    fn from(value: String) -> Self {
        Self::String(Cow::Owned(value))
    }
}

impl From<Vec<u8>> for Value<'static> {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        Self::Bytes(Cow::Owned(value))
    }
}

impl<'de> From<&'de str> for Value<'de> {
    #[inline]
    fn from(value: &'de str) -> Self {
        Self::String(Cow::Borrowed(value))
    }
}

impl<'de> From<&'de [u8]> for Value<'de> {
    #[inline]
    fn from(value: &'de [u8]) -> Self {
        Self::Bytes(Cow::Borrowed(value))
    }
}

impl<'de> From<Vec<Value<'de>>> for Value<'de> {
    #[inline]
    fn from(value: Vec<Value<'de>>) -> Self {
        Self::Array(value)
    }
}

impl<'de> From<Map<Value<'de>>> for Value<'de> {
    #[inline]
    fn from(value: Map<Value<'de>>) -> Self {
        Self::Map(value)
    }
}

impl<'de> From<Object<'de>> for Value<'de> {
    #[inline]
    fn from(value: Object<'de>) -> Self {
        Self::Map(value.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
}

impl<'de> From<Cow<'de, str>> for Value<'de> {
    #[inline]
    fn from(value: Cow<'de, str>) -> Self {
        Self::String(value)
    }
}

impl From<()> for Value<'_> {
    #[inline]
    fn from(_: ()) -> Self {
        Self::Primitive(ValuePrimitive::Unit)
    }
}

impl Value<'_> {
    fn unexpected(&self) -> serde::de::Unexpected<'_> {
        match self {
            Value::String(s) => serde::de::Unexpected::Str(s),
            Value::Bytes(b) => serde::de::Unexpected::Bytes(b),
            Value::Array(_) => serde::de::Unexpected::Seq,
            Value::Map(_) => serde::de::Unexpected::Map,
            Value::Primitive(p) => p.unexpected(),
        }
    }
}

impl ValuePrimitive {
    fn unexpected(&self) -> serde::de::Unexpected<'_> {
        match self {
            ValuePrimitive::Null => serde::de::Unexpected::Option,
            ValuePrimitive::Unit => serde::de::Unexpected::Unit,
            ValuePrimitive::F64(OrderedFloat(f)) => serde::de::Unexpected::Float(*f as _),
            ValuePrimitive::F32(OrderedFloat(f)) => serde::de::Unexpected::Float(*f as _),
            ValuePrimitive::U8(u) => serde::de::Unexpected::Unsigned(*u as _),
            ValuePrimitive::U16(u) => serde::de::Unexpected::Unsigned(*u as _),
            ValuePrimitive::U32(u) => serde::de::Unexpected::Unsigned(*u as _),
            ValuePrimitive::U64(u) => serde::de::Unexpected::Unsigned(*u as _),
            ValuePrimitive::U128(u) => serde::de::Unexpected::Unsigned(*u as _),
            ValuePrimitive::I8(i) => serde::de::Unexpected::Signed(*i as _),
            ValuePrimitive::I16(i) => serde::de::Unexpected::Signed(*i as _),
            ValuePrimitive::I32(i) => serde::de::Unexpected::Signed(*i as _),
            ValuePrimitive::I64(i) => serde::de::Unexpected::Signed(*i as _),
            ValuePrimitive::I128(i) => serde::de::Unexpected::Signed(*i as _),
            ValuePrimitive::Bool(b) => serde::de::Unexpected::Bool(*b),
            ValuePrimitive::Char(c) => serde::de::Unexpected::Char(*c),
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum ValueError {
    #[error("{0}")]
    Custom(String),
}

impl serde::de::Error for ValueError {
    #[inline]
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Map<K, V = K>(pub Vec<(K, V)>);

impl<K, V> Default for Map<K, V> {
    #[inline]
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<K, V> Map<K, V> {
    #[inline]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline]
    pub fn push(&mut self, key: K, value: V) {
        self.0.push((key, value));
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, (K, V)> {
        self.0.iter()
    }
}

impl<K, V> IntoIterator for Map<K, V> {
    type IntoIter = std::vec::IntoIter<(K, V)>;
    type Item = (K, V);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a Map<K, V> {
    type IntoIter = std::slice::Iter<'a, (K, V)>;
    type Item = &'a (K, V);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<K, V> Extend<(K, V)> for Map<K, V> {
    #[inline]
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

impl<K, V> FromIterator<(K, V)> for Map<K, V> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[derive(Debug)]
pub struct Object<'de>(pub Map<Cow<'de, str>, Value<'de>>);

impl<'de> IntoIterator for Object<'de> {
    type IntoIter = std::vec::IntoIter<(Cow<'de, str>, Value<'de>)>;
    type Item = (Cow<'de, str>, Value<'de>);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'de> IntoIterator for &'de Object<'de> {
    type IntoIter = std::slice::Iter<'de, (Cow<'de, str>, Value<'de>)>;
    type Item = &'de (Cow<'de, str>, Value<'de>);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Default for Object<'_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'de> Object<'de> {
    #[inline]
    pub fn new() -> Self {
        Self(Map::new())
    }

    #[inline]
    pub fn merge(&mut self, iter: impl IntoIterator<Item = (Cow<'de, str>, Value<'de>)>) {
        self.0.extend(iter);
    }
}

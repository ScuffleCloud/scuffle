use std::borrow::Cow;
use std::collections::BTreeMap;

use bytes::Bytes;
use ordered_float::OrderedFloat;

pub mod de;
pub mod ser;

#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ValueOwned(pub Value<'static>);

impl std::ops::Deref for ValueOwned {
    type Target = Value<'static>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ValueOwned {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ValueOwned {
    #[inline]
    pub fn into_inner(self) -> Value<'static> {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Value<'a> {
    String(Cow<'a, str>),
    Bytes(Cow<'a, [u8]>),
    BytesOwned(Bytes),
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
    Array(Vec<Value<'a>>),
    Map(Map<Value<'a>>),
    Object(Object<'a>),
}

impl Value<'_> {
    pub fn into_static(self) -> Value<'static> {
        match self {
            Self::String(s) => Value::String(Cow::Owned(s.into_owned())),
            Self::Bytes(b) => Value::Bytes(Cow::Owned(b.into_owned())),
            Self::BytesOwned(b) => Value::BytesOwned(b),
            Self::F64(f) => Value::F64(f),
            Self::F32(f) => Value::F32(f),
            Self::U8(u) => Value::U8(u),
            Self::U16(u) => Value::U16(u),
            Self::U32(u) => Value::U32(u),
            Self::U64(u) => Value::U64(u),
            Self::U128(u) => Value::U128(u),
            Self::I8(i) => Value::I8(i),
            Self::I16(i) => Value::I16(i),
            Self::I32(i) => Value::I32(i),
            Self::I64(i) => Value::I64(i),
            Self::I128(i) => Value::I128(i),
            Self::Bool(b) => Value::Bool(b),
            Self::Char(c) => Value::Char(c),
            Self::Null => Value::Null,
            Self::Unit => Value::Unit,
            Self::Array(a) => Value::Array(a.into_iter().map(|v| v.into_static()).collect()),
            Self::Map(m) => Value::Map(m.into_iter().map(|(k, v)| (k.into_static(), v.into_static())).collect()),
            Self::Object(o) => Value::Object(o.into_static()),
        }
    }

    pub fn into_owned(self) -> ValueOwned {
        ValueOwned(self.into_static())
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
impl_from_primitive!(String, String);
impl_from_primitive!(Bytes, Vec<u8>);

impl<'a> From<&'a str> for Value<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        Value::String(Cow::Borrowed(value))
    }
}

impl<'a> From<&'a [u8]> for Value<'a> {
    #[inline]
    fn from(value: &'a [u8]) -> Self {
        Value::Bytes(Cow::Borrowed(value))
    }
}

impl<'a> From<Vec<Value<'a>>> for Value<'a> {
    #[inline]
    fn from(value: Vec<Value<'a>>) -> Self {
        Value::Array(value)
    }
}

impl<'a> From<Map<Value<'a>>> for Value<'a> {
    #[inline]
    fn from(value: Map<Value<'a>>) -> Self {
        Value::Map(value)
    }
}

impl<'a> From<Object<'a>> for Value<'a> {
    #[inline]
    fn from(value: Object<'a>) -> Self {
        Value::Map(value.0.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
}

impl From<ObjectOwned> for Value<'static> {
    #[inline]
    fn from(value: ObjectOwned) -> Self {
        Value::Map(value.0.0.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
}

impl<'a> From<Cow<'a, str>> for Value<'a> {
    #[inline]
    fn from(value: Cow<'a, str>) -> Self {
        Value::String(value)
    }
}

impl From<ValueOwned> for Value<'static> {
    #[inline]
    fn from(value: ValueOwned) -> Self {
        value.into_inner()
    }
}

impl From<Value<'_>> for ValueOwned {
    #[inline]
    fn from(value: Value<'_>) -> Self {
        value.into_owned()
    }
}

impl From<()> for Value<'_> {
    #[inline]
    fn from(_: ()) -> Self {
        Self::Unit
    }
}

impl Value<'_> {
    fn unexpected(&self) -> serde::de::Unexpected<'_> {
        match self {
            Value::String(s) => serde::de::Unexpected::Str(s),
            Value::Bytes(b) => serde::de::Unexpected::Bytes(b),
            Value::BytesOwned(b) => serde::de::Unexpected::Bytes(b),
            Value::Array(_) => serde::de::Unexpected::Seq,
            Value::Map(_) | Value::Object(_) => serde::de::Unexpected::Map,
            Value::Unit => serde::de::Unexpected::Unit,
            Value::Null => serde::de::Unexpected::Option,
            Value::F64(OrderedFloat(f)) => serde::de::Unexpected::Float(*f as _),
            Value::F32(OrderedFloat(f)) => serde::de::Unexpected::Float(*f as _),
            Value::U8(u) => serde::de::Unexpected::Unsigned(*u as _),
            Value::U16(u) => serde::de::Unexpected::Unsigned(*u as _),
            Value::U32(u) => serde::de::Unexpected::Unsigned(*u as _),
            Value::U64(u) => serde::de::Unexpected::Unsigned(*u as _),
            Value::U128(u) => serde::de::Unexpected::Unsigned(*u as _),
            Value::I8(i) => serde::de::Unexpected::Signed(*i as _),
            Value::I16(i) => serde::de::Unexpected::Signed(*i as _),
            Value::I32(i) => serde::de::Unexpected::Signed(*i as _),
            Value::I64(i) => serde::de::Unexpected::Signed(*i as _),
            Value::I128(i) => serde::de::Unexpected::Signed(*i as _),
            Value::Bool(b) => serde::de::Unexpected::Bool(*b),
            Value::Char(c) => serde::de::Unexpected::Char(*c),
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
        ValueError::Custom(msg.to_string())
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
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

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    #[inline]
    pub fn with_entries(entries: Vec<(K, V)>) -> Self {
        Self(entries)
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
        let iter = iter.into_iter();
        if let (_, Some(upper)) = iter.size_hint() {
            self.reserve(upper);
        }
        self.0.extend(iter);
    }
}

impl<K, V> FromIterator<(K, V)> for Map<K, V> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Object<'ir>(pub BTreeMap<Cow<'ir, str>, Value<'ir>>);

#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ObjectOwned(pub Object<'static>);

impl<'ir> Object<'ir> {
    #[inline]
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    #[inline]
    pub fn merge(&mut self, iter: impl IntoIterator<Item = (Cow<'ir, str>, Value<'ir>)>) {
        self.0.extend(iter);
    }

    #[inline]
    pub fn insert(&mut self, key: Cow<'ir, str>, value: Value<'ir>) {
        self.0.insert(key, value);
    }

    #[inline]
    pub fn into_owned(self) -> ObjectOwned {
        ObjectOwned(self.into_static())
    }

    pub fn into_static(self) -> Object<'static> {
        Object(
            self.0
                .into_iter()
                .map(|(k, v)| (Cow::Owned(k.into_owned()), v.into_static()))
                .collect(),
        )
    }

    pub fn entry(&mut self, key: Cow<'ir, str>) -> std::collections::btree_map::Entry<'_, Cow<'ir, str>, Value<'ir>> {
        self.0.entry(key)
    }

    pub fn iter(&self) -> std::collections::btree_map::Iter<'_, Cow<'ir, str>, Value<'ir>> {
        self.0.iter()
    }
}

impl Default for Object<'_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'ir> IntoIterator for Object<'ir> {
    type IntoIter = std::collections::btree_map::IntoIter<Cow<'ir, str>, Value<'ir>>;
    type Item = (Cow<'ir, str>, Value<'ir>);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, 'ir> IntoIterator for &'a Object<'ir> {
    type IntoIter = std::collections::btree_map::Iter<'a, Cow<'ir, str>, Value<'ir>>;
    type Item = (&'a Cow<'ir, str>, &'a Value<'ir>);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl ObjectOwned {
    #[inline]
    pub fn into_inner(self) -> Object<'static> {
        self.0
    }
}

impl std::ops::Deref for ObjectOwned {
    type Target = Object<'static>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ObjectOwned {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for ObjectOwned {
    type IntoIter = <Object<'static> as IntoIterator>::IntoIter;
    type Item = <Object<'static> as IntoIterator>::Item;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a ObjectOwned {
    type IntoIter = <&'a Object<'static> as IntoIterator>::IntoIter;
    type Item = <&'a Object<'static> as IntoIterator>::Item;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

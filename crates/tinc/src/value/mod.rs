use std::borrow::Cow;

use ordered_float::OrderedFloat;

pub mod de;
pub mod ser;

#[derive(Debug)]
pub enum ValueKind<'de> {
    String(String),
    StringRef(&'de str),
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
    Array(Vec<ValueKind<'de>>),
    Map(Map<ValueKind<'de>>),
    Bytes(Vec<u8>),
    BytesRef(&'de [u8]),
    Null,
    Unit,
}

impl ValueKind<'_> {
    fn unexpected(&self) -> serde::de::Unexpected<'_> {
        match self {
            ValueKind::String(s) => serde::de::Unexpected::Str(s),
            ValueKind::StringRef(s) => serde::de::Unexpected::Str(s),
            ValueKind::Bytes(b) => serde::de::Unexpected::Bytes(b),
            ValueKind::BytesRef(b) => serde::de::Unexpected::Bytes(b),
            ValueKind::Array(_) => serde::de::Unexpected::Seq,
            ValueKind::Map(_) => serde::de::Unexpected::Map,
            ValueKind::Null => serde::de::Unexpected::Option,
            ValueKind::Unit => serde::de::Unexpected::Unit,
            ValueKind::F32(OrderedFloat(f)) => serde::de::Unexpected::Float(*f as _),
            ValueKind::F64(OrderedFloat(f)) => serde::de::Unexpected::Float(*f),
            ValueKind::U8(u) => serde::de::Unexpected::Unsigned(*u as _),
            ValueKind::U16(u) => serde::de::Unexpected::Unsigned(*u as _),
            ValueKind::U32(u) => serde::de::Unexpected::Unsigned(*u as _),
            ValueKind::U64(u) => serde::de::Unexpected::Unsigned(*u as _),
            ValueKind::U128(u) => serde::de::Unexpected::Unsigned(*u as _),
            ValueKind::I8(i) => serde::de::Unexpected::Signed(*i as _),
            ValueKind::I16(i) => serde::de::Unexpected::Signed(*i as _),
            ValueKind::I32(i) => serde::de::Unexpected::Signed(*i as _),
            ValueKind::I64(i) => serde::de::Unexpected::Signed(*i as _),
            ValueKind::I128(i) => serde::de::Unexpected::Signed(*i as _),
            ValueKind::Bool(b) => serde::de::Unexpected::Bool(*b),
            ValueKind::Char(c) => serde::de::Unexpected::Char(*c),
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

#[derive(Debug)]
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
pub struct Object<'de, 'b>(pub Map<Cow<'de, str>, Value<'de, 'b>>);

#[derive(Debug)]
pub struct PartialObject<'de>(pub Map<Cow<'de, str>, ValueKind<'de>>);

impl Default for PartialObject<'_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'de, 'b> PartialObject<'de> {
    #[inline]
    pub fn new() -> Self {
        Self(Map::new())
    }

    #[inline]
    pub fn into_object(self, config: &'b ValueConfig) -> Object<'de, 'b> {
        Object(
            self.0
                .into_iter()
                .map(|(k, v)| (k, Value::new(v, Cow::Borrowed(config))))
                .collect(),
        )
    }
}

impl<'de> serde::Deserialize<'de> for PartialObject<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self(map))
    }
}

impl<'de> IntoIterator for PartialObject<'de> {
    type IntoIter = std::vec::IntoIter<(Cow<'de, str>, ValueKind<'de>)>;
    type Item = (Cow<'de, str>, ValueKind<'de>);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'de> IntoIterator for &'de PartialObject<'de> {
    type IntoIter = std::slice::Iter<'de, (Cow<'de, str>, ValueKind<'de>)>;
    type Item = &'de (Cow<'de, str>, ValueKind<'de>);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'de, 'b> IntoIterator for Object<'de, 'b> {
    type IntoIter = std::vec::IntoIter<(Cow<'de, str>, Value<'de, 'b>)>;
    type Item = (Cow<'de, str>, Value<'de, 'b>);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'de, 'b> IntoIterator for &'de Object<'de, 'b> {
    type IntoIter = std::slice::Iter<'de, (Cow<'de, str>, Value<'de, 'b>)>;
    type Item = &'de (Cow<'de, str>, Value<'de, 'b>);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Default for Object<'_, '_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'de, 'b> Object<'de, 'b> {
    #[inline]
    pub fn new() -> Self {
        Self(Map::new())
    }

    #[inline]
    pub fn extend(&mut self, config: &'b ValueConfig, iter: impl IntoIterator<Item = (Cow<'de, str>, ValueKind<'de>)>) {
        self.merge(iter.into_iter().map(|(k, v)| (k, Value::new(v, Cow::Borrowed(config)))));
    }

    #[inline]
    pub fn merge(&mut self, iter: impl IntoIterator<Item = (Cow<'de, str>, Value<'de, 'b>)>) {
        self.0.extend(iter);
    }
}

impl<'de, 'b> std::ops::Deref for Object<'de, 'b> {
    type Target = Map<Cow<'de, str>, Value<'de, 'b>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Object<'_, '_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct Value<'de, 'b> {
    pub kind: ValueKind<'de>,
    pub config: Cow<'b, ValueConfig>,
}

impl<'de, 'b> Value<'de, 'b> {
    #[inline]
    fn new(kind: ValueKind<'de>, config: impl Into<Cow<'b, ValueConfig>>) -> Self {
        Self {
            kind,
            config: config.into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ValueConfig {
    pub parse_string_primitive: bool,
}

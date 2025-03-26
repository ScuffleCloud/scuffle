use ordered_float::OrderedFloat;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use super::{Map, Object, ObjectOwned, Value, ValueOwned};

impl Serialize for Value<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::String(s) => serializer.serialize_str(s),
            Self::Map(m) => m.serialize(serializer),
            Self::Array(a) => a.serialize(serializer),
            Self::I64(v) => serializer.serialize_i64(*v),
            Self::I32(v) => serializer.serialize_i32(*v),
            Self::F64(OrderedFloat(f)) => serializer.serialize_f64(*f),
            Self::F32(OrderedFloat(f)) => serializer.serialize_f32(*f),
            Self::Bool(v) => serializer.serialize_bool(*v),
            Self::Bytes(b) => serializer.serialize_bytes(b),
            Self::BytesOwned(b) => serializer.serialize_bytes(b),
            Self::U8(v) => serializer.serialize_u8(*v),
            Self::U16(v) => serializer.serialize_u16(*v),
            Self::U32(v) => serializer.serialize_u32(*v),
            Self::U64(v) => serializer.serialize_u64(*v),
            Self::U128(v) => serializer.serialize_u128(*v),
            Self::I8(v) => serializer.serialize_i8(*v),
            Self::I16(v) => serializer.serialize_i16(*v),
            Self::I128(v) => serializer.serialize_i128(*v),
            Self::Char(c) => serializer.serialize_char(*c),
            Self::Null => serializer.serialize_none(),
            Self::Unit => serializer.serialize_unit(),
            Self::Object(o) => o.serialize(serializer),
        }
    }
}

impl Serialize for ValueOwned {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<K, V> Serialize for Map<K, V>
where
    K: Serialize,
    V: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl Serialize for Object<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl Serialize for ObjectOwned {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

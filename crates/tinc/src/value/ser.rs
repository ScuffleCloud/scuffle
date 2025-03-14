use serde::ser::SerializeMap;

use super::{Map, Object, PartialObject, Value, ValueKind};

impl<K, V> serde::Serialize for Map<K, V>
where
    K: serde::Serialize,
    V: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (key, value) in self {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

impl serde::ser::Serialize for ValueKind<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            ValueKind::String(s) => serializer.serialize_str(s),
            ValueKind::StringRef(s) => serializer.serialize_str(s),
            ValueKind::F64(f) => serializer.serialize_f64(f.into_inner()),
            ValueKind::F32(f) => serializer.serialize_f32(f.into_inner()),
            ValueKind::U8(u) => serializer.serialize_u8(*u),
            ValueKind::U16(u) => serializer.serialize_u16(*u),
            ValueKind::U32(u) => serializer.serialize_u32(*u),
            ValueKind::U64(u) => serializer.serialize_u64(*u),
            ValueKind::U128(u) => serializer.serialize_u128(*u),
            ValueKind::I8(i) => serializer.serialize_i8(*i),
            ValueKind::I16(i) => serializer.serialize_i16(*i),
            ValueKind::I32(i) => serializer.serialize_i32(*i),
            ValueKind::I64(i) => serializer.serialize_i64(*i),
            ValueKind::I128(i) => serializer.serialize_i128(*i),
            ValueKind::Bool(b) => serializer.serialize_bool(*b),
            ValueKind::Char(c) => serializer.serialize_char(*c),
            ValueKind::Array(a) => a.serialize(serializer),
            ValueKind::Map(m) => m.serialize(serializer),
            ValueKind::Bytes(b) => serializer.serialize_bytes(b),
            ValueKind::BytesRef(b) => serializer.serialize_bytes(b),
            ValueKind::Null => serializer.serialize_none(),
            ValueKind::Unit => serializer.serialize_unit(),
        }
    }
}

impl serde::ser::Serialize for Object<'_, '_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl serde::ser::Serialize for Value<'_, '_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.kind.serialize(serializer)
    }
}

impl serde::ser::Serialize for PartialObject<'_> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.0.serialize(serializer)
    }
}

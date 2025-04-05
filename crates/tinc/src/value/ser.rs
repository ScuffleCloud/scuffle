use ordered_float::OrderedFloat;
use serde::{Serialize, Serializer};

use super::{Key, Value, ValueOwned};

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
            Self::Bytes(b) => b.serialize(serializer),
            Self::U8(v) => serializer.serialize_u8(*v),
            Self::U16(v) => serializer.serialize_u16(*v),
            Self::U32(v) => serializer.serialize_u32(*v),
            Self::U64(v) => serializer.serialize_u64(*v),
            Self::I8(v) => serializer.serialize_i8(*v),
            Self::I16(v) => serializer.serialize_i16(*v),
            Self::Null => serializer.serialize_none(),
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

impl Serialize for Key<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::String(s) => s.serialize(serializer),
            Self::U8(v) => serializer.serialize_u8(*v),
            Self::U16(v) => serializer.serialize_u16(*v),
            Self::U32(v) => serializer.serialize_u32(*v),
            Self::U64(v) => serializer.serialize_u64(*v),
            Self::I8(v) => serializer.serialize_i8(*v),
            Self::I16(v) => serializer.serialize_i16(*v),
            Self::I32(v) => serializer.serialize_i32(*v),
            Self::I64(v) => serializer.serialize_i64(*v),
            Self::Bool(v) => serializer.serialize_bool(*v),
        }
    }
}

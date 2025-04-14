use std::borrow::Cow;
use std::collections::HashMap;

use serde::de::IntoDeserializer;

#[derive(Debug, Clone, PartialEq)]
pub enum Value<'de> {
    /// primitive types for `bool`: `false`/`true`
    Bool(bool),
    /// primitive types for `i8`
    I8(i8),
    /// primitive types for `i16`
    I16(i16),
    /// primitive types for `i32`
    I32(i32),
    /// primitive types for `i64`
    I64(i64),
    /// primitive types for `i128`
    I128(i128),
    /// primitive types for `u8`
    U8(u8),
    /// primitive types for `u16`
    U16(u16),
    /// primitive types for `u32`
    U32(u32),
    /// primitive types for `u64`
    U64(u64),
    /// primitive types for `u128`
    U128(u128),
    /// primitive types for `f32`
    F32(f32),
    /// primitive types for `f64`
    F64(f64),
    /// primitive types for `char`
    Char(char),
    /// string type
    ///
    /// UTF-8 bytes with a length and no null terminator. May contain 0-bytes.
    Str(Cow<'de, str>),
    /// byte array
    ///
    /// Similar to strings, during deserialization byte arrays can be transient, owned, or borrowed.
    Bytes(Cow<'de, [u8]>),
    /// `None` part of an `Option`
    None,
    /// `Some` part of an `Option`
    ///
    /// # Note
    ///
    /// We use `Box` here to workaround recursive data type.
    Some(Box<Value<'de>>),
    /// The type of `()` in Rust.
    ///
    /// It represents an anonymous value containing no data.
    Unit,
    /// For example `struct Unit` or `PhantomData<T>`.
    ///
    /// It represents a named value containing no data.
    UnitStruct(&'static str),
    /// For example the `E::A` and `E::B` in `enum E { A, B }`.
    UnitVariant {
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    },
    /// For example struct `Millimeters(u8)`.
    NewtypeStruct(&'static str, Box<Value<'de>>),
    /// For example the `E::N` in `enum E { N(u8) }`.
    ///
    /// # Note
    ///
    /// We use `Box` here to workaround recursive data type.
    NewtypeVariant {
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: Box<Value<'de>>,
    },
    /// A variably sized heterogeneous sequence of values, for example `Vec<T>` or `HashSet<T>`
    Seq(Vec<Value<'de>>),
    /// A statically sized heterogeneous sequence of values for which the length will be known at deserialization time without looking at the serialized data.
    ///
    /// For example `(u8,)` or `(String, u64, Vec<T>)` or `[u64; 10]`.
    Tuple(Vec<Value<'de>>),
    /// A named tuple, for example `struct Rgb(u8, u8, u8)`.
    TupleStruct(&'static str, Vec<Value<'de>>),
    /// For example the `E::T` in `enum E { T(u8, u8) }`.
    TupleVariant {
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        fields: Vec<Value<'de>>,
    },
    /// A variably sized heterogeneous key-value pairing, for example `BTreeMap<K, V>`
    Map(Vec<(Value<'de>, Value<'de>)>),
    /// A statically sized heterogeneous key-value pairing in which the keys are compile-time
    /// constant strings and will be known at deserialization time without looking at the
    /// serialized data.
    ///
    /// For example `struct S { r: u8, g: u8, b: u8 }`.
    Struct(&'static str, Vec<(&'static str, Value<'de>)>),
    /// For example the `E::S` in `enum E { S { r: u8, g: u8, b: u8 } }`.
    StructVariant {
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        fields: Vec<(&'static str, Value<'de>)>,
    },
}

impl Value<'_> {
    pub fn into_static(self) -> Value<'static> {
        match self {
            Value::Str(Cow::Borrowed(s)) => Value::Str(Cow::Owned(s.to_string())),
            Value::Str(Cow::Owned(s)) => Value::Str(Cow::Owned(s)),
            Value::Bytes(Cow::Borrowed(b)) => Value::Bytes(Cow::Owned(b.to_vec())),
            Value::Bytes(Cow::Owned(b)) => Value::Bytes(Cow::Owned(b)),
            Value::Seq(seq) => Value::Seq(seq.into_iter().map(|v| v.into_static()).collect()),
            Value::Map(map) => Value::Map(map.into_iter().map(|(k, v)| (k.into_static(), v.into_static())).collect()),
            Value::Struct(name, fields) => {
                Value::Struct(name, fields.into_iter().map(|(k, v)| (k, v.into_static())).collect())
            }
            Value::StructVariant {
                name,
                variant_index,
                variant,
                fields,
            } => Value::StructVariant {
                name,
                variant_index,
                variant,
                fields: fields.into_iter().map(|(k, v)| (k, v.into_static())).collect(),
            },
            Value::Tuple(tuple) => Value::Tuple(tuple.into_iter().map(|v| v.into_static()).collect()),
            Value::TupleStruct(name, tuple) => {
                Value::TupleStruct(name, tuple.into_iter().map(|v| v.into_static()).collect())
            }
            Value::TupleVariant {
                name,
                variant_index,
                variant,
                fields,
            } => Value::TupleVariant {
                name,
                variant_index,
                variant,
                fields: fields.into_iter().map(|v| v.into_static()).collect(),
            },
            Value::UnitVariant {
                name,
                variant_index,
                variant,
            } => Value::UnitVariant {
                name,
                variant_index,
                variant,
            },
            Value::NewtypeStruct(name, value) => Value::NewtypeStruct(name, Box::new(value.into_static())),
            Value::NewtypeVariant {
                name,
                variant_index,
                variant,
                value,
            } => Value::NewtypeVariant {
                name,
                variant_index,
                variant,
                value: Box::new(value.into_static()),
            },
            Value::Unit => Value::Unit,
            Value::Bool(v) => Value::Bool(v),
            Value::I8(v) => Value::I8(v),
            Value::I16(v) => Value::I16(v),
            Value::I32(v) => Value::I32(v),
            Value::I64(v) => Value::I64(v),
            Value::I128(v) => Value::I128(v),
            Value::U8(v) => Value::U8(v),
            Value::U16(v) => Value::U16(v),
            Value::U32(v) => Value::U32(v),
            Value::U64(v) => Value::U64(v),
            Value::U128(v) => Value::U128(v),
            Value::F32(v) => Value::F32(v),
            Value::F64(v) => Value::F64(v),
            Value::Char(v) => Value::Char(v),
            Value::None => Value::None,
            Value::Some(v) => Value::Some(Box::new(v.into_static())),
            Value::UnitStruct(name) => Value::UnitStruct(name),
        }
    }
}

struct ValueVisitor;

impl<'de> serde::de::Visitor<'de> for ValueVisitor {
    type Value = Value<'de>;

    fn expecting(&self, f: &mut core::fmt::Formatter) -> std::fmt::Result {
        write!(f, "expecting visitor")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bool(v))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::I8(v))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::I16(v))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::I32(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::I64(v))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::U8(v))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::U16(v))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::U32(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::U64(v))
    }

    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::F32(v))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::F64(v))
    }

    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Char(v))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Str(Cow::Owned(v.to_string())))
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Str(Cow::Borrowed(v)))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Str(Cow::Owned(v)))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bytes(Cow::Owned(v.to_vec())))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bytes(Cow::Borrowed(v)))
    }

    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Bytes(Cow::Owned(v)))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::None)
    }

    fn visit_some<D>(self, d: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Value::Some(Box::new(d.deserialize_any(ValueVisitor)?)))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Value::Unit)
    }

    fn visit_newtype_struct<D>(self, d: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Value::NewtypeStruct("", Box::new(d.deserialize_any(ValueVisitor)?)))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut vec = Vec::new();
        while let Some(v) = seq.next_element()? {
            vec.push(v);
        }
        Ok(Value::Seq(vec))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut im = Vec::new();
        while let Some((k, v)) = map.next_entry()? {
            im.push((k, v));
        }
        Ok(Value::Map(im))
    }
}

impl<'de> serde::de::Deserialize<'de> for Value<'de> {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        d.deserialize_any(ValueVisitor)
    }
}

pub struct ValueDeserializer<'de, E> {
    value: Value<'de>,
    error: std::marker::PhantomData<E>,
}

impl<'de, E> serde::de::Deserializer<'de> for ValueDeserializer<'de, E>
where
    E: serde::de::Error,
{
    type Error = E;

    fn deserialize_any<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match &self.value {
            Value::Bool(_) => self.deserialize_bool(vis),
            Value::I8(_) => self.deserialize_i8(vis),
            Value::I16(_) => self.deserialize_i16(vis),
            Value::I32(_) => self.deserialize_i32(vis),
            Value::I64(_) => self.deserialize_i64(vis),
            Value::I128(_) => self.deserialize_i128(vis),
            Value::U8(_) => self.deserialize_u8(vis),
            Value::U16(_) => self.deserialize_u16(vis),
            Value::U32(_) => self.deserialize_u32(vis),
            Value::U64(_) => self.deserialize_u64(vis),
            Value::U128(_) => self.deserialize_u128(vis),
            Value::F32(_) => self.deserialize_f32(vis),
            Value::F64(_) => self.deserialize_f64(vis),
            Value::Char(_) => self.deserialize_char(vis),
            Value::Str(Cow::Owned(_)) => self.deserialize_string(vis),
            Value::Str(Cow::Borrowed(_)) => self.deserialize_str(vis),
            Value::Bytes(Cow::Owned(_)) => self.deserialize_byte_buf(vis),
            Value::Bytes(Cow::Borrowed(_)) => self.deserialize_bytes(vis),
            Value::None | Value::Some(_) => self.deserialize_option(vis),
            Value::Unit => vis.visit_unit(),
            Value::Map(_) => self.deserialize_map(vis),
            Value::Seq(_) => self.deserialize_seq(vis),
            Value::Struct(_, _) => self.deserialize_map(vis),
            v => unimplemented!("deserialize_any for {:?}", v),
        }
    }

    fn deserialize_bool<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Bool(v) => vis.visit_bool(v),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_i8<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::I8(v) => vis.visit_i8(v),
            Value::I16(v) => vis.visit_i8(i8::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I32(v) => vis.visit_i8(i8::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I64(v) => vis.visit_i8(i8::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I128(v) => vis.visit_i8(i8::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U8(v) => vis.visit_i8(i8::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U16(v) => vis.visit_i8(i8::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U32(v) => vis.visit_i8(i8::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U64(v) => vis.visit_i8(i8::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U128(v) => vis.visit_i8(i8::try_from(v).map_err(serde::de::Error::custom)?),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_i16<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::I8(v) => vis.visit_i16(i16::from(v)),
            Value::I16(v) => vis.visit_i16(v),
            Value::I32(v) => vis.visit_i16(i16::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I64(v) => vis.visit_i16(i16::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I128(v) => vis.visit_i16(i16::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U8(v) => vis.visit_i16(i16::from(v)),
            Value::U16(v) => vis.visit_i16(i16::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U32(v) => vis.visit_i16(i16::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U64(v) => vis.visit_i16(i16::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U128(v) => vis.visit_i16(i16::try_from(v).map_err(serde::de::Error::custom)?),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_i32<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::I8(v) => vis.visit_i32(i32::from(v)),
            Value::I16(v) => vis.visit_i32(i32::from(v)),
            Value::I32(v) => vis.visit_i32(v),
            Value::I64(v) => vis.visit_i32(i32::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I128(v) => vis.visit_i32(i32::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U8(v) => vis.visit_i32(i32::from(v)),
            Value::U16(v) => vis.visit_i32(i32::from(v)),
            Value::U32(v) => vis.visit_i32(i32::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U64(v) => vis.visit_i32(i32::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U128(v) => vis.visit_i32(i32::try_from(v).map_err(serde::de::Error::custom)?),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_i64<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::I8(v) => vis.visit_i64(i64::from(v)),
            Value::I16(v) => vis.visit_i64(i64::from(v)),
            Value::I32(v) => vis.visit_i64(i64::from(v)),
            Value::I64(v) => vis.visit_i64(v),
            Value::I128(v) => vis.visit_i64(i64::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U8(v) => vis.visit_i64(i64::from(v)),
            Value::U16(v) => vis.visit_i32(i32::from(v)),
            Value::U32(v) => vis.visit_i64(i64::from(v)),
            Value::U64(v) => vis.visit_i64(i64::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U128(v) => vis.visit_i64(i64::try_from(v).map_err(serde::de::Error::custom)?),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_u8<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::I8(v) => vis.visit_u8(u8::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I16(v) => vis.visit_u8(u8::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I32(v) => vis.visit_u8(u8::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I64(v) => vis.visit_u8(u8::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I128(v) => vis.visit_u8(u8::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U8(v) => vis.visit_u8(v),
            Value::U16(v) => vis.visit_u8(u8::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U32(v) => vis.visit_u8(u8::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U64(v) => vis.visit_u8(u8::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U128(v) => vis.visit_u8(u8::try_from(v).map_err(serde::de::Error::custom)?),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_u16<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::I8(v) => vis.visit_u16(u16::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I16(v) => vis.visit_u16(u16::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I32(v) => vis.visit_u16(u16::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I64(v) => vis.visit_u16(u16::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I128(v) => vis.visit_u16(u16::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U8(v) => vis.visit_u16(u16::from(v)),
            Value::U16(v) => vis.visit_u16(v),
            Value::U32(v) => vis.visit_u16(u16::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U64(v) => vis.visit_u16(u16::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U128(v) => vis.visit_u16(u16::try_from(v).map_err(serde::de::Error::custom)?),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_u32<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::I8(v) => vis.visit_u32(u32::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I16(v) => vis.visit_u32(u32::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I32(v) => vis.visit_u32(u32::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I64(v) => vis.visit_u32(u32::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I128(v) => vis.visit_u32(u32::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U8(v) => vis.visit_u32(u32::from(v)),
            Value::U16(v) => vis.visit_u32(u32::from(v)),
            Value::U32(v) => vis.visit_u32(v),
            Value::U64(v) => vis.visit_u32(u32::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U128(v) => vis.visit_u32(u32::try_from(v).map_err(serde::de::Error::custom)?),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_u64<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::I8(v) => vis.visit_u64(u64::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I16(v) => vis.visit_u64(u64::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I32(v) => vis.visit_u64(u64::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I64(v) => vis.visit_u64(u64::try_from(v).map_err(serde::de::Error::custom)?),
            Value::I128(v) => vis.visit_u64(u64::try_from(v).map_err(serde::de::Error::custom)?),
            Value::U8(v) => vis.visit_u64(u64::from(v)),
            Value::U16(v) => vis.visit_u64(u64::from(v)),
            Value::U32(v) => vis.visit_u64(u64::from(v)),
            Value::U64(v) => vis.visit_u64(v),
            Value::U128(v) => vis.visit_u64(u64::try_from(v).map_err(serde::de::Error::custom)?),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_f32<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::F32(v) => vis.visit_f32(v),
            Value::F64(v) => vis.visit_f32(v as f32),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_f64<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::F32(v) => vis.visit_f64(f64::from(v)),
            Value::F64(v) => vis.visit_f64(v),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_char<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Char(v) => vis.visit_char(v),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_str<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Str(Cow::Owned(v)) => vis.visit_string(v),
            Value::Str(Cow::Borrowed(v)) => vis.visit_borrowed_str(v),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_string<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Str(Cow::Owned(v)) => vis.visit_string(v),
            Value::Str(Cow::Borrowed(v)) => vis.visit_borrowed_str(v),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_bytes<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Bytes(Cow::Owned(v)) => vis.visit_byte_buf(v),
            Value::Bytes(Cow::Borrowed(v)) => vis.visit_borrowed_bytes(v),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_byte_buf<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Bytes(Cow::Owned(v)) => vis.visit_byte_buf(v),
            Value::Bytes(Cow::Borrowed(v)) => vis.visit_borrowed_bytes(v),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_option<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::None => vis.visit_none(),
            Value::Some(v) => vis.visit_some((*v).into_deserializer()),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_unit<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Unit => vis.visit_unit(),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::UnitStruct(vn) if vn == name => vis.visit_unit(),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::NewtypeStruct(vn, vv) if vn == name => vis.visit_newtype_struct((*vv).into_deserializer()),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_seq<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Tuple(v) => vis.visit_seq(SeqAccessor::new(v.into_iter())),
            Value::Seq(v) => vis.visit_seq(SeqAccessor::new(v.into_iter())),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_tuple<V>(self, len: usize, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Tuple(v) if len == v.len() => vis.visit_seq(SeqAccessor::new(v.into_iter())),
            Value::Seq(v) if len == v.len() => vis.visit_seq(SeqAccessor::new(v.into_iter())),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_tuple_struct<V>(self, name: &'static str, len: usize, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::TupleStruct(vn, vf) if name == vn && len == vf.len() => vis.visit_seq(SeqAccessor::new(vf.into_iter())),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_map<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Map(v) => vis.visit_map(MapAccessor::new(v.into_iter())),
            Value::Struct(_, vf) => vis.visit_map(StructAccessor::new(vf.into_iter())),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        vis: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::Struct(vn, mut vf) if vn == name => {
                // We need to sort the values by the field order in `fields`.
                let index_map: HashMap<&str, usize> = fields.iter().enumerate().map(|(i, &s)| (s, i)).collect();
                vf.sort_by_key(|(k, _)| index_map.get(k).copied().unwrap_or(usize::MAX));
                vis.visit_map(StructAccessor::new(vf.into_iter()))
            }
            Value::Map(fields) => vis.visit_map(MapAccessor::new(fields.into_iter())),
            _ => self.deserialize_any(vis),
        }
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        vis: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        vis.visit_enum(EnumAccessor::new(name, variants, self.value))
    }

    fn deserialize_identifier<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_str(vis)
    }

    fn deserialize_ignored_any<V>(self, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_any(vis)
    }
}

impl<'de, E> serde::de::IntoDeserializer<'de, E> for Value<'de>
where
    E: serde::de::Error,
{
    type Deserializer = ValueDeserializer<'de, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        ValueDeserializer {
            value: self,
            error: std::marker::PhantomData,
        }
    }
}

struct SeqAccessor<I, E> {
    values: I,
    error: std::marker::PhantomData<E>,
}

impl<I, E> SeqAccessor<I, E> {
    fn new(values: I) -> Self {
        Self {
            values,
            error: std::marker::PhantomData,
        }
    }
}

impl<'de, I, E> serde::de::SeqAccess<'de> for SeqAccessor<I, E>
where
    I: Iterator<Item = Value<'de>>,
    E: serde::de::Error,
{
    type Error = E;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.values.next() {
            Some(value) => seed.deserialize(value.into_deserializer()).map(Some),
            None => Ok(None),
        }
    }
}

struct MapAccessor<'de, I, E> {
    values: I,
    _value: Option<Value<'de>>,
    error: std::marker::PhantomData<E>,
}

impl<I, E> MapAccessor<'_, I, E> {
    fn new(values: I) -> Self {
        Self {
            values,
            _value: None,
            error: std::marker::PhantomData,
        }
    }
}

impl<'de, I, E> serde::de::MapAccess<'de> for MapAccessor<'de, I, E>
where
    I: Iterator<Item = (Value<'de>, Value<'de>)>,
    E: serde::de::Error,
{
    type Error = E;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.values.next() {
            Some((key, value)) => {
                self._value = Some(value);
                seed.deserialize(key.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self._value.take() {
            Some(value) => seed.deserialize(value.into_deserializer()),
            None => Err(serde::de::Error::custom("no value")),
        }
    }
}

struct StructAccessor<'de, I, E> {
    values: I,
    _value: Option<Value<'de>>,
    error: std::marker::PhantomData<E>,
}

impl<I, E> StructAccessor<'_, I, E> {
    fn new(values: I) -> Self {
        Self {
            values,
            _value: None,
            error: std::marker::PhantomData,
        }
    }
}

impl<'de, I, E> serde::de::MapAccess<'de> for StructAccessor<'de, I, E>
where
    I: Iterator<Item = (&'static str, Value<'de>)>,
    E: serde::de::Error,
{
    type Error = E;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self.values.next() {
            Some((key, value)) => {
                self._value = Some(value);
                seed.deserialize(key.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        match self._value.take() {
            Some(value) => seed.deserialize(value.into_deserializer()),
            None => Err(serde::de::Error::custom("no value")),
        }
    }
}

struct EnumAccessor<'de, E> {
    name: &'static str,
    variants: &'static [&'static str],
    value: Value<'de>,
    error: std::marker::PhantomData<E>,
}

impl<'de, E> EnumAccessor<'de, E> {
    fn new(name: &'static str, variants: &'static [&'static str], value: Value<'de>) -> Self {
        Self {
            name,
            variants,
            value,
            error: std::marker::PhantomData,
        }
    }
}

impl<'de, E> serde::de::EnumAccess<'de> for EnumAccessor<'de, E>
where
    E: serde::de::Error,
{
    type Error = E;
    type Variant = VariantAccessor<'de, E>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        let value = match &self.value {
            Value::UnitVariant {
                name: vn,
                variant_index: vvi,
                variant: vv,
            } if &self.name == vn && &self.variants[*vvi as usize] == vv => seed.deserialize(vv.into_deserializer())?,
            Value::TupleVariant {
                name: vn,
                variant_index: vvi,
                variant: vv,
                ..
            } if &self.name == vn && &self.variants[*vvi as usize] == vv => seed.deserialize(vv.into_deserializer())?,
            Value::StructVariant {
                name: vn,
                variant_index: vvi,
                variant: vv,
                ..
            } if &self.name == vn && &self.variants[*vvi as usize] == vv => seed.deserialize(vv.into_deserializer())?,
            Value::NewtypeVariant {
                name: vn,
                variant_index: vvi,
                variant: vv,
                ..
            } if &self.name == vn && &self.variants[*vvi as usize] == vv => seed.deserialize(vv.into_deserializer())?,
            _ => return Err(serde::de::Error::custom("invalid type")),
        };

        Ok((value, VariantAccessor::new(self.value)))
    }
}

struct VariantAccessor<'de, E> {
    value: Value<'de>,
    error: std::marker::PhantomData<E>,
}

impl<'de, E> VariantAccessor<'de, E> {
    fn new(value: Value<'de>) -> Self {
        Self {
            value,
            error: std::marker::PhantomData,
        }
    }
}

impl<'de, E> serde::de::VariantAccess<'de> for VariantAccessor<'de, E>
where
    E: serde::de::Error,
{
    type Error = E;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            Value::UnitVariant { .. } => Ok(()),
            _ => Err(serde::de::Error::custom("invalid type")),
        }
    }

    fn newtype_variant_seed<V>(self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        match self.value {
            Value::NewtypeVariant { value, .. } => seed.deserialize((*value).into_deserializer()),
            _ => Err(serde::de::Error::custom("invalid type")),
        }
    }

    fn tuple_variant<V>(self, len: usize, vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::TupleVariant { fields, .. } if fields.len() == len => vis.visit_seq(SeqAccessor::new(fields.into_iter())),
            _ => Err(serde::de::Error::custom("invalid type")),
        }
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], vis: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self.value {
            Value::StructVariant { fields: mut vf, .. } => {
                let index_map: HashMap<&str, usize> = fields.iter().enumerate().map(|(i, &s)| (s, i)).collect();
                vf.sort_by_key(|(k, _)| index_map.get(k).copied().unwrap_or(usize::MAX));
                vis.visit_map(StructAccessor::new(vf.into_iter()))
            }
            _ => Err(serde::de::Error::custom("invalid type")),
        }
    }
}

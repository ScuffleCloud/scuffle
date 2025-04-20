// This module is private and nothing here should be used outside of
// generated code.
//
// We will iterate on the implementation for a few releases and only have to
// worry about backward compatibility for the `untagged` and `tag` attributes
// rather than for this entire mechanism.
//
// This issue is tracking making some of this stuff public:
// https://github.com/serde-rs/serde/issues/741

use std::fmt;
use std::marker::PhantomData;

use serde::de::value::{MapDeserializer, SeqDeserializer};
use serde::de::{self, Deserialize, Deserializer, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, Visitor};

/// Used from generated code to buffer the contents of the Deserializer when
/// deserializing untagged enums and internally tagged enums.
///
/// Not public API. Use serde-value instead.
#[derive(Debug, Clone)]
pub enum Content<'de> {
    Bool(bool),

    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),

    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),

    F32(f32),
    F64(f64),

    Char(char),
    String(String),
    Str(&'de str),
    ByteBuf(Vec<u8>),
    Bytes(&'de [u8]),

    None,
    Some(Box<Content<'de>>),

    Unit,
    Newtype(Box<Content<'de>>),
    Seq(Vec<Content<'de>>),
    Map(Vec<(Content<'de>, Content<'de>)>),
}

impl<'de> Content<'de> {
    pub fn into_static(self) -> Content<'static> {
        match self {
            Content::Str(x) => Content::String(x.to_owned()),
            Content::String(x) => Content::String(x),
            Content::Bytes(x) => Content::ByteBuf(x.to_owned()),
            Content::ByteBuf(x) => Content::ByteBuf(x),
            Content::Bool(x) => Content::Bool(x),
            Content::U8(x) => Content::U8(x),
            Content::U16(x) => Content::U16(x),
            Content::U32(x) => Content::U32(x),
            Content::U64(x) => Content::U64(x),
            Content::I8(x) => Content::I8(x),
            Content::I16(x) => Content::I16(x),
            Content::I32(x) => Content::I32(x),
            Content::I64(x) => Content::I64(x),
            Content::F32(x) => Content::F32(x),
            Content::F64(x) => Content::F64(x),
            Content::Char(x) => Content::Char(x),
            Content::None => Content::None,
            Content::Some(x) => Content::Some(Box::new(x.into_static())),
            Content::Unit => Content::Unit,
            Content::Newtype(x) => Content::Newtype(Box::new(x.into_static())),
            Content::Seq(x) => Content::Seq(x.into_iter().map(Content::into_static).collect()),
            Content::Map(x) => Content::Map(x.into_iter().map(|(k, v)| (k.into_static(), v.into_static())).collect()),
        }
    }
}

impl<'de> Deserialize<'de> for Content<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Untagged and internally tagged enums are only supported in
        // self-describing formats.
        deserializer.deserialize_any(ContentVisitor::new())
    }
}

impl<'de, E> de::IntoDeserializer<'de, E> for Content<'de>
where
    E: de::Error,
{
    type Deserializer = ContentDeserializer<'de, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        ContentDeserializer::new(self)
    }
}

/// Used to capture data in [`Content`] from other deserializers.
/// Cannot capture externally tagged enums, `i128` and `u128`.
struct ContentVisitor<'de> {
    value: PhantomData<Content<'de>>,
}

impl<'de> ContentVisitor<'de> {
    fn new() -> Self {
        ContentVisitor { value: PhantomData }
    }
}

impl<'de> Visitor<'de> for ContentVisitor<'de> {
    type Value = Content<'de>;

    fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("any value")
    }

    fn visit_bool<F>(self, value: bool) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::Bool(value))
    }

    fn visit_i8<F>(self, value: i8) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::I8(value))
    }

    fn visit_i16<F>(self, value: i16) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::I16(value))
    }

    fn visit_i32<F>(self, value: i32) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::I32(value))
    }

    fn visit_i64<F>(self, value: i64) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::I64(value))
    }

    fn visit_u8<F>(self, value: u8) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::U8(value))
    }

    fn visit_u16<F>(self, value: u16) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::U16(value))
    }

    fn visit_u32<F>(self, value: u32) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::U32(value))
    }

    fn visit_u64<F>(self, value: u64) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::U64(value))
    }

    fn visit_f32<F>(self, value: f32) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::F32(value))
    }

    fn visit_f64<F>(self, value: f64) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::F64(value))
    }

    fn visit_char<F>(self, value: char) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::Char(value))
    }

    fn visit_str<F>(self, value: &str) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::String(value.into()))
    }

    fn visit_borrowed_str<F>(self, value: &'de str) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::Str(value))
    }

    fn visit_string<F>(self, value: String) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::String(value))
    }

    fn visit_bytes<F>(self, value: &[u8]) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::ByteBuf(value.into()))
    }

    fn visit_borrowed_bytes<F>(self, value: &'de [u8]) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::Bytes(value))
    }

    fn visit_byte_buf<F>(self, value: Vec<u8>) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::ByteBuf(value))
    }

    fn visit_unit<F>(self) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::Unit)
    }

    fn visit_none<F>(self) -> Result<Self::Value, F>
    where
        F: de::Error,
    {
        Ok(Content::None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Deserialize::deserialize(deserializer)?;
        Ok(Content::Some(Box::new(v)))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Deserialize::deserialize(deserializer)?;
        Ok(Content::Newtype(Box::new(v)))
    }

    fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut vec = Vec::<Content>::with_capacity(visitor.size_hint().unwrap_or_default());
        while let Some(e) = visitor.next_element()? {
            vec.push(e);
        }
        Ok(Content::Seq(vec))
    }

    fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut vec = Vec::<(Content, Content)>::with_capacity(visitor.size_hint().unwrap_or_default());
        while let Some(kv) = visitor.next_entry()? {
            vec.push(kv);
        }
        Ok(Content::Map(vec))
    }

    fn visit_enum<V>(self, _visitor: V) -> Result<Self::Value, V::Error>
    where
        V: EnumAccess<'de>,
    {
        Err(de::Error::custom(
            "untagged and internally tagged enums do not support enum input",
        ))
    }
}

/// Not public API
pub struct ContentDeserializer<'de, E> {
    content: Content<'de>,
    err: PhantomData<E>,
}

fn visit_content_seq<'de, V, E>(content: Vec<Content<'de>>, visitor: V) -> Result<V::Value, E>
where
    V: Visitor<'de>,
    E: de::Error,
{
    let mut seq_visitor = SeqDeserializer::new(content.into_iter());
    let value = visitor.visit_seq(&mut seq_visitor)?;
    seq_visitor.end()?;
    Ok(value)
}

fn visit_content_map<'de, V, E>(content: Vec<(Content<'de>, Content<'de>)>, visitor: V) -> Result<V::Value, E>
where
    V: Visitor<'de>,
    E: de::Error,
{
    let mut map_visitor = MapDeserializer::new(content.into_iter());
    let value = visitor.visit_map(&mut map_visitor)?;
    map_visitor.end()?;
    Ok(value)
}

/// Used when deserializing an internally tagged enum because the content
/// will be used exactly once.
impl<'de, E> Deserializer<'de> for ContentDeserializer<'de, E>
where
    E: de::Error,
{
    type Error = E;

    serde::forward_to_deserialize_any! {
        f64 f32 u64 u32 u16 u8 i64 i32 i16 i8 bool char string str
        bytes byte_buf identifier
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.content {
            Content::Bool(v) => visitor.visit_bool(v),
            Content::U8(v) => visitor.visit_u8(v),
            Content::U16(v) => visitor.visit_u16(v),
            Content::U32(v) => visitor.visit_u32(v),
            Content::U64(v) => visitor.visit_u64(v),
            Content::I8(v) => visitor.visit_i8(v),
            Content::I16(v) => visitor.visit_i16(v),
            Content::I32(v) => visitor.visit_i32(v),
            Content::I64(v) => visitor.visit_i64(v),
            Content::F32(v) => visitor.visit_f32(v),
            Content::F64(v) => visitor.visit_f64(v),
            Content::Char(v) => visitor.visit_char(v),
            Content::String(v) => visitor.visit_string(v),
            Content::Str(v) => visitor.visit_borrowed_str(v),
            Content::ByteBuf(v) => visitor.visit_byte_buf(v),
            Content::Bytes(v) => visitor.visit_borrowed_bytes(v),
            Content::Unit => visitor.visit_unit(),
            Content::None => visitor.visit_none(),
            Content::Some(v) => visitor.visit_some(ContentDeserializer::new(*v)),
            Content::Newtype(v) => visitor.visit_newtype_struct(ContentDeserializer::new(*v)),
            Content::Seq(v) => visit_content_seq(v, visitor),
            Content::Map(v) => visit_content_map(v, visitor),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.content {
            Content::None => visitor.visit_none(),
            Content::Some(v) => visitor.visit_some(ContentDeserializer::new(*v)),
            Content::Unit => visitor.visit_unit(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.content {
            Content::Unit => visitor.visit_unit(),

            // Allow deserializing newtype variant containing unit.
            //
            //     #[derive(Deserialize)]
            //     #[serde(tag = "result")]
            //     enum Response<T> {
            //         Success(T),
            //     }
            //
            // We want {"result":"Success"} to deserialize into Response<()>.
            Content::Map(ref v) if v.is_empty() => visitor.visit_unit(),
            Content::Seq(ref v) if v.is_empty() => visitor.visit_unit(),
            v => v.into_deserializer().deserialize_any(visitor),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.content {
            // As a special case, allow deserializing untagged newtype
            // variant containing unit struct.
            //
            //     #[derive(Deserialize)]
            //     struct Info;
            //
            //     #[derive(Deserialize)]
            //     #[serde(tag = "topic")]
            //     enum Message {
            //         Info(Info),
            //     }
            //
            // We want {"topic":"Info"} to deserialize even though
            // ordinarily unit structs do not deserialize from empty map/seq.
            Content::Map(ref v) if v.is_empty() => visitor.visit_unit(),
            Content::Seq(ref v) if v.is_empty() => visitor.visit_unit(),
            _ => self.deserialize_any(visitor),
        }
    }

    fn deserialize_newtype_struct<V>(self, _name: &str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.content {
            Content::Newtype(v) => visitor.visit_newtype_struct(ContentDeserializer::new(*v)),
            _ => visitor.visit_newtype_struct(self),
        }
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.content {
            Content::Seq(v) => visit_content_seq(v, visitor),
            v => v.into_deserializer().deserialize_any(visitor),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(self, _name: &'static str, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.content {
            Content::Map(v) => visit_content_map(v, visitor),
            v => v.into_deserializer().deserialize_any(visitor),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.content {
            Content::Seq(v) => visit_content_seq(v, visitor),
            Content::Map(v) => visit_content_map(v, visitor),
            v => v.into_deserializer().deserialize_any(visitor),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (variant, value) = match self.content {
            Content::Map(value) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(de::Error::invalid_value(de::Unexpected::Map, &"map with a single key"));
                    }
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(de::Error::invalid_value(de::Unexpected::Map, &"map with a single key"));
                }
                (variant, Some(value))
            }
            s @ Content::String(_) | s @ Content::Str(_) => (s, None),
            other => return other.into_deserializer().deserialize_any(visitor),
        };

        visitor.visit_enum(EnumDeserializer::new(variant, value))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }
}

impl<'de, E> ContentDeserializer<'de, E> {
    /// private API, don't use
    pub fn new(content: Content<'de>) -> Self {
        ContentDeserializer {
            content,
            err: PhantomData,
        }
    }
}

pub struct EnumDeserializer<'de, E>
where
    E: de::Error,
{
    variant: Content<'de>,
    value: Option<Content<'de>>,
    err: PhantomData<E>,
}

impl<'de, E> EnumDeserializer<'de, E>
where
    E: de::Error,
{
    pub fn new(variant: Content<'de>, value: Option<Content<'de>>) -> EnumDeserializer<'de, E> {
        EnumDeserializer {
            variant,
            value,
            err: PhantomData,
        }
    }
}

impl<'de, E> de::EnumAccess<'de> for EnumDeserializer<'de, E>
where
    E: de::Error,
{
    type Error = E;
    type Variant = VariantDeserializer<'de, Self::Error>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), E>
    where
        V: de::DeserializeSeed<'de>,
    {
        let visitor = VariantDeserializer {
            value: self.value,
            err: PhantomData,
        };
        seed.deserialize(ContentDeserializer::new(self.variant)).map(|v| (v, visitor))
    }
}

pub struct VariantDeserializer<'de, E>
where
    E: de::Error,
{
    value: Option<Content<'de>>,
    err: PhantomData<E>,
}

impl<'de, E> de::VariantAccess<'de> for VariantDeserializer<'de, E>
where
    E: de::Error,
{
    type Error = E;

    fn unit_variant(self) -> Result<(), E> {
        match self.value {
            Some(value) => de::Deserialize::deserialize(ContentDeserializer::new(value)),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, E>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(ContentDeserializer::new(value)),
            None => seed.deserialize(ContentDeserializer::new(Content::Unit)),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Some(Content::Seq(v)) => de::Deserializer::deserialize_any(SeqDeserializer::new(v.into_iter()), visitor),
            Some(other) => other.into_deserializer().deserialize_any(visitor),
            None => visitor.visit_unit(),
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        match self.value {
            Some(Content::Map(v)) => de::Deserializer::deserialize_any(MapDeserializer::new(v.into_iter()), visitor),
            Some(Content::Seq(v)) => de::Deserializer::deserialize_any(SeqDeserializer::new(v.into_iter()), visitor),
            Some(other) => other.into_deserializer().deserialize_any(visitor),
            None => visitor.visit_unit(),
        }
    }
}

impl<'de, E> de::IntoDeserializer<'de, E> for ContentDeserializer<'de, E>
where
    E: de::Error,
{
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
    }
}

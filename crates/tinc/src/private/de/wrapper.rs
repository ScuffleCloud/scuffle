use std::fmt;

struct DeserializeWrapper<T>(T);

impl<'de, T> serde::de::DeserializeSeed<'de> for DeserializeWrapper<T>
where
    T: serde::de::DeserializeSeed<'de>,
{
    type Value = T::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.0.deserialize(DeserializerWrapper::new(deserializer))
    }
}

pub struct DeserializerWrapper<D> {
    deserializer: D,
}

impl<D> DeserializerWrapper<D> {
    pub fn new(deserializer: D) -> Self {
        Self { deserializer }
    }
}

struct WrappedVisitor<V> {
    visitor: V,
}

impl<V> WrappedVisitor<V> {
    fn new(visitor: V) -> Self {
        Self { visitor }
    }
}

struct SeqAccessWrapper<A> {
    access: A,
}

impl<'de, A> serde::de::SeqAccess<'de> for SeqAccessWrapper<A>
where
    A: serde::de::SeqAccess<'de>,
{
    type Error = A::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        self.access.next_element_seed(DeserializeWrapper(seed))
    }

    fn size_hint(&self) -> Option<usize> {
        self.access.size_hint()
    }
}

impl<A> SeqAccessWrapper<A> {
    fn new(access: A) -> Self {
        Self { access }
    }
}

struct MapAccessWrapper<A> {
    access: A,
    next_key: bool,
}

impl<'de, A> serde::de::MapAccess<'de> for &mut MapAccessWrapper<A>
where
    A: serde::de::MapAccess<'de>,
{
    type Error = A::Error;

    #[inline]
    fn next_entry_seed<K, V>(&mut self, kseed: K, vseed: V) -> Result<Option<(K::Value, V::Value)>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
        V: serde::de::DeserializeSeed<'de>,
    {
        if !self.next_key {
            Err(serde::de::Error::custom("invalid call to next_entry_seed"))
        } else {
            self.access
                .next_entry_seed(DeserializeWrapper(kseed), DeserializeWrapper(vseed))
        }
    }

    #[inline]
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: serde::de::DeserializeSeed<'de>,
    {
        if !self.next_key {
            Err(serde::de::Error::custom("invalid call to next_key_seed"))
        } else {
            self.next_key = false;
            self.access.next_key_seed(DeserializeWrapper(seed))
        }
    }

    #[inline]
    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        if self.next_key {
            Err(serde::de::Error::custom("invalid call to next_value_seed"))
        } else {
            self.next_key = true;
            self.access.next_value_seed(DeserializeWrapper(seed))
        }
    }

    #[inline]
    fn size_hint(&self) -> Option<usize> {
        self.access.size_hint()
    }
}

impl<A> MapAccessWrapper<A> {
    #[inline]
    fn new(access: A) -> Self {
        Self { access, next_key: true }
    }
}

impl<'de, V> serde::de::Visitor<'de> for WrappedVisitor<V>
where
    V: serde::de::Visitor<'de>,
{
    type Value = V::Value;

    #[inline]
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.visitor.expecting(formatter)
    }

    fn visit_map<A>(self, access: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut access = MapAccessWrapper::new(access);
        let result = self.visitor.visit_map(&mut access);
        if !access.next_key && access.access.next_key::<serde::de::IgnoredAny>().is_err() {
            return result;
        }

        serde::de::IgnoredAny.visit_map(access.access).ok();

        result
    }

    #[inline]
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_bool(v)
    }

    #[inline]
    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_borrowed_bytes(v)
    }

    #[inline]
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_borrowed_str(v)
    }

    #[inline]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_byte_buf(v)
    }

    #[inline]
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_bytes(v)
    }

    #[inline]
    fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_char(v)
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        println!("visit_enum");
        self.visitor.visit_enum(EnumAccessWrapper::new(data))
    }

    #[inline]
    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_f32(v)
    }

    #[inline]
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_f64(v)
    }

    #[inline]
    fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_i128(v)
    }

    #[inline]
    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_i16(v)
    }

    #[inline]
    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_i32(v)
    }

    #[inline]
    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_i64(v)
    }

    #[inline]
    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_i8(v)
    }

    #[inline]
    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.visitor.visit_newtype_struct(DeserializerWrapper::new(deserializer))
    }

    #[inline]
    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_none()
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut seq = SeqAccessWrapper::new(seq);
        let result = self.visitor.visit_seq(&mut seq);
        serde::de::IgnoredAny.visit_seq(seq.access).ok();
        result
    }

    #[inline]
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        self.visitor.visit_some(DeserializerWrapper::new(deserializer))
    }

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_str(v)
    }

    #[inline]
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_string(v)
    }

    #[inline]
    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_u128(v)
    }

    #[inline]
    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_u16(v)
    }

    #[inline]
    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_u32(v)
    }

    #[inline]
    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_u64(v)
    }

    #[inline]
    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_u8(v)
    }

    #[inline]
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        self.visitor.visit_unit()
    }
}

struct EnumAccessWrapper<A> {
    access: A,
}

impl<A> EnumAccessWrapper<A> {
    fn new(access: A) -> Self {
        Self { access }
    }
}

impl<'de, A> serde::de::EnumAccess<'de> for EnumAccessWrapper<A>
where
    A: serde::de::EnumAccess<'de>,
{
    type Error = A::Error;
    type Variant = A::Variant;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: serde::de::DeserializeSeed<'de>,
    {
        self.access.variant_seed(DeserializeWrapper(seed))
    }
}

impl<'de, D> serde::Deserializer<'de> for DeserializerWrapper<D>
where
    D: serde::Deserializer<'de>,
{
    type Error = D::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_any(WrappedVisitor::new(visitor))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_bool(WrappedVisitor::new(visitor))
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_byte_buf(WrappedVisitor::new(visitor))
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_bytes(WrappedVisitor::new(visitor))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_char(WrappedVisitor::new(visitor))
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer
            .deserialize_enum(name, variants, WrappedVisitor::new(visitor))
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_f32(WrappedVisitor::new(visitor))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_f64(WrappedVisitor::new(visitor))
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_i128(WrappedVisitor::new(visitor))
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_i16(WrappedVisitor::new(visitor))
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_i32(WrappedVisitor::new(visitor))
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_i64(WrappedVisitor::new(visitor))
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_i8(WrappedVisitor::new(visitor))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_identifier(WrappedVisitor::new(visitor))
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_ignored_any(WrappedVisitor::new(visitor))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_map(WrappedVisitor::new(visitor))
    }

    fn deserialize_newtype_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer
            .deserialize_newtype_struct(name, WrappedVisitor::new(visitor))
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_option(WrappedVisitor::new(visitor))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_seq(WrappedVisitor::new(visitor))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_str(WrappedVisitor::new(visitor))
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_string(WrappedVisitor::new(visitor))
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer
            .deserialize_struct(name, fields, WrappedVisitor::new(visitor))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_tuple(len, WrappedVisitor::new(visitor))
    }

    fn deserialize_tuple_struct<V>(self, name: &'static str, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer
            .deserialize_tuple_struct(name, len, WrappedVisitor::new(visitor))
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_u128(WrappedVisitor::new(visitor))
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_u16(WrappedVisitor::new(visitor))
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_u32(WrappedVisitor::new(visitor))
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_u64(WrappedVisitor::new(visitor))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_u8(WrappedVisitor::new(visitor))
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_unit(WrappedVisitor::new(visitor))
    }

    fn deserialize_unit_struct<V>(self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserializer.deserialize_unit_struct(name, WrappedVisitor::new(visitor))
    }

    fn is_human_readable(&self) -> bool {
        self.deserializer.is_human_readable()
    }
}

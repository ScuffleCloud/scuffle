use std::fmt;

use scuffle_bytes_util::zero_copy::ZeroCopyReader;
use serde::de::SeqAccess;

use crate::Amf0Error;
use crate::decoder::Amf0Decoder;

/// Deserializer stream for AMF0 values.
///
/// This is a stream of AMF0 values that can be deserialized into a type.
///
/// It is created by calling [`Amf0Decoder::deserialize_stream`].
#[must_use = "Iterators are lazy and do nothing unless consumed"]
pub struct Amf0DeserializerStream<'a, R, T> {
    de: &'a mut Amf0Decoder<R>,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, R, T> Amf0DeserializerStream<'a, R, T> {
    pub(crate) fn new(de: &'a mut Amf0Decoder<R>) -> Self {
        Self {
            de,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'de, R, T> Iterator for Amf0DeserializerStream<'_, R, T>
where
    R: ZeroCopyReader<'de>,
    T: serde::de::Deserialize<'de>,
{
    type Item = Result<T, Amf0Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.de.has_remaining() {
            Ok(true) => Some(T::deserialize(&mut *self.de)),
            Ok(false) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'de, R, T> std::iter::FusedIterator for Amf0DeserializerStream<'_, R, T>
where
    R: ZeroCopyReader<'de>,
    T: serde::de::Deserialize<'de>,
{
}

pub(crate) struct MultiValueDe<'a, R> {
    pub(crate) de: &'a mut Amf0Decoder<R>,
}

impl<'de, R> SeqAccess<'de> for MultiValueDe<'_, R>
where
    R: ZeroCopyReader<'de>,
{
    type Error = Amf0Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if self.de.has_remaining()? {
            seed.deserialize(&mut *self.de).map(Some)
        } else {
            Ok(None)
        }
    }
}

pub(crate) const MULTI_VALUE_NEW_TYPE: &str = "___AMF0_MULTI_VALUE__DO_NOT_USE__";

/// A wrapper around a value that can be deserialized as a series of individual values.
///
/// This is useful if your amf0 encoded data is a series of individual values.
pub struct MultiValue<T>(pub T);

impl<'de, T> serde::de::Deserialize<'de> for MultiValue<T>
where
    T: serde::de::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<T>(std::marker::PhantomData<T>);

        impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
        where
            T: serde::de::Deserialize<'de>,
        {
            type Value = T;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a series of values")
            }

            fn visit_seq<V>(self, seq: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::SeqAccess<'de>,
            {
                T::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))
            }
        }

        deserializer
            .deserialize_newtype_struct(MULTI_VALUE_NEW_TYPE, Visitor(std::marker::PhantomData))
            .map(MultiValue)
    }
}

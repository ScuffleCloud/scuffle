use std::marker::PhantomData;

use base64::Engine;
use bytes::Bytes;

use super::{DeserializeContent, DeserializeHelper, Expected, Tracker, TrackerDeserializer, TrackerFor};

pub struct BytesTracker<T>(PhantomData<T>);

impl<T> std::fmt::Debug for BytesTracker<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BytesTracker<{}>", std::any::type_name::<T>())
    }
}

pub trait BytesLikeTracker: Tracker {
    fn set_target(&mut self, target: &mut Self::Target, buf: impl bytes::Buf);

    fn set_target_vec(&mut self, target: &mut Self::Target, data: Vec<u8>) {
        self.set_target(target, data.as_slice());
    }
}

impl BytesLikeTracker for BytesTracker<Bytes> {
    fn set_target(&mut self, target: &mut Self::Target, mut buf: impl bytes::Buf) {
        *target = buf.copy_to_bytes(buf.remaining());
    }
}
impl BytesLikeTracker for BytesTracker<Vec<u8>> {
    fn set_target(&mut self, target: &mut Self::Target, mut buf: impl bytes::Buf) {
        target.clear();
        target.reserve_exact(buf.remaining());
        while buf.has_remaining() {
            let chunk = buf.chunk();
            target.extend_from_slice(chunk);
            buf.advance(chunk.len());
        }
    }

    fn set_target_vec(&mut self, target: &mut Self::Target, data: Vec<u8>) {
        *target = data;
    }
}

impl<T> Default for BytesTracker<T> {
    fn default() -> Self {
        BytesTracker(PhantomData)
    }
}

impl<T: Expected> Tracker for BytesTracker<T> {
    type Target = T;

    fn allow_duplicates(&self) -> bool {
        false
    }
}

impl TrackerFor for Vec<u8> {
    type Tracker = BytesTracker<Vec<u8>>;
}

impl Expected for Vec<u8> {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "bytes")
    }
}

impl TrackerFor for bytes::Bytes {
    type Tracker = BytesTracker<Self>;
}

impl Expected for bytes::Bytes {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "bytes")
    }
}

impl<'de, T> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, BytesTracker<T>>
where
    T: Expected,
    BytesTracker<T>: Tracker<Target = T> + BytesLikeTracker,
{
    type Value = ();

    fn deserialize<D>(self, de: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        de.deserialize_str(self)
    }
}

impl<'de, T> serde::de::Visitor<'de> for DeserializeHelper<'_, BytesTracker<T>>
where
    T: Expected,
    BytesTracker<T>: Tracker<Target = T> + BytesLikeTracker,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        T::expecting(formatter)
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let config = base64::engine::GeneralPurposeConfig::new()
            .with_decode_allow_trailing_bits(true)
            .with_encode_padding(true)
            .with_decode_padding_mode(base64::engine::DecodePaddingMode::Indifferent);

        let alphabet = if v.as_bytes().iter().any(|b| b == &b'-' || b == &b'_') {
            &base64::alphabet::URL_SAFE
        } else {
            &base64::alphabet::STANDARD
        };

        let engine = base64::engine::GeneralPurpose::new(alphabet, config);
        let bytes = engine.decode(v.as_bytes()).map_err(serde::de::Error::custom)?;
        self.tracker.set_target_vec(self.value, bytes);
        Ok(())
    }
}

impl<'de, T> TrackerDeserializer<'de> for BytesTracker<T>
where
    T: Expected,
    BytesTracker<T>: Tracker<Target = T> + BytesLikeTracker,
{
    fn deserialize<D>(&mut self, value: &mut Self::Target, deserializer: D) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>,
    {
        deserializer.deserialize_seed(DeserializeHelper { value, tracker: self })
    }
}

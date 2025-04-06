use serde::Deserialize;

use super::struct_::{StructDeserializer, TrackerStruct};
use super::{StoreError, Tracker, TrackerError};

#[derive(Debug, Clone, Default)]
pub struct TrackerRepeatedStruct {
    pub children: Vec<TrackerStruct>,
    pub errors: Vec<TrackerError>,
}

impl StoreError for TrackerRepeatedStruct {
    fn store_error(&mut self, error: TrackerError) {
        self.errors.push(error);
    }
}

pub struct RepeatedStructDeserializer<'a, V> {
    value: &'a mut Vec<V>,
    tracker: Tracker<'a, TrackerRepeatedStruct>,
    is_invalid: std::cell::Cell<bool>,
}

impl<'a, V> RepeatedStructDeserializer<'a, V> {
    #[inline]
    pub fn new(value: &'a mut Vec<V>, tracker: Tracker<'a, TrackerRepeatedStruct>) -> Self {
        Self {
            value,
            tracker,
            is_invalid: std::cell::Cell::new(false),
        }
    }
}

impl<'de, V> serde::de::DeserializeSeed<'de> for RepeatedStructDeserializer<'_, V>
where
    V: Default,
    for<'a> StructDeserializer<'a, V>: serde::de::DeserializeSeed<'de>,
{
    type Value = ();

    #[inline]
    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(&mut self)
    }
}

impl<'de, V> serde::de::Visitor<'de> for &mut RepeatedStructDeserializer<'_, V>
where
    V: Default,
    for<'a> StructDeserializer<'a, V>: serde::de::DeserializeSeed<'de>,
{
    type Value = ();

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        if let Some(size) = seq.size_hint() {
            self.value.reserve(size);
        }

        loop {
            let mut current_value = V::default();
            let mut tracker = TrackerStruct::default();

            if seq
                .next_element_seed(StructDeserializer::new(
                    &mut current_value,
                    Tracker {
                        inner: &mut tracker,
                        shared: self.tracker.shared,
                    },
                ))?
                .is_none()
            {
                break;
            }

            self.value.push(current_value);
            self.tracker.inner.children.push(tracker);
        }

        Ok(())
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if !self.tracker.shared.fail_fast {
            while let Some((serde::de::IgnoredAny, serde::de::IgnoredAny)) = map.next_entry()? {}
        }

        Err(serde::de::Error::invalid_type(serde::de::Unexpected::Map, &self))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if !self.tracker.shared.fail_fast {
            serde::de::IgnoredAny::deserialize(deserializer)?;
        }

        Err(serde::de::Error::invalid_type(serde::de::Unexpected::NewtypeStruct, &self))
    }

    fn visit_enum<A>(self, _: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::EnumAccess<'de>,
    {
        Err(serde::de::Error::custom("unsupported type enum"))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if !self.tracker.shared.fail_fast {
            serde::de::IgnoredAny::deserialize(deserializer)?;
        }

        Err(serde::de::Error::invalid_type(serde::de::Unexpected::Option, &self))
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.is_invalid.set(true);
        formatter.write_str("an array")
    }
}

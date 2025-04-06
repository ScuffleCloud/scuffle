use serde::Deserialize;

use super::struct_::{StructDeserializer, TrackerStruct};
use super::{ErrorLocation, StoreError, Tracker, TrackerError};
use crate::__private::de::map::{Map, MappableKey};

#[derive(Debug, Clone, Default)]
pub struct TrackerMapStruct {
    pub children: linear_map::LinearMap<MappableKey, TrackerStruct>,
    pub errors: Vec<TrackerError>,
}

impl StoreError for TrackerMapStruct {
    fn store_error(&mut self, error: TrackerError) {
        self.errors.push(error);
    }
}

pub struct MapStructDeserializer<'a, M, K, V> {
    value: &'a mut M,
    tracker: Tracker<'a, TrackerMapStruct>,
    _marker: std::marker::PhantomData<(K, V)>,
}

impl<'a, M, K, V> MapStructDeserializer<'a, M, K, V> {
    #[inline]
    pub fn new(value: &'a mut M, tracker: Tracker<'a, TrackerMapStruct>) -> Self {
        Self {
            value,
            tracker,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'de, M, K, V> serde::de::DeserializeSeed<'de> for MapStructDeserializer<'_, M, K, V>
where
    K: serde::de::Deserialize<'de>,
    MappableKey: for<'a> From<&'a K>,
    M: Map<K, V>,
    V: Default,
    for<'a> StructDeserializer<'a, V>: serde::de::DeserializeSeed<'de, Value = ()>,
{
    type Value = ();

    #[inline]
    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(&mut self)
    }
}

impl<'de, M, K, V> serde::de::Visitor<'de> for &mut MapStructDeserializer<'_, M, K, V>
where
    K: serde::de::Deserialize<'de>,
    MappableKey: for<'a> From<&'a K>,
    M: Map<K, V>,
    V: Default,
    for<'a> StructDeserializer<'a, V>: serde::de::DeserializeSeed<'de, Value = ()>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if let Some(size) = map.size_hint() {
            self.value.reserve(size);
        }

        while let Some(key) = map.next_key::<K>().transpose() {
            let key = match key {
                Ok(key) => key,
                Err(err) => {
                    self.tracker.report_error(None, err)?;
                    break;
                }
            };

            let mappable_key = MappableKey::from(&key);
            let tracker = self
                .tracker
                .inner
                .children
                .entry(mappable_key.clone())
                .or_insert_with(TrackerStruct::default);

            if let Err(err) = map.next_value_seed(StructDeserializer::new(
                self.value.get(key),
                Tracker {
                    inner: tracker,
                    shared: self.tracker.shared,
                },
            )) {
                self.tracker
                    .report_error(Some(ErrorLocation::MapKey { key: mappable_key }), err)?;
            }
        }

        Ok(())
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        if !self.tracker.shared.fail_fast {
            while let Ok(Some(serde::de::IgnoredAny)) = seq.next_element() {}
        }

        Err(serde::de::Error::invalid_type(serde::de::Unexpected::Seq, &self))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if !self.tracker.shared.fail_fast {
            serde::de::IgnoredAny::deserialize(deserializer).ok();
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
            serde::de::IgnoredAny::deserialize(deserializer).ok();
        }

        Err(serde::de::Error::invalid_type(serde::de::Unexpected::Option, &self))
    }
}

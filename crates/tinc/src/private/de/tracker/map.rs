use super::{ErrorLocation, StoreError, Tracker, TrackerError};
use crate::__private::de::map::{Map, MappableKey};

#[derive(Debug, Clone, Default)]
pub struct TrackerMap {
    pub errors: Vec<TrackerError>,
}

impl StoreError for TrackerMap {
    fn store_error(&mut self, error: TrackerError) {
        self.errors.push(error);
    }
}

pub struct MapDeserializer<'a, M, K, V, U> {
    value: &'a mut M,
    tracker: Tracker<'a, TrackerMap>,
    _marker: std::marker::PhantomData<(K, V, U)>,
}

impl<'a, M, K, V> MapDeserializer<'a, M, K, V, V> {
    #[inline]
    pub fn new(value: &'a mut M, tracker: Tracker<'a, TrackerMap>) -> Self {
        Self {
            value,
            tracker,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, M, K, U> MapDeserializer<'a, M, K, i32, U> {
    #[inline]
    pub fn new_with_helper(value: &'a mut M, tracker: Tracker<'a, TrackerMap>, _: std::marker::PhantomData<U>) -> Self {
        Self {
            value,
            tracker,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'de, M, K, V, U> serde::de::DeserializeSeed<'de> for MapDeserializer<'_, M, K, V, U>
where
    K: serde::de::Deserialize<'de>,
    MappableKey: for<'a> From<&'a K>,
    M: Map<K, V>,
    V: From<U>,
    U: serde::de::Deserialize<'de>,
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

impl<'de, M, K, V, U> serde::de::Visitor<'de> for &mut MapDeserializer<'_, M, K, V, U>
where
    K: serde::de::Deserialize<'de>,
    MappableKey: for<'a> From<&'a K>,
    M: Map<K, V>,
    V: From<U>,
    U: serde::de::Deserialize<'de>,
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

            let value = match map.next_value::<U>() {
                Ok(value) => value,
                Err(err) => {
                    self.tracker.report_error(
                        Some(ErrorLocation::MapKey {
                            key: MappableKey::from(&key),
                        }),
                        err,
                    )?;
                    continue;
                }
            };

            if let Some(mapkey) = self.value.insert(key, V::from(value)) {
                let error = serde::de::Error::custom(format!("duplicate key: {mapkey}"));
                self.tracker
                    .report_error(Some(ErrorLocation::MapKey { key: mapkey }), error)?;
            }
        }

        Ok(())
    }
}

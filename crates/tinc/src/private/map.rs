use std::collections::{BTreeMap, HashMap};
use std::hash::BuildHasher;
use std::marker::PhantomData;

use super::{
    DeserializeHelper, Expected, PathToken, TrackedError, Tracker, TrackerDeserializer, TrackerFor, TrackerValidation,
    report_error, report_serde_error, set_irrecoverable,
};

pub struct MapTracker<K: Eq, T, M> {
    map: linear_map::LinearMap<K, T>,
    _marker: PhantomData<M>,
}

impl<K: Eq, T, M> std::ops::Deref for MapTracker<K, T, M> {
    type Target = linear_map::LinearMap<K, T>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl<K: Eq, T, M> std::ops::DerefMut for MapTracker<K, T, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

impl<K: Eq + std::fmt::Debug, T: std::fmt::Debug, M> std::fmt::Debug for MapTracker<K, T, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut map = f.debug_map();
        for (key, value) in self.iter() {
            map.entry(key, value);
        }
        map.finish()
    }
}

impl<K: Eq, T, M> Default for MapTracker<K, T, M> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
            map: linear_map::LinearMap::new(),
        }
    }
}

trait Map<K, V> {
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;
    fn get(&self, key: &K) -> Option<&V>;
    fn insert(&mut self, key: K, value: V) -> Option<V>;
    fn reserve(&mut self, additional: usize);
}

impl<K: Eq, T: Tracker, M: Default + Expected> Tracker for MapTracker<K, T, M> {
    type Target = M;

    fn allow_duplicates(&self) -> bool {
        true
    }
}

impl<K: std::hash::Hash + Eq + Expected, V: TrackerFor + Default + Expected, S: Default> TrackerFor for HashMap<K, V, S> {
    type Tracker = MapTracker<K, V::Tracker, HashMap<K, <V::Tracker as Tracker>::Target, S>>;
}

impl<K: Ord + Expected, V: TrackerFor + Default + Expected> TrackerFor for BTreeMap<K, V> {
    type Tracker = MapTracker<K, V::Tracker, BTreeMap<K, <V::Tracker as Tracker>::Target>>;
}

impl<K: std::hash::Hash + Eq, V: Default, S: BuildHasher> Map<K, V> for HashMap<K, V, S> {
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        HashMap::get_mut(self, key)
    }

    fn get(&self, key: &K) -> Option<&V> {
        HashMap::get(self, key)
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        HashMap::insert(self, key, value)
    }

    fn reserve(&mut self, additional: usize) {
        HashMap::reserve(self, additional)
    }
}

impl<K: Ord, V: Default> Map<K, V> for BTreeMap<K, V> {
    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        BTreeMap::get_mut(self, key)
    }

    fn get(&self, key: &K) -> Option<&V> {
        BTreeMap::get(self, key)
    }

    fn insert(&mut self, key: K, value: V) -> Option<V> {
        BTreeMap::insert(self, key, value)
    }

    fn reserve(&mut self, _: usize) {}
}

impl<'de, K, T, M> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, MapTracker<K, T, M>>
where
    for<'a> DeserializeHelper<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
    T: Tracker + Default,
    K: serde::de::Deserialize<'de> + std::cmp::Eq + Clone + std::fmt::Display + Expected,
    M: Map<K, T::Target>,
    MapTracker<K, T, M>: Tracker<Target = M>,
    T::Target: Default,
{
    type Value = ();

    fn deserialize<D>(self, de: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        de.deserialize_map(self)
    }
}

impl<'de, K, T, M> serde::de::Visitor<'de> for DeserializeHelper<'_, MapTracker<K, T, M>>
where
    for<'a> DeserializeHelper<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
    T: Tracker + Default,
    K: serde::de::Deserialize<'de> + std::cmp::Eq + Clone + std::fmt::Display + Expected,
    M: Map<K, T::Target>,
    MapTracker<K, T, M>: Tracker<Target = M>,
    T::Target: Default,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        HashMap::<K, T::Target>::expecting(formatter)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if let Some(size) = map.size_hint() {
            self.tracker.reserve(size);
            self.value.reserve(size);
        }

        let mut new_value = T::Target::default();

        while let Some(key) = map.next_key::<K>().inspect_err(|_| {
            set_irrecoverable();
        })? {
            let _token = PathToken::push_key(&key);
            let entry = self.tracker.entry(key.clone());
            if let linear_map::Entry::Occupied(entry) = &entry {
                if !entry.get().allow_duplicates() {
                    report_error(TrackedError::duplicate_field())?;
                    map.next_value::<serde::de::IgnoredAny>().inspect_err(|_| {
                        set_irrecoverable();
                    })?;
                    continue;
                }
            }

            let tracker = entry.or_insert_with(Default::default);
            let value = self.value.get_mut(&key);
            let used_new = value.is_none();
            let value = value.unwrap_or(&mut new_value);
            match map.next_value_seed(DeserializeHelper { value, tracker }) {
                Ok(_) => {}
                Err(error) => {
                    report_serde_error(error)?;
                    continue;
                }
            }

            drop(_token);

            if used_new {
                self.value.insert(key, std::mem::take(&mut new_value));
            }
        }

        Ok(())
    }
}

impl<'de, K, T, M> TrackerDeserializer<'de> for MapTracker<K, T, M>
where
    for<'a> DeserializeHelper<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
    T: Tracker + Default,
    K: serde::de::Deserialize<'de> + std::cmp::Eq + Clone + std::fmt::Display + Expected,
    M: Map<K, T::Target>,
    MapTracker<K, T, M>: Tracker<Target = M>,
    T::Target: Default,
{
    fn deserialize<D>(&mut self, value: &mut Self::Target, deserializer: D) -> Result<(), D::Error>
    where
        D: super::DeserializeContent<'de>,
    {
        deserializer.deserialize_seed(DeserializeHelper { value, tracker: self })
    }
}

impl<K, T, M> TrackerValidation for MapTracker<K, T, M>
where
    T: TrackerValidation + Default,
    K: std::cmp::Eq + Clone + std::fmt::Display + Expected,
    M: Map<K, T::Target>,
    MapTracker<K, T, M>: Tracker<Target = M>,
    T::Target: Default,
{
    fn validate<E>(&mut self, value: &Self::Target) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        for (key, tracker) in self.iter_mut() {
            let _token = PathToken::push_key(key);
            if let Some(value) = value.get(key) {
                tracker.validate(value)?;
            }
        }
        Ok(())
    }
}

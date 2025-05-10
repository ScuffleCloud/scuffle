use super::{DeserializeHelper, Expected, Tracker, TrackerDeserializer, TrackerFor, TrackerWrapper};

#[derive(Debug)]
pub struct OptionalTracker<T>(pub Option<T>);

impl<T: Tracker> TrackerWrapper for OptionalTracker<T> {
    type Tracker = T;
}

impl<T> std::ops::Deref for OptionalTracker<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for OptionalTracker<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> Default for OptionalTracker<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T: Tracker> Tracker for OptionalTracker<T> {
    type Target = Option<T::Target>;

    fn allow_duplicates(&self) -> bool {
        self.0.as_ref().map(|t| t.allow_duplicates()).unwrap_or(false)
    }
}

impl<T: TrackerFor> TrackerFor for Option<T> {
    type Tracker = OptionalTracker<T::Tracker>;
}

impl<'de, T> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, OptionalTracker<T>>
where
    for<'a> DeserializeHelper<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
    T: Tracker + Default,
    T::Target: Default,
{
    type Value = ();

    fn deserialize<D>(self, de: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if let Some(value) = self.value {
            DeserializeHelper {
                value,
                tracker: self.tracker.get_or_insert_default(),
            }
            .deserialize(de)
        } else {
            de.deserialize_option(self)
        }
    }
}

impl<'de, T> serde::de::Visitor<'de> for DeserializeHelper<'_, OptionalTracker<T>>
where
    for<'a> DeserializeHelper<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
    T: Tracker + Default,
    T::Target: Default,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        Option::<T::Target>::expecting(formatter)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }

    fn visit_some<D>(self, de: D) -> Result<Self::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        serde::de::DeserializeSeed::deserialize(
            DeserializeHelper {
                value: self.value.get_or_insert_default(),
                tracker: self.tracker.get_or_insert_default(),
            },
            de,
        )
    }
}

impl<'de, T> TrackerDeserializer<'de> for OptionalTracker<T>
where
    for<'a> DeserializeHelper<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
    T: Tracker + Default,
    T::Target: Default,
{
    fn deserialize<D>(&mut self, value: &mut Self::Target, deserializer: D) -> Result<(), D::Error>
    where
        D: super::DeserializeContent<'de>,
    {
        deserializer.deserialize_seed(DeserializeHelper { value, tracker: self })
    }
}

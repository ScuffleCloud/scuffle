use std::marker::PhantomData;

use super::Tracker;

pub trait DeserializeContent<'de>: Sized {
    type Error: serde::de::Error;

    fn deserialize<T>(self) -> Result<T, Self::Error>
    where
        T: serde::de::Deserialize<'de>,
    {
        self.deserialize_seed(PhantomData)
    }

    fn deserialize_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>;
}

pub(crate) struct MapAccessValueDeserializer<'a, T> {
    pub map: &'a mut T,
    pub deserialized: &'a mut bool,
}

impl<'de, M> DeserializeContent<'de> for MapAccessValueDeserializer<'_, M>
where
    M: serde::de::MapAccess<'de>,
{
    type Error = M::Error;

    fn deserialize_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if *self.deserialized {
            return Err(serde::de::Error::custom("invalid state: field already deserialized"));
        }

        *self.deserialized = true;
        self.map.next_value_seed(seed)
    }
}

pub(crate) struct SerdeDeserializer<D> {
    pub deserializer: D,
}

impl<'de, D> DeserializeContent<'de> for SerdeDeserializer<D>
where
    D: serde::Deserializer<'de>,
{
    type Error = D::Error;

    fn deserialize_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.deserializer)
    }
}

pub(crate) struct DeserializeHelper<'a, T: Tracker> {
    pub value: &'a mut T::Target,
    pub tracker: &'a mut T,
}

impl<'de, T: Tracker> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, Box<T>>
where
    for<'a> DeserializeHelper<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
{
    type Value = ();

    fn deserialize<D>(self, de: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        DeserializeHelper {
            value: self.value.as_mut(),
            tracker: self.tracker.as_mut(),
        }
        .deserialize(de)
    }
}

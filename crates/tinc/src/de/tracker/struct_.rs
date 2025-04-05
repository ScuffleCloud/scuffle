use serde::de::DeserializeSeed;

use super::map::TrackerMap;
use super::map_struct::TrackerMapStruct;
use super::repeated::TrackerRepeated;
use super::repeated_struct::TrackerRepeatedStruct;
use super::{StoreError, Tracker, TrackerAny, TrackerError};
use crate::de::bit_field::BitField;
use crate::de::{
    DeserializeFieldValue, StructField, StructIdentifier, StructIdentifierDeserializer, TrackedStructDeserializer,
};

#[derive(Debug, Clone, Default)]
pub struct TrackerStruct {
    fields: BitField,
    nulled: bool,
    children: linear_map::LinearMap<&'static str, TrackerAny>,
    errors: Vec<TrackerError>,
}

impl TrackerStruct {
    pub fn set_field_present(&mut self, name: &impl StructField) -> bool {
        self.fields.set(name.idx())
    }

    pub fn get_field_presence(&self, name: &impl StructField) -> bool {
        self.fields.get(name.idx())
    }

    pub fn get_child(&mut self, name: &impl StructField) -> Option<&mut TrackerAny> {
        self.children.get_mut(name.name())
    }

    pub fn push_child_struct<E>(&mut self, name: &impl StructField) -> Result<&mut TrackerStruct, E>
    where
        E: serde::de::Error,
    {
        self.set_field_present(name);
        let name = name.name();
        match self
            .children
            .entry(name)
            .or_insert_with(|| TrackerAny::Struct(TrackerStruct::default()))
        {
            TrackerAny::Struct(tracker) => Ok(tracker),
            v => Err(serde::de::Error::custom(format!(
                "bad field type: {}, expected Struct, got {}",
                name,
                v.name()
            ))),
        }
    }

    pub fn push_child_map_struct<E>(&mut self, name: &impl StructField) -> Result<&mut TrackerMapStruct, E>
    where
        E: serde::de::Error,
    {
        self.set_field_present(name);
        let name = name.name();

        match self
            .children
            .entry(name)
            .or_insert_with(|| TrackerAny::MapStruct(TrackerMapStruct::default()))
        {
            TrackerAny::MapStruct(tracker) => Ok(tracker),
            v => Err(serde::de::Error::custom(format!(
                "bad field type: {}, expected MapStruct, got {}",
                name,
                v.name()
            ))),
        }
    }

    pub fn push_child_map<E>(&mut self, name: &impl StructField) -> Result<&mut TrackerMap, E>
    where
        E: serde::de::Error,
    {
        self.set_field_present(name);
        let name = name.name();
        match self
            .children
            .entry(name)
            .or_insert_with(|| TrackerAny::Map(TrackerMap::default()))
        {
            TrackerAny::Map(tracker) => Ok(tracker),
            v => Err(serde::de::Error::custom(format!(
                "bad field type: {}, expected Map, got {}",
                name,
                v.name()
            ))),
        }
    }

    pub fn push_child_repeated_struct<E>(&mut self, name: &impl StructField) -> Result<&mut TrackerRepeatedStruct, E>
    where
        E: serde::de::Error,
    {
        self.set_field_present(name);
        let name = name.name();
        match self
            .children
            .entry(name)
            .or_insert_with(|| TrackerAny::RepeatedStruct(TrackerRepeatedStruct::default()))
        {
            TrackerAny::RepeatedStruct(tracker) => Ok(tracker),
            v => Err(serde::de::Error::custom(format!(
                "bad field type: {}, expected RepeatedStruct, got {}",
                name,
                v.name()
            ))),
        }
    }

    pub fn push_child_repeated<E>(&mut self, name: &impl StructField) -> Result<&mut TrackerRepeated, E>
    where
        E: serde::de::Error,
    {
        self.set_field_present(name);
        let name = name.name();
        match self
            .children
            .entry(name)
            .or_insert_with(|| TrackerAny::Repeated(TrackerRepeated::default()))
        {
            TrackerAny::Repeated(tracker) => Ok(tracker),
            v => Err(serde::de::Error::custom(format!(
                "bad field type: {}, expected Repeated, got {}",
                name,
                v.name()
            ))),
        }
    }
}

impl StoreError for TrackerStruct {
    fn store_error(&mut self, error: TrackerError) {
        self.errors.push(error);
    }
}

pub struct StructDeserializer<'a, T> {
    pub value: &'a mut T,
    pub tracker: Tracker<'a, TrackerStruct>,
}

impl<'a, T> StructDeserializer<'a, T> {
    #[inline]
    pub fn new(value: &'a mut T, tracker: Tracker<'a, TrackerStruct>) -> Self {
        Self { value, tracker }
    }
}

struct MapAccessNextValue<'a, M> {
    map: &'a mut M,
    was_read: &'a mut bool,
}

impl<'de, M> DeserializeFieldValue<'de> for MapAccessNextValue<'_, M>
where
    M: serde::de::MapAccess<'de>,
{
    type Error = M::Error;

    fn deserialize<T>(self) -> Result<T, Self::Error>
    where
        T: serde::de::Deserialize<'de>,
    {
        *self.was_read = true;
        self.map.next_value()
    }

    fn deserialize_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        *self.was_read = true;
        self.map.next_value_seed(seed)
    }
}

impl<'de, T: TrackedStructDeserializer<'de>> serde::de::Visitor<'de> for &mut StructDeserializer<'_, T> {
    type Value = ();

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        while let Some(field) = map.next_key_seed(StructIdentifierDeserializer::new()).transpose() {
            let mut was_read = false;
            match field {
                Ok(StructIdentifier::Field(field)) => {
                    match self.value.deserialize(
                        field,
                        &mut self.tracker,
                        MapAccessNextValue {
                            map: &mut map,
                            was_read: &mut was_read,
                        },
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            self.tracker.report_error(None, err)?;
                        }
                    }
                }
                Ok(StructIdentifier::Unknown(unknown)) => {
                    // todo: handle unknown fields
                    println!("unknown field: {}", unknown);
                }
                Err(err) => {
                    self.tracker.report_error(None, err)?;
                    break;
                }
            }

            if !was_read {
                map.next_value::<serde::de::IgnoredAny>()?;
            }
        }
        Ok(())
    }

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "a map with fields: {}", T::FIELDS.join(", "))
    }
}

pub struct OptionalStructDeserializer<'a, T> {
    pub value: &'a mut Option<T>,
    pub field: &'static str,
    pub tracker: Tracker<'a, TrackerStruct>,
}

impl<'a, T> OptionalStructDeserializer<'a, T> {
    #[inline]
    pub fn new(value: &'a mut Option<T>, field: &'static str, tracker: Tracker<'a, TrackerStruct>) -> Self {
        Self { value, field, tracker }
    }
}

impl<'de, T> serde::de::DeserializeSeed<'de> for OptionalStructDeserializer<'_, T>
where
    T: Default,
    for<'a> StructDeserializer<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_option(self)
    }
}

impl<'de, T> serde::de::Visitor<'de> for OptionalStructDeserializer<'_, T>
where
    T: Default,
    for<'a> StructDeserializer<'a, T>: serde::de::DeserializeSeed<'de, Value = ()>,
{
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an optional struct")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if self.value.is_some() || self.tracker.inner.nulled {
            return Err(serde::de::Error::duplicate_field(self.field));
        }

        self.tracker.inner.nulled = true;

        Ok(())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if self.tracker.inner.nulled {
            return Err(serde::de::Error::duplicate_field(self.field));
        }

        let value = self.value.get_or_insert_default();
        StructDeserializer::new(value, self.tracker).deserialize(deserializer)
    }
}

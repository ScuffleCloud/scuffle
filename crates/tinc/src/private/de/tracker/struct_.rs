use serde::de::DeserializeSeed;

use super::map::TrackerMap;
use super::map_struct::TrackerMapStruct;
use super::repeated::TrackerRepeated;
use super::repeated_struct::TrackerRepeatedStruct;
use super::{ErrorLocation, StoreError, Tracker, TrackerAny, TrackerError};
use crate::__private::de::bit_field::BitField;
use crate::__private::de::{
    DeserializeFieldValue, StructField, StructIdentifier, StructIdentifierDeserializer, TrackedStructDeserializer,
};

#[derive(Debug, Clone, Default)]
pub struct TrackerStruct {
    fields: BitField,
    nulled: bool,
    children: Vec<Option<TrackerAny>>,
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
        self.children.get_mut(name.idx()).and_then(|child| child.as_mut())
    }

    fn push_child(&mut self, field: &impl StructField, or_insert: impl FnOnce() -> TrackerAny) -> &mut TrackerAny {
        self.set_field_present(field);

        if self.children.len() <= field.idx() {
            self.children.resize(field.idx() + 1, None);
        }

        self.children[field.idx()].get_or_insert_with(or_insert)
    }

    pub fn push_child_struct<E>(&mut self, field: &impl StructField) -> Result<&mut TrackerStruct, E>
    where
        E: serde::de::Error,
    {
        match self.push_child(field, || TrackerAny::Struct(TrackerStruct::default())) {
            TrackerAny::Struct(tracker) => Ok(tracker),
            v => Err(serde::de::Error::custom(format!(
                "bad field type: {}, expected Struct, got {}",
                field.name(),
                v.name()
            ))),
        }
    }

    pub fn push_child_map_struct<E>(&mut self, field: &impl StructField) -> Result<&mut TrackerMapStruct, E>
    where
        E: serde::de::Error,
    {
        match self.push_child(field, || TrackerAny::MapStruct(TrackerMapStruct::default())) {
            TrackerAny::MapStruct(tracker) => Ok(tracker),
            v => Err(serde::de::Error::custom(format!(
                "bad field type: {}, expected MapStruct, got {}",
                field.name(),
                v.name()
            ))),
        }
    }

    pub fn push_child_map<E>(&mut self, field: &impl StructField) -> Result<&mut TrackerMap, E>
    where
        E: serde::de::Error,
    {
        match self.push_child(field, || TrackerAny::Map(TrackerMap::default())) {
            TrackerAny::Map(tracker) => Ok(tracker),
            v => Err(serde::de::Error::custom(format!(
                "bad field type: {}, expected Map, got {}",
                field.name(),
                v.name()
            ))),
        }
    }

    pub fn push_child_repeated_struct<E>(&mut self, field: &impl StructField) -> Result<&mut TrackerRepeatedStruct, E>
    where
        E: serde::de::Error,
    {
        match self.push_child(field, || TrackerAny::RepeatedStruct(TrackerRepeatedStruct::default())) {
            TrackerAny::RepeatedStruct(tracker) => Ok(tracker),
            v => Err(serde::de::Error::custom(format!(
                "bad field type: {}, expected RepeatedStruct, got {}",
                field.name(),
                v.name()
            ))),
        }
    }

    pub fn push_child_repeated<E>(&mut self, field: &impl StructField) -> Result<&mut TrackerRepeated, E>
    where
        E: serde::de::Error,
    {
        match self.push_child(field, || TrackerAny::Repeated(TrackerRepeated::default())) {
            TrackerAny::Repeated(tracker) => Ok(tracker),
            v => Err(serde::de::Error::custom(format!(
                "bad field type: {}, expected Repeated, got {}",
                field.name(),
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
        while let Some(field) = map.next_key_seed(StructIdentifierDeserializer::<T::Field>::new()).transpose() {
            let mut was_read = false;
            match field {
                Ok(StructIdentifier::Field(field)) => {
                    let name = field.name();
                    match self.value.deserialize(
                        field,
                        Tracker {
                            inner: self.tracker.inner,
                            shared: self.tracker.shared,
                        },
                        MapAccessNextValue {
                            map: &mut map,
                            was_read: &mut was_read,
                        },
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            self.tracker.report_error(Some(ErrorLocation::StructField { name }), err)?;
                        }
                    }
                }
                Ok(StructIdentifier::Unknown(unknown)) => {
                    self.value.handle_unknown_field(
                        &unknown,
                        Tracker {
                            inner: self.tracker.inner,
                            shared: self.tracker.shared,
                        },
                    )?;
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

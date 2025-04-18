use std::marker::PhantomData;

use super::{DeserializeHelper, Expected, Tracker, TrackerFor};
use crate::__private::well_known::WellKnownAlias;

pub struct WellKnownTracker<T>(PhantomData<T>);

impl<T> std::fmt::Debug for WellKnownTracker<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WellKnownTracker<{}>", std::any::type_name::<T>())
    }
}

impl<T: Expected> Expected for WellKnownTracker<T> {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        T::expecting(formatter)
    }
}

impl<T> Default for WellKnownTracker<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Default + Expected> Tracker for WellKnownTracker<T> {
    type Target = T;

    fn allow_duplicates(&self) -> bool {
        false
    }
}

impl TrackerFor for prost_types::Struct {
    type Tracker = WellKnownTracker<prost_types::Struct>;
}

impl Expected for prost_types::Struct {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "struct")
    }
}

impl TrackerFor for prost_types::ListValue {
    type Tracker = WellKnownTracker<prost_types::ListValue>;
}

impl Expected for prost_types::ListValue {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "list")
    }
}

impl TrackerFor for prost_types::Timestamp {
    type Tracker = WellKnownTracker<prost_types::Timestamp>;
}

impl Expected for prost_types::Timestamp {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "timestamp")
    }
}

impl TrackerFor for prost_types::Duration {
    type Tracker = WellKnownTracker<prost_types::Duration>;
}

impl Expected for prost_types::Duration {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "duration")
    }
}

impl TrackerFor for prost_types::Value {
    type Tracker = WellKnownTracker<prost_types::Value>;
}

impl Expected for prost_types::Value {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "value")
    }
}

impl TrackerFor for () {
    type Tracker = WellKnownTracker<()>;
}

impl Expected for () {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "empty object")
    }
}

impl<'de, T> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, WellKnownTracker<T>>
where
    T: WellKnownAlias + Default + Expected,
    T::Helper: serde::Deserialize<'de>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value: T::Helper = serde::Deserialize::deserialize(deserializer)?;
        *self.value = T::reverse_cast(value);
        Ok(())
    }
}

use std::marker::PhantomData;

use super::{DeserializeContent, DeserializeHelper, Expected, Tracker, TrackerDeserializer, TrackerFor};

pub struct PrimitiveTracker<T>(PhantomData<T>);

impl<T> std::fmt::Debug for PrimitiveTracker<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PrimitiveTracker<{}>", std::any::type_name::<T>())
    }
}

impl<T> Default for PrimitiveTracker<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<'de, T> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, PrimitiveTracker<T>>
where
    T: serde::Deserialize<'de>,
    PrimitiveTracker<T>: Tracker<Target = T>,
{
    type Value = ();

    fn deserialize<D>(self, de: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        *self.value = serde::Deserialize::deserialize(de)?;
        Ok(())
    }
}

impl<T: Default + Expected> Tracker for PrimitiveTracker<T> {
    type Target = T;

    #[inline(always)]
    fn allow_duplicates(&self) -> bool {
        false
    }
}

macro_rules! impl_tracker_for_primitive {
    ($($ty:ty),*) => {
        $(
            impl TrackerFor for $ty {
                type Tracker = PrimitiveTracker<$ty>;
            }

            impl Expected for $ty {
                fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, stringify!($ty))
                }
            }
        )*
    };
}

impl_tracker_for_primitive!(String, bool, u32, u64, i32, i64, f32, f64);

impl<'de, T> TrackerDeserializer<'de> for PrimitiveTracker<T>
where
    T: serde::Deserialize<'de>,
    PrimitiveTracker<T>: Tracker<Target = T>,
{
    fn deserialize<D>(&mut self, value: &mut Self::Target, deserializer: D) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>,
    {
        deserializer.deserialize_seed(DeserializeHelper { value, tracker: self })
    }
}

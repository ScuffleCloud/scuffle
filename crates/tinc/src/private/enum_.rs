use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;

use super::{DeserializeContent, DeserializeHelper, Expected, Tracker, TrackerDeserializer, TrackerFor};

pub struct EnumTracker<T>(PhantomData<T>);

impl<T> std::fmt::Debug for EnumTracker<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EnumTracker<{}>", std::any::type_name::<T>())
    }
}

impl<T> Default for EnumTracker<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T> Tracker for EnumTracker<T> {
    type Target = i32;

    fn allow_duplicates(&self) -> bool {
        false
    }
}

pub trait EnumHelper {
    type Target<E>;
}

#[repr(transparent)]
pub struct Enum<T> {
    value: i32,
    _marker: PhantomData<T>,
}

impl<T: TryFrom<i32> + Default + std::fmt::Debug> std::fmt::Debug for Enum<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Enum({:?})", T::try_from(self.value).unwrap_or_default())
    }
}

impl<T: Expected> Expected for Enum<T> {
    fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "an enum of `")?;
        T::expecting(formatter)?;
        write!(formatter, "`")
    }
}

impl<T> Default for Enum<T> {
    fn default() -> Self {
        Self {
            value: Default::default(),
            _marker: PhantomData,
        }
    }
}

impl<T> TrackerFor for Enum<T> {
    type Tracker = EnumTracker<T>;
}

impl EnumHelper for i32 {
    type Target<E> = Enum<E>;
}

impl EnumHelper for Option<i32> {
    type Target<E> = Option<Enum<E>>;
}

impl EnumHelper for Vec<i32> {
    type Target<E> = Vec<Enum<E>>;
}

impl<K: Ord> EnumHelper for BTreeMap<K, i32> {
    type Target<E> = BTreeMap<K, Enum<E>>;
}

impl<K, S> EnumHelper for HashMap<K, i32, S> {
    type Target<E> = HashMap<K, Enum<E>, S>;
}

impl<'de, T> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, EnumTracker<T>>
where
    T: serde::Deserialize<'de> + Into<i32>,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        *self.value = T::deserialize(deserializer)?.into();
        Ok(())
    }
}

impl<'de, T> TrackerDeserializer<'de> for EnumTracker<T>
where
    T: serde::Deserialize<'de> + Into<i32>,
{
    fn deserialize<D>(&mut self, value: &mut Self::Target, deserializer: D) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>,
    {
        deserializer.deserialize_seed(DeserializeHelper { value, tracker: self })
    }
}

use serde::Serialize;

impl<T> serde::Serialize for Enum<T>
where
    T: Serialize + TryFrom<i32>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = T::try_from(self.value).map_err(|_| serde::ser::Error::custom("invalid enum value"))?;
        value.serialize(serializer)
    }
}

/// # Safety
/// This trait is marked as unsafe because the implementator
/// must ensure that Helper has the same layout & memory representation as Self.
unsafe trait EnumSerialize<T> {
    type Helper: Serialize;

    fn cast(&self) -> &Self::Helper {
        // Safety: This trait is marked as unsafe and that safety condition
        // makes this operation safe.
        unsafe { &*(self as *const Self as *const Self::Helper) }
    }
}

/// Safety: [`Enum`] is `#[repr(transparent)]` for [`i32`].
unsafe impl<T: Serialize + TryFrom<i32>> EnumSerialize<T> for i32 {
    type Helper = Enum<T>;
}

/// Safety: [`Enum`] is `#[repr(transparent)]` for [`i32`].
unsafe impl<T: Serialize + TryFrom<i32>> EnumSerialize<T> for Option<i32> {
    type Helper = Option<Enum<T>>;
}

/// Safety: [`Enum`] is `#[repr(transparent)]` for [`i32`].
unsafe impl<T: Serialize + TryFrom<i32>> EnumSerialize<T> for Vec<i32> {
    type Helper = Vec<Enum<T>>;
}

/// Safety: [`Enum`] is `#[repr(transparent)]` for [`i32`].
unsafe impl<K: Serialize, V: Serialize + TryFrom<i32>> EnumSerialize<V> for BTreeMap<K, i32> {
    type Helper = BTreeMap<K, Enum<V>>;
}

/// Safety: [`Enum`] is `#[repr(transparent)]` for [`i32`].
unsafe impl<K: Serialize, V: Serialize + TryFrom<i32>> EnumSerialize<V> for HashMap<K, i32> {
    type Helper = HashMap<K, Enum<V>>;
}

#[allow(private_bounds)]
pub fn serialize_enum<T, V, S>(value: &V, serializer: S) -> Result<S::Ok, S::Error>
where
    V: EnumSerialize<T>,
    S: serde::Serializer,
{
    value.cast().serialize(serializer)
}

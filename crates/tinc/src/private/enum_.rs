use std::collections::{BTreeMap, HashMap};

use serde::Serialize;

#[repr(transparent)]
struct Enum<T>(i32, std::marker::PhantomData<T>);

impl<T> serde::Serialize for Enum<T>
where
    T: Serialize + TryFrom<i32>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let value = T::try_from(self.0).map_err(|_| serde::ser::Error::custom("invalid enum value"))?;
        value.serialize(serializer)
    }
}

/// # Safety
/// This trait is marked as unsafe because the implementator
/// must ensure that Helper has the same layout & memory representation as Self.
unsafe trait EnumSerialize<T> {
    type Helper: Serialize;

    fn cast(&self) -> &Self::Helper {
        unsafe { &*(self as *const Self as *const Self::Helper) }
    }
}

unsafe impl<T: Serialize + TryFrom<i32>> EnumSerialize<T> for i32 {
    type Helper = Enum<T>;
}

unsafe impl<T: Serialize + TryFrom<i32>> EnumSerialize<T> for Option<i32> {
    type Helper = Option<Enum<T>>;
}

unsafe impl<T: Serialize + TryFrom<i32>> EnumSerialize<T> for Vec<i32> {
    type Helper = Vec<Enum<T>>;
}

unsafe impl<K: Serialize, V: Serialize + TryFrom<i32>> EnumSerialize<V> for BTreeMap<K, i32> {
    type Helper = BTreeMap<K, Enum<V>>;
}

unsafe impl<K: Serialize, V: Serialize + TryFrom<i32>> EnumSerialize<V> for HashMap<K, i32> {
    type Helper = HashMap<K, Enum<V>>;
}

#[allow(private_bounds)]
pub fn serialize<T, V, S>(value: &V, serializer: S) -> Result<S::Ok, S::Error>
where
    V: EnumSerialize<T>,
    S: serde::Serializer,
{
    value.cast().serialize(serializer)
}

use core::fmt;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::marker::PhantomData;

use num_traits::{Float, ToPrimitive};
use serde::Serialize;
use serde::de::Error;

use super::{DeserializeContent, DeserializeHelper, Expected, Tracker, TrackerDeserializer, TrackerFor};

pub struct ExtFloatTracker<T>(PhantomData<T>);

impl<T> fmt::Debug for ExtFloatTracker<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ExtFloatTracker<{}>", std::any::type_name::<T>())
    }
}

impl<T> Default for ExtFloatTracker<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<'de, T> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, ExtFloatTracker<T>>
where
    T: serde::Deserialize<'de> + Float,
    ExtFloatTracker<T>: Tracker<Target = T>,
{
    type Value = ();

    fn deserialize<D>(self, de: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<T>(PhantomData<T>);

        impl<T> Default for Visitor<T> {
            fn default() -> Self {
                Self(PhantomData)
            }
        }

        macro_rules! visit_convert_to_float {
            ($deserialize:ident => $ty:ident) => {
                fn $deserialize<E>(self, v: $ty) -> Result<Self::Value, E>
                where
                    E: Error,
                {
                    Ok(T::from(v).unwrap())
                }
            };
        }

        impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
        where
            T: serde::Deserialize<'de> + Float,
        {
            type Value = T;

            visit_convert_to_float!(visit_f32 => f32);

            visit_convert_to_float!(visit_f64 => f64);

            visit_convert_to_float!(visit_u8 => u8);

            visit_convert_to_float!(visit_u16 => u16);

            visit_convert_to_float!(visit_u32 => u32);

            visit_convert_to_float!(visit_u64 => u64);

            visit_convert_to_float!(visit_i8 => i8);

            visit_convert_to_float!(visit_i16 => i16);

            visit_convert_to_float!(visit_i32 => i32);

            visit_convert_to_float!(visit_i64 => i64);

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, stringify!(T))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match v {
                    "Infinity" => Ok(T::infinity()),
                    "-Infinity" => Ok(T::neg_infinity()),
                    "NaN" => Ok(T::nan()),
                    _ => Err(E::custom(format!("unrecognized floating string: {}", v))),
                }
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_str(v)
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_str(&v)
            }
        }

        *self.value = de.deserialize_any(Visitor::default())?;
        Ok(())
    }
}

impl<T: Default + Expected + Float> Tracker for ExtFloatTracker<T> {
    type Target = T;

    #[inline(always)]
    fn allow_duplicates(&self) -> bool {
        false
    }
}

macro_rules! impl_tracker_for_ext_float {
    ($($ty:ty),*) => {
        $(
            impl TrackerFor for $ty {
                type Tracker = ExtFloatTracker<$ty>;
            }

            impl Expected for $ty {
                fn expecting(formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(formatter, stringify!($ty))
                }
            }
        )*
    };
}

impl_tracker_for_ext_float!(f64, f32);

impl<'de, T> TrackerDeserializer<'de> for ExtFloatTracker<T>
where
    T: serde::Deserialize<'de> + Float,
    ExtFloatTracker<T>: Tracker<Target = T>,
{
    fn deserialize<D>(&mut self, value: &mut Self::Target, deserializer: D) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>,
    {
        deserializer.deserialize_seed(DeserializeHelper { value, tracker: self })
    }
}

/// # Safety
/// This trait is marked as unsafe because the implementator
/// must ensure that Helper has the same layout & memory representation as Self.
pub(crate) unsafe trait ExtFloatAlias: Sized {
    type Helper: Sized;

    fn cast_ref(value: &Self) -> &Self::Helper {
        // Safety: this is safe given that the `unsafe trait`'s precondition is held.
        unsafe { &*(value as *const Self as *const Self::Helper) }
    }
}

pub struct FWrapper<T: Float + ToPrimitive + Display>(T);

impl<T: Float + ToPrimitive + Display> serde::Serialize for FWrapper<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match (self.0.is_nan(), self.0.is_infinite(), self.0.is_sign_negative()) {
            (true, _, _) => serializer.serialize_str("NaN"),
            (false, true, true) => serializer.serialize_str("-Infinity"),
            (false, true, false) => serializer.serialize_str("Infinity"),
            _ => {
                let converted = self
                    .0
                    .to_f64()
                    .ok_or_else(|| serde::ser::Error::custom(format!("Failed to convert {} to f64", self.0)))?;
                serializer.serialize_f64(converted)
            }
        }
    }
}

/// Safety: f32 can be wrapped by FWrapper
unsafe impl ExtFloatAlias for f32 {
    type Helper = FWrapper<f32>;
}

/// Safety: f64 can be wrapped by FWrapper
unsafe impl ExtFloatAlias for f64 {
    type Helper = FWrapper<f64>;
}

/// Safety: [`Float + ToPrimitive + Display`] is `#[repr(transparent)]` for [`FWrapper`]
unsafe impl<T: Float + ToPrimitive + Display> ExtFloatAlias for FWrapper<T> {
    type Helper = T;
}

/// Safety: If `T` is a [`ExtFloatAlias`] type, then its safe to cast `Option<T>` to `Option<T::Helper>`.
unsafe impl<T: ExtFloatAlias> ExtFloatAlias for Option<T> {
    type Helper = Option<T::Helper>;
}

/// Safety: If `T` is a [`ExtFloatAlias`] type, then its safe to cast `Vec<T>` to `Vec<T::Helper>`.
unsafe impl<T: ExtFloatAlias> ExtFloatAlias for Vec<T> {
    type Helper = Vec<T::Helper>;
}

/// Safety: `V` is a [`ExtFloatAlias`] type, then its safe to cast `BTreeMap<K, V>` to `BTreeMap<K, V::Helper>`.
unsafe impl<K, V: ExtFloatAlias> ExtFloatAlias for BTreeMap<K, V> {
    type Helper = BTreeMap<K, V::Helper>;
}

/// Safety: `V` is a [`ExtFloatAlias`] type, then its safe to cast `HashMap<K, V>` to `HashMap<K, V::Helper>`.
unsafe impl<K, V: ExtFloatAlias, S> ExtFloatAlias for HashMap<K, V, S> {
    type Helper = HashMap<K, V::Helper, S>;
}

#[allow(private_bounds)]
pub fn serialize_floats<V, S>(value: &V, serializer: S) -> Result<S::Ok, S::Error>
where
    V: ExtFloatAlias,
    V::Helper: serde::Serialize,
    S: serde::Serializer,
{
    V::cast_ref(value).serialize(serializer)
}

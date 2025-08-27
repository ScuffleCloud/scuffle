use core::fmt;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::marker::PhantomData;

use num_traits::{Float, FromPrimitive, ToPrimitive};
use serde::Serialize;
use serde::de::Error;

use super::{DeserializeContent, DeserializeHelper, Expected, Tracker, TrackerDeserializer, TrackerFor};

pub struct FloatWithNonFinTracker<T>(PhantomData<T>);

impl<T> fmt::Debug for FloatWithNonFinTracker<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FloatWithNonFinTracker<{}>", std::any::type_name::<T>())
    }
}

impl<T> Default for FloatWithNonFinTracker<T> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T: Expected> Tracker for FloatWithNonFinTracker<T> {
    type Target = T;

    #[inline(always)]
    fn allow_duplicates(&self) -> bool {
        false
    }
}

#[repr(transparent)]
pub struct FloatWithNonFinite<T>(T);

impl<T: Default> Default for FloatWithNonFinite<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Expected> TrackerFor for FloatWithNonFinite<T> {
    type Tracker = FloatWithNonFinTracker<T>;
}

impl<T> Expected for FloatWithNonFinite<T> {
    fn expecting(formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, stringify!(T))
    }
}

// Deserialization

pub trait FloatWithNonFinDesHelper: Sized {
    type Target;
}

impl FloatWithNonFinDesHelper for f32 {
    type Target = FloatWithNonFinite<f32>;
}

impl FloatWithNonFinDesHelper for f64 {
    type Target = FloatWithNonFinite<f64>;
}

impl<T: FloatWithNonFinDesHelper> FloatWithNonFinDesHelper for Option<T> {
    type Target = Option<T::Target>;
}

impl<T: FloatWithNonFinDesHelper> FloatWithNonFinDesHelper for Vec<T> {
    type Target = Vec<T::Target>;
}

impl<K, V: FloatWithNonFinDesHelper> FloatWithNonFinDesHelper for BTreeMap<K, V> {
    type Target = BTreeMap<K, V::Target>;
}

impl<K, V: FloatWithNonFinDesHelper, S> FloatWithNonFinDesHelper for HashMap<K, V, S> {
    type Target = HashMap<K, V::Target, S>;
}

impl<'de, T> serde::de::DeserializeSeed<'de> for DeserializeHelper<'_, FloatWithNonFinTracker<T>>
where
    T: serde::Deserialize<'de> + Float + ToPrimitive + FromPrimitive,
    FloatWithNonFinTracker<T>: Tracker<Target = T>,
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
            ($visitor_func:ident, $conv_func:ident, $ty:ident) => {
                fn $visitor_func<E>(self, v: $ty) -> Result<Self::Value, E>
                where
                    E: Error,
                {
                    match T::$conv_func(v) {
                        Some(v) => Ok(v),
                        None => Err(E::custom(format!("unable to extract float-type from {}", v))),
                    }
                }
            };
        }

        impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
        where
            T: serde::Deserialize<'de> + Float + ToPrimitive + FromPrimitive,
        {
            type Value = T;

            visit_convert_to_float!(visit_f32, from_f32, f32);

            visit_convert_to_float!(visit_f64, from_f64, f64);

            visit_convert_to_float!(visit_u8, from_u8, u8);

            visit_convert_to_float!(visit_u16, from_u16, u16);

            visit_convert_to_float!(visit_u32, from_u32, u32);

            visit_convert_to_float!(visit_u64, from_u64, u64);

            visit_convert_to_float!(visit_i8, from_i8, i8);

            visit_convert_to_float!(visit_i16, from_i16, i16);

            visit_convert_to_float!(visit_i32, from_i32, i32);

            visit_convert_to_float!(visit_i64, from_i64, i64);

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

impl<'de, T> TrackerDeserializer<'de> for FloatWithNonFinTracker<T>
where
    T: serde::Deserialize<'de> + Float + FromPrimitive,
    FloatWithNonFinTracker<T>: Tracker<Target = T>,
{
    fn deserialize<D>(&mut self, value: &mut Self::Target, deserializer: D) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>,
    {
        deserializer.deserialize_seed(DeserializeHelper { value, tracker: self })
    }
}

// Serialization

impl<T: Float + FromPrimitive + Display> serde::Serialize for FloatWithNonFinite<T> {
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

/// # Safety
/// This trait is marked as unsafe because the implementator
/// must ensure that Helper has the same layout & memory representation as Self.
unsafe trait FloatWithNonFinSerHelper: Sized {
    type Helper: Sized;

    fn cast(value: &Self) -> &Self::Helper {
        // Safety: this is safe given that the `unsafe trait`'s precondition is held.
        unsafe { &*(value as *const Self as *const Self::Helper) }
    }
}

/// Safety: [`FloatWithNonFinite`] is `#[repr(transparent)]` for [`f32`].
unsafe impl FloatWithNonFinSerHelper for f32 {
    type Helper = FloatWithNonFinite<f32>;
}

/// Safety: [`FloatWithNonFinite`] is `#[repr(transparent)]` for [`f64`].
unsafe impl FloatWithNonFinSerHelper for f64 {
    type Helper = FloatWithNonFinite<f64>;
}

/// Safety: [`FloatWithNonFinite<T>`] is naturally same as [`FloatWithNonFinite<T>`].
unsafe impl<T: Float + FromPrimitive> FloatWithNonFinSerHelper for FloatWithNonFinite<T> {
    type Helper = FloatWithNonFinite<T>;
}

/// Safety: If `T` is a [`FloatWithNonFinSerHelper`] type, then `Option<T>` can be cast to `Option<T::Helper>`
unsafe impl<T: FloatWithNonFinSerHelper> FloatWithNonFinSerHelper for Option<T> {
    type Helper = Option<T::Helper>;
}

/// Safety: If `T` is a [`FloatWithNonFinSerHelper`] type, then `Vec<T>` can be cast to `Vec<T::Helper>`
unsafe impl<T: FloatWithNonFinSerHelper> FloatWithNonFinSerHelper for Vec<T> {
    type Helper = Vec<T::Helper>;
}

/// Safety: If `T` is a [`FloatWithNonFinSerHelper`] type, then `BTreeMap<K,V>` can be cast to `BTreeMap<K,V::Helper>`
unsafe impl<K, V: FloatWithNonFinSerHelper> FloatWithNonFinSerHelper for BTreeMap<K, V> {
    type Helper = BTreeMap<K, V::Helper>;
}

/// Safety: If `T` is a [`FloatWithNonFinSerHelper`] type, then `HashMap<K,V>` can be cast to `HashMap<K,V::Helper>`
unsafe impl<K, V: FloatWithNonFinSerHelper, S> FloatWithNonFinSerHelper for HashMap<K, V, S> {
    type Helper = HashMap<K, V::Helper, S>;
}

#[allow(private_bounds)]
pub fn serialize_floats_with_non_finite<V, S>(value: &V, serializer: S) -> Result<S::Ok, S::Error>
where
    V: FloatWithNonFinSerHelper,
    V::Helper: serde::Serialize,
    S: serde::Serializer,
{
    V::cast(value).serialize(serializer)
}

use core::fmt;
use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use serde::ser::{SerializeMap, SerializeSeq};
use serde::{Deserialize, Serialize};

use super::{DeserializeContent, DeserializeHelper, Expected, Tracker, TrackerDeserializer, TrackerFor, TrackerValidation};

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

impl<'de, T> TrackerDeserializer<'de> for WellKnownTracker<T>
where
    T: WellKnownAlias + Default + Expected,
    T::Helper: serde::Deserialize<'de>,
{
    fn deserialize<D>(&mut self, value: &mut Self::Target, deserializer: D) -> Result<(), D::Error>
    where
        D: DeserializeContent<'de>,
    {
        deserializer.deserialize_seed(DeserializeHelper { tracker: self, value })
    }
}

impl<T: Default + Expected> TrackerValidation for WellKnownTracker<T> {
    fn validate<E>(&mut self, _: &Self::Target) -> Result<(), E>
    where
        E: serde::de::Error,
    {
        Ok(())
    }
}

#[repr(transparent)]
pub struct List(pub prost_types::ListValue);

impl From<prost_types::ListValue> for List {
    fn from(value: prost_types::ListValue) -> Self {
        Self(value)
    }
}

impl From<List> for prost_types::ListValue {
    fn from(value: List) -> Self {
        value.0
    }
}

#[repr(transparent)]
pub struct Struct(pub prost_types::Struct);

impl From<prost_types::Struct> for Struct {
    fn from(value: prost_types::Struct) -> Self {
        Self(value)
    }
}

impl From<Struct> for prost_types::Struct {
    fn from(value: Struct) -> Self {
        value.0
    }
}

#[repr(transparent)]
pub struct Value(pub prost_types::Value);

impl From<prost_types::Value> for Value {
    fn from(value: prost_types::Value) -> Self {
        Self(value)
    }
}

#[repr(transparent)]
pub struct Timestamp(pub prost_types::Timestamp);

impl From<prost_types::Timestamp> for Timestamp {
    fn from(value: prost_types::Timestamp) -> Self {
        Self(value)
    }
}

#[repr(transparent)]
pub struct Duration(pub prost_types::Duration);

impl From<prost_types::Duration> for Duration {
    fn from(value: prost_types::Duration) -> Self {
        Self(value)
    }
}

#[repr(transparent)]
pub struct Empty(pub ());

impl From<()> for Empty {
    fn from(value: ()) -> Self {
        Self(value)
    }
}

impl From<Empty> for () {
    fn from(value: Empty) -> Self {
        value.0
    }
}

impl<'de> serde::Deserialize<'de> for List {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = List;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a list")
            }

            fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::SeqAccess<'de>,
            {
                let mut values = Vec::new();

                while let Some(value) = visitor.next_element::<Value>()? {
                    values.push(value.0);
                }

                Ok(List(prost_types::ListValue { values }))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

impl serde::Serialize for List {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.values.len()))?;

        for value in self.0.values.iter() {
            seq.serialize_element(WellKnownAlias::cast_ref(value))?;
        }

        seq.end()
    }
}

impl<'de> serde::Deserialize<'de> for Struct {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Struct;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a struct")
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut fields = BTreeMap::new();

                while let Some((key, value)) = visitor.next_entry::<String, Value>()? {
                    fields.insert(key, value.0);
                }

                Ok(Struct(prost_types::Struct { fields }))
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

impl serde::Serialize for Struct {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.fields.len()))?;

        for (key, value) in self.0.fields.iter() {
            map.serialize_key(key)?;
            map.serialize_value(WellKnownAlias::cast_ref(value))?;
        }

        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        macro_rules! visit_number {
            ($visit_fn:ident, $ty:ty) => {
                fn $visit_fn<E>(self, v: $ty) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(Value(prost_types::Value {
                        kind: Some(prost_types::value::Kind::NumberValue(v as f64)),
                    }))
                }
            };
        }

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Value;

            visit_number!(visit_f32, f32);

            visit_number!(visit_f64, f64);

            visit_number!(visit_i8, i8);

            visit_number!(visit_i16, i16);

            visit_number!(visit_i32, i32);

            visit_number!(visit_i64, i64);

            visit_number!(visit_u8, u8);

            visit_number!(visit_u16, u16);

            visit_number!(visit_u32, u32);

            visit_number!(visit_u64, u64);

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a value")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value(prost_types::Value {
                    kind: Some(prost_types::value::Kind::BoolValue(v)),
                }))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value(prost_types::Value {
                    kind: Some(prost_types::value::Kind::StringValue(v)),
                }))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(v.to_string())
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(v)
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let Struct(value) = Struct::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;

                Ok(Value(prost_types::Value {
                    kind: Some(prost_types::value::Kind::StructValue(value)),
                }))
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let List(value) = List::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))?;

                Ok(Value(prost_types::Value {
                    kind: Some(prost_types::value::Kind::ListValue(value)),
                }))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Value(prost_types::Value {
                    kind: Some(prost_types::value::Kind::NullValue(prost_types::NullValue::NullValue as i32)),
                }))
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::de::Deserializer<'de>,
            {
                deserializer.deserialize_any(self)
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_none()
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_any(self)
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

impl serde::Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.0.kind {
            None | Some(prost_types::value::Kind::NullValue(_)) => serializer.serialize_none(),
            Some(prost_types::value::Kind::NumberValue(value)) => serializer.serialize_f64(*value),
            Some(prost_types::value::Kind::StringValue(value)) => serializer.serialize_str(value),
            Some(prost_types::value::Kind::BoolValue(value)) => serializer.serialize_bool(*value),
            Some(prost_types::value::Kind::StructValue(value)) => WellKnownAlias::cast_ref(value).serialize(serializer),
            Some(prost_types::value::Kind::ListValue(value)) => WellKnownAlias::cast_ref(value).serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = Timestamp;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a timestamp")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let timestamp = chrono::DateTime::parse_from_rfc3339(v).map_err(E::custom)?.to_utc();

                Ok(Timestamp(prost_types::Timestamp {
                    seconds: timestamp.timestamp(),
                    nanos: timestamp.timestamp_subsec_nanos() as i32,
                }))
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl serde::Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let total_nanos = self.0.seconds * 1_000_000_000 + self.0.nanos as i64;
        let timestamp = chrono::DateTime::from_timestamp_nanos(total_nanos);
        serializer.serialize_str(&timestamp.to_rfc3339())
    }
}

impl<'de> serde::Deserialize<'de> for Duration {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = Duration;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a duration")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let mut v = v.strip_suffix('s').ok_or_else(|| E::custom("invalid duration format"))?;
                if v.is_empty() || !v.is_ascii() {
                    return Err(E::custom("invalid duration format"));
                }

                let negative = match v.as_bytes()[0] {
                    b'-' => {
                        v = &v[1..];
                        -1
                    }
                    b'+' => {
                        v = &v[1..];
                        1
                    }
                    b'0'..=b'9' => 1,
                    _ => {
                        return Err(E::custom("invalid duration format"));
                    }
                };

                if v.is_empty() || !v.as_bytes()[0].is_ascii_digit() {
                    return Err(E::custom("invalid duration format"));
                }

                let (seconds, nanos) = v.split_once('.').unwrap_or((v, "0"));
                if nanos.is_empty() || !nanos.as_bytes()[0].is_ascii_digit() {
                    return Err(E::custom("invalid duration format"));
                }

                let seconds = seconds.parse::<i64>().map_err(|_| E::custom("invalid duration format"))? * negative as i64;
                let nanos_size = nanos.len().min(9);
                // only take the first 9 digits of nanos (we only support nanosecond precision)
                let nanos = &nanos[..nanos_size];
                // convert the string to an i32
                let nanos = nanos.parse::<i32>().map_err(|_| E::custom("invalid duration format"))?;

                // We now need to scale the nanos by the number of digits in the nanos string
                let multiplier = 10_i32.pow(9 - nanos_size as u32);
                let nanos = nanos * multiplier * negative;

                Ok(Duration(prost_types::Duration { seconds, nanos }))
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

impl serde::Serialize for Duration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let seconds = self.0.seconds;
        let nanos = self.0.nanos;

        let mut s = seconds.to_string();

        if nanos != 0 {
            // Convert nanos to 9-digit zero-padded string
            let mut buf = [b'0'; 9];
            let mut n = nanos;
            let mut i = 9;
            let mut first_non_zero = None;
            while n != 0 && i > 0 {
                i -= 1;
                let modulus = n % 10;
                if modulus != 0 && first_non_zero.is_none() {
                    first_non_zero = Some(i);
                }
                buf[i] = b'0' + (n % 10) as u8;
                n /= 10;
            }

            s.push('.');
            s.push_str(unsafe { std::str::from_utf8_unchecked(&buf[..first_non_zero.unwrap_or(8) + 1]) });
            s.push('s');
        } else {
            s.push('s');
        }

        serializer.serialize_str(&s)
    }
}

impl<'de> serde::Deserialize<'de> for Empty {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Empty;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an empty value")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
                    return Err(serde::de::Error::custom("expected empty sequence"));
                }

                Ok(Empty(()))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                if map.next_key::<serde::de::IgnoredAny>()?.is_some() {
                    return Err(serde::de::Error::custom("expected empty map"));
                }

                Ok(Empty(()))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.is_empty() {
                    return Ok(Empty(()));
                }
                Err(E::custom("expected empty string"))
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_any(self)
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.is_empty() {
                    return Ok(Empty(()));
                }

                Err(E::custom("expected empty bytes"))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Empty(()))
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Empty(()))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

impl serde::Serialize for Empty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_map(Some(0))?.end()
    }
}

/// # Safety
/// This trait is marked as unsafe because the implementator
/// must ensure that Helper has the same layout & memory representation as Self.
pub(crate) unsafe trait WellKnownAlias: Sized {
    type Helper: Sized;

    fn reverse_cast(value: Self::Helper) -> Self {
        const {
            assert!(std::mem::size_of::<Self>() == std::mem::size_of::<Self::Helper>());
            assert!(std::mem::align_of::<Self>() == std::mem::align_of::<Self::Helper>());
        };

        let mut value = ManuallyDrop::new(value);
        let casted = unsafe { &mut *(&mut value as *mut _ as *mut ManuallyDrop<Self>) };
        unsafe { ManuallyDrop::take(casted) }
    }

    fn cast_ref(value: &Self) -> &Self::Helper {
        unsafe { &*(value as *const Self as *const Self::Helper) }
    }
}

/// Safety: [`List`] is `#[repr(transparent)]` for [`prost_types::ListValue`]
unsafe impl WellKnownAlias for prost_types::ListValue {
    type Helper = List;
}

/// Safety: [`Struct`] is `#[repr(transparent)]` for [`prost_types::Struct`]
unsafe impl WellKnownAlias for prost_types::Struct {
    type Helper = Struct;
}

/// Safety: [`Value`] is `#[repr(transparent)]` for [`prost_types::Value`]
unsafe impl WellKnownAlias for prost_types::Value {
    type Helper = Value;
}

/// Safety: [`Timestamp`] is `#[repr(transparent)]` for [`prost_types::Timestamp`]
unsafe impl WellKnownAlias for prost_types::Timestamp {
    type Helper = Timestamp;
}

/// Safety: [`Duration`] is `#[repr(transparent)]` for [`prost_types::Duration`]
unsafe impl WellKnownAlias for prost_types::Duration {
    type Helper = Duration;
}

/// Safety: [`Empty`] is `#[repr(transparent)]` for `()`
unsafe impl WellKnownAlias for () {
    type Helper = Empty;
}

/// Safety: If `T` is a [`SerializeWellKnown`] type, then its safe to cast `Option<T>` to `Option<T::Helper>`.
unsafe impl<T: WellKnownAlias> WellKnownAlias for Option<T> {
    type Helper = Option<T::Helper>;
}

/// Safety: If `T` is a [`SerializeWellKnown`] type, then its safe to cast `Vec<T>` to `Vec<T::Helper>`.
unsafe impl<T: WellKnownAlias> WellKnownAlias for Vec<T> {
    type Helper = Vec<T::Helper>;
}

/// Safety: `V` is a [`SerializeWellKnown`] type, then its safe to cast `BTreeMap<K, V>` to `BTreeMap<K, V::Helper>`.
unsafe impl<K, V: WellKnownAlias> WellKnownAlias for BTreeMap<K, V> {
    type Helper = BTreeMap<K, V::Helper>;
}

/// Safety: `V` is a [`SerializeWellKnown`] type, then its safe to cast `HashMap<K, V>` to `HashMap<K, V::Helper>`.
unsafe impl<K, V: WellKnownAlias, S> WellKnownAlias for HashMap<K, V, S> {
    type Helper = HashMap<K, V::Helper, S>;
}

#[allow(private_bounds)]
pub fn serialize_well_known<V, S>(value: &V, serializer: S) -> Result<S::Ok, S::Error>
where
    V: WellKnownAlias,
    V::Helper: serde::Serialize,
    S: serde::Serializer,
{
    V::cast_ref(value).serialize(serializer)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_duration_deserialize() {
        let cases = [
            // Basic positive and negative cases
            ("1s", prost_types::Duration { seconds: 1, nanos: 0 }),
            ("-1s", prost_types::Duration { seconds: -1, nanos: 0 }),
            // Zero cases
            ("0s", prost_types::Duration { seconds: 0, nanos: 0 }),
            ("-0s", prost_types::Duration { seconds: 0, nanos: 0 }),
            // Positive fractions
            (
                "0.5s",
                prost_types::Duration {
                    seconds: 0,
                    nanos: 500_000_000,
                },
            ),
            (
                "1.5s",
                prost_types::Duration {
                    seconds: 1,
                    nanos: 500_000_000,
                },
            ),
            ("0.000000001s", prost_types::Duration { seconds: 0, nanos: 1 }),
            ("0.000000123s", prost_types::Duration { seconds: 0, nanos: 123 }),
            (
                "0.999999999s",
                prost_types::Duration {
                    seconds: 0,
                    nanos: 999_999_999,
                },
            ),
            ("1.000000001s", prost_types::Duration { seconds: 1, nanos: 1 }),
            (
                "1.234567890s",
                prost_types::Duration {
                    seconds: 1,
                    nanos: 234_567_890,
                },
            ),
            // Negative fractions
            (
                "-0.5s",
                prost_types::Duration {
                    seconds: 0,
                    nanos: -500_000_000,
                },
            ),
            (
                "-1.5s",
                prost_types::Duration {
                    seconds: -1,
                    nanos: -500_000_000,
                },
            ),
            ("-0.000000001s", prost_types::Duration { seconds: 0, nanos: -1 }),
            ("-0.000000123s", prost_types::Duration { seconds: 0, nanos: -123 }),
            (
                "-0.999999999s",
                prost_types::Duration {
                    seconds: 0,
                    nanos: -999_999_999,
                },
            ),
            ("-1.000000001s", prost_types::Duration { seconds: -1, nanos: -1 }),
            (
                "-1.234567890s",
                prost_types::Duration {
                    seconds: -1,
                    nanos: -234_567_890,
                },
            ),
            // Large positive integers
            ("1000s", prost_types::Duration { seconds: 1000, nanos: 0 }),
            (
                "3155695200s",
                prost_types::Duration {
                    seconds: 3155695200,
                    nanos: 0,
                },
            ), // 100 years
            (
                "1000000000s",
                prost_types::Duration {
                    seconds: 1_000_000_000,
                    nanos: 0,
                },
            ),
            (
                "9223372036s",
                prost_types::Duration {
                    seconds: 9223372036,
                    nanos: 0,
                },
            ),
            // Large negative integers
            (
                "-1000s",
                prost_types::Duration {
                    seconds: -1000,
                    nanos: 0,
                },
            ),
            (
                "-3155695200s",
                prost_types::Duration {
                    seconds: -3155695200,
                    nanos: 0,
                },
            ), // -100 years
            (
                "-1000000000s",
                prost_types::Duration {
                    seconds: -1_000_000_000,
                    nanos: 0,
                },
            ),
            (
                "-9223372036s",
                prost_types::Duration {
                    seconds: -9223372036,
                    nanos: 0,
                },
            ),
            // Large positive fractions
            (
                "3155695200.987654321s",
                prost_types::Duration {
                    seconds: 3155695200,
                    nanos: 987_654_321,
                },
            ),
            (
                "1000000000.123456789s",
                prost_types::Duration {
                    seconds: 1_000_000_000,
                    nanos: 123_456_789,
                },
            ),
            (
                "9223372036.854775807s",
                prost_types::Duration {
                    seconds: 9223372036,
                    nanos: 854_775_807,
                },
            ),
            (
                "9223372036.999999999s",
                prost_types::Duration {
                    seconds: 9223372036,
                    nanos: 999_999_999,
                },
            ),
            // Large negative fractions
            (
                "-3155695200.987654321s",
                prost_types::Duration {
                    seconds: -3155695200,
                    nanos: -987_654_321,
                },
            ),
            (
                "-1000000000.123456789s",
                prost_types::Duration {
                    seconds: -1_000_000_000,
                    nanos: -123_456_789,
                },
            ),
            (
                "-9223372036.854775807s",
                prost_types::Duration {
                    seconds: -9223372036,
                    nanos: -854_775_807,
                },
            ),
            (
                "-9223372036.999999999s",
                prost_types::Duration {
                    seconds: -9223372036,
                    nanos: -999_999_999,
                },
            ),
            // Near-boundary handling
            (
                "9223372036.854775807s",
                prost_types::Duration {
                    seconds: 9223372036,
                    nanos: 854_775_807,
                },
            ),
            (
                "-9223372036.854775807s",
                prost_types::Duration {
                    seconds: -9223372036,
                    nanos: -854_775_807,
                },
            ),
            (
                "9223372036.999999999s",
                prost_types::Duration {
                    seconds: 9223372036,
                    nanos: 999_999_999,
                },
            ),
            (
                "-9223372036.999999999s",
                prost_types::Duration {
                    seconds: -9223372036,
                    nanos: -999_999_999,
                },
            ),
            // Exact integers with max precision
            ("1.000000000s", prost_types::Duration { seconds: 1, nanos: 0 }),
            ("-1.000000000s", prost_types::Duration { seconds: -1, nanos: 0 }),
            ("0.000000000s", prost_types::Duration { seconds: 0, nanos: 0 }),
            ("-0.000000000s", prost_types::Duration { seconds: 0, nanos: 0 }),
            // Various decimal precision levels
            (
                "1.2s",
                prost_types::Duration {
                    seconds: 1,
                    nanos: 200_000_000,
                },
            ),
            (
                "1.23s",
                prost_types::Duration {
                    seconds: 1,
                    nanos: 230_000_000,
                },
            ),
            (
                "1.234s",
                prost_types::Duration {
                    seconds: 1,
                    nanos: 234_000_000,
                },
            ),
            (
                "1.2345s",
                prost_types::Duration {
                    seconds: 1,
                    nanos: 234_500_000,
                },
            ),
            (
                "1.23456s",
                prost_types::Duration {
                    seconds: 1,
                    nanos: 234_560_000,
                },
            ),
            (
                "1.234567s",
                prost_types::Duration {
                    seconds: 1,
                    nanos: 234_567_000,
                },
            ),
            (
                "1.2345678s",
                prost_types::Duration {
                    seconds: 1,
                    nanos: 234_567_800,
                },
            ),
            (
                "1.23456789s",
                prost_types::Duration {
                    seconds: 1,
                    nanos: 234_567_890,
                },
            ),
            (
                "1.234567891s",
                prost_types::Duration {
                    seconds: 1,
                    nanos: 234_567_891,
                },
            ),
            // this will be truncated to 1.23456789s
            (
                "1.2345678901s",
                prost_types::Duration {
                    seconds: 1,
                    nanos: 234_567_890,
                },
            ),
            (
                "1.23456789055s",
                prost_types::Duration {
                    seconds: 1,
                    nanos: 234_567_890,
                },
            ),
        ];

        for (idx, (input, expected)) in cases.into_iter().enumerate() {
            let Duration(duration) = Duration::deserialize(serde_json::Value::String(input.to_string())).unwrap();
            assert_eq!(duration, expected, "case {idx} failed with input {input}");
        }
    }

    #[test]
    fn test_duration_deserialize_error() {
        let error_cases = [
            // Completely invalid strings
            ("", "invalid duration format"),
            ("abc", "invalid duration format"),
            ("randomtext", "invalid duration format"),
            ("1second", "invalid duration format"),
            ("s1", "invalid duration format"),
            ("1.23", "invalid duration format"),
            ("1 s", "invalid duration format"),
            ("s", "invalid duration format"),
            ("1.s", "invalid duration format"),
            ("1.0.s", "invalid duration format"),
            // Invalid number formats
            ("1..0s", "invalid duration format"),
            ("1..s", "invalid duration format"),
            ("-1..0s", "invalid duration format"),
            ("-1..s", "invalid duration format"),
            (".1s", "invalid duration format"),
            ("-.1s", "invalid duration format"),
            ("1.s", "invalid duration format"),
            ("-1.s", "invalid duration format"),
            ("1.0.0s", "invalid duration format"),
            ("1.0.s", "invalid duration format"),
            // Invalid negative signs
            ("--1s", "invalid duration format"),
            ("-s", "invalid duration format"),
            ("--0.5s", "invalid duration format"),
            // Incorrect use of decimal points
            ("1..s", "invalid duration format"),
            ("-1..s", "invalid duration format"),
            ("0..0s", "invalid duration format"),
            ("0.0.0s", "invalid duration format"),
            ("1.0.0s", "invalid duration format"),
            // Missing unit
            ("1", "invalid duration format"),
            ("0.5", "invalid duration format"),
            ("-0.5", "invalid duration format"),
            ("1.", "invalid duration format"),
            ("-1.", "invalid duration format"),
            // Extra characters
            ("1sabc", "invalid duration format"),
            ("-1sabc", "invalid duration format"),
            ("1.0sabc", "invalid duration format"),
            ("0.5sab", "invalid duration format"),
            ("-0.5sxyz", "invalid duration format"),
            // Misplaced 's' character
            ("1s1s", "invalid duration format"),
            ("1.0ss", "invalid duration format"),
            ("-1s1", "invalid duration format"),
            ("-0.5s0.5s", "invalid duration format"),
            // Multiple decimals
            ("1.1.1s", "invalid duration format"),
            ("-1.1.1s", "invalid duration format"),
            ("0.1.1s", "invalid duration format"),
            ("-0.1.1s", "invalid duration format"),
            // Overflow beyond maximum supported range
            ("9223372036854775808s", "invalid duration format"), // One more than i64::MAX
            ("-9223372036854775809s", "invalid duration format"), // One less than i64::MIN
            ("10000000000000000000.0s", "invalid duration format"), // Excessively large number
            ("-10000000000000000000.0s", "invalid duration format"), // Excessively large negative number
            // Non-numeric characters in numbers
            ("1a.0s", "invalid duration format"),
            ("1.0as", "invalid duration format"),
            ("-1.a0s", "invalid duration format"),
            ("1.0a0s", "invalid duration format"),
            ("1a0s", "invalid duration format"),
            // Empty fraction part
            ("1.s", "invalid duration format"),
            ("-1.s", "invalid duration format"),
            // Invalid leading/trailing spaces
            (" 1s", "invalid duration format"),
            ("1s ", "invalid duration format"),
            ("- 1s", "invalid duration format"),
            ("1s s", "invalid duration format"),
            ("1 .0s", "invalid duration format"),
            // Misuse of signs
            ("1.+0s", "invalid duration format"),
            ("-+1s", "invalid duration format"),
            ("+-1s", "invalid duration format"),
            ("--1.0s", "invalid duration format"),
        ];

        for (idx, (input, error)) in error_cases.into_iter().enumerate() {
            let result = Duration::deserialize(serde_json::Value::String(input.to_string()));
            match result {
                Ok(_) => panic!("case {idx} {input} should not be deserialized"),
                Err(e) => assert_eq!(e.to_string(), error, "case {idx} has bad error: {input}"),
            }
        }
    }
}

//! This file is a bit of a hack to implement a general purpose Serde helper for
//! the generated code.
//!
//! The goal is to make it so that we only need to tell serde to use `::tinc::serde_helpers::_` to serialize / deserialize our types.
//! Even if they are nested in Option, Vec, HashMap, etc.
//!
//! It is subject to change at any time.

use std::marker::PhantomData;

use axum::extract::FromRequestParts;
use bytes::Buf;
use headers_accept::Accept;
use http::request::Parts;
use http_body_util::BodyExt;
use mediatype::{MediaType, ReadParams};
use multer::Constraints;
use scuffle_bytes_util::BytesCow;
use serde::de::IntoDeserializer;

use crate::value::{Value, ValueOwned};

pub trait SerdeTransform<T> {
    fn serialize<S>(value: &Self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
        Self: Sized;
}

trait SerdeHelper<T>: Sized {
    fn serialize<S>(value: &Self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer;

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>;
}

impl<T, X> SerdeHelper<X> for Box<T>
where
    T: SerdeHelper<X>,
{
    fn serialize<S>(value: &Self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        <T as SerdeHelper<X>>::serialize(&**value, serializer)
    }

    fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Box::new(<T as SerdeHelper<X>>::deserialize(deserializer)?))
    }
}

trait Cast<T> {
    fn cast(self) -> T;
}

macro_rules! cast_asserts {
    ($helper:ty, $type:ty) => {
        const {
            assert!(
                std::mem::size_of::<$helper>() == std::mem::size_of::<$type>(),
                concat!(
                    "Size of ",
                    stringify!($helper),
                    " must be the same as ",
                    stringify!($type)
                ),
            );
            assert!(
                std::mem::align_of::<$helper>() == std::mem::align_of::<$type>(),
                concat!(
                    "Alignment of ",
                    stringify!($helper),
                    " must be the same as ",
                    stringify!($type)
                ),
            );
        };
    };
}

pub mod well_known {
    use super::{Cast, SerdeHelper};

    #[allow(private_bounds)]
    #[inline(always)]
    pub fn serialize<S, T, X>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: SerdeHelper<X>,
    {
        T::serialize(value, serializer)
    }

    #[allow(private_bounds)]
    #[inline(always)]
    pub fn deserialize<'de, D, T, X>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: SerdeHelper<X>,
    {
        T::deserialize(deserializer)
    }

    #[allow(private_bounds)]
    #[inline(always)]
    pub fn deserialize_non_optional<'de, D, T, X>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: SerdeHelper<X>,
    {
        let t = T::deserialize(deserializer)?;
        Ok(Some(t))
    }

    macro_rules! forward_schema {
        ($ty:ty) => {
            fn always_inline_schema() -> bool {
                <$ty as ::schemars::JsonSchema>::always_inline_schema()
            }

            fn schema_name() -> std::borrow::Cow<'static, str> {
                <$ty as ::schemars::JsonSchema>::schema_name()
            }

            fn schema_id() -> std::borrow::Cow<'static, str> {
                <$ty as ::schemars::JsonSchema>::schema_id()
            }

            fn json_schema(generator: &mut ::schemars::SchemaGenerator) -> ::schemars::Schema {
                <$ty as ::schemars::JsonSchema>::json_schema(generator)
            }

            fn _schemars_private_non_optional_json_schema(
                generator: &mut ::schemars::SchemaGenerator,
            ) -> ::schemars::Schema {
                <$ty as ::schemars::JsonSchema>::_schemars_private_non_optional_json_schema(generator)
            }

            fn _schemars_private_is_option() -> bool {
                <$ty as ::schemars::JsonSchema>::_schemars_private_is_option()
            }
        };
    }

    macro_rules! impl_cast {
        ($raw:ty, $type:ty, $helper:ty $(: $($tt:tt)*)?) => {
            impl<$($($tt)*)?> Cast<$type> for $helper {
                #[inline(always)]
                fn cast(self) -> $type {
                    cast_asserts!($helper, $type);
                    unsafe { std::mem::transmute::<$helper, $type>(self) }
                }
            }

            impl<'a, $($($tt)*)?> Cast<&'a $type> for &'a $helper {
                #[inline(always)]
                fn cast(self) -> &'a $type {
                    cast_asserts!($helper, $type);
                    unsafe { &*(self as *const $helper as *const $type) }
                }
            }

            impl<$($($tt)*)?> Cast<$helper> for $type {
                #[inline(always)]
                fn cast(self) -> $helper {
                    cast_asserts!($helper, $type);
                    unsafe { std::mem::transmute::<$type, $helper>(self) }
                }
            }

            impl<'a, $($($tt)*)?> Cast<&'a $helper> for &'a $type {
                #[inline(always)]
                fn cast(self) -> &'a $helper {
                    cast_asserts!($helper, $type);
                    unsafe { &*(self as *const $type as *const $helper) }
                }
            }

            impl<$($($tt)*)?> SerdeHelper<$raw> for $type {
                #[inline(always)]
                fn serialize<S>(value: &Self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    ::serde::Serialize::serialize(value.cast(), serializer)
                }

                #[inline(always)]
                fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    let t: $helper = ::serde::Deserialize::deserialize(deserializer)?;
                    Ok(t.cast())
                }
            }
        };
    }

    macro_rules! impl_serde_helper {
        ($helper:ident, $type:ty) => {
            #[repr(transparent)]
            pub struct $helper($type);

            impl From<$type> for $helper {
                fn from(value: $type) -> Self {
                    $helper(value)
                }
            }

            impl From<$helper> for $type {
                fn from(value: $helper) -> Self {
                    value.0
                }
            }

            const _: () = {
                impl_cast!($type, $type, $helper);
                impl_cast!($type, Option<$type>, Option<$helper>);
                impl_cast!($type, Vec<$type>, Vec<$helper>);
                impl_cast!($type, std::collections::HashMap<K, $type>, std::collections::HashMap<K, $helper>: K: std::hash::Hash + std::cmp::Eq + serde::Serialize + for<'de> serde::Deserialize<'de>);
                impl_cast!($type, std::collections::BTreeMap<K, $type>, std::collections::BTreeMap<K, $helper>: K: std::cmp::Ord + serde::Serialize + for<'de> serde::Deserialize<'de>);
            };
        };
    }

    impl_serde_helper!(Timestamp, prost_types::Timestamp);

    impl serde::Serialize for Timestamp {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            chrono::DateTime::<chrono::Utc>::from_timestamp(self.0.seconds, self.0.nanos.max(0) as u32)
                .unwrap_or_default()
                .to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)
                .serialize(serializer)
        }
    }

    impl<'de> serde::Deserialize<'de> for Timestamp {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s: std::borrow::Cow<'_, str> = serde::Deserialize::deserialize(deserializer)?;
            let dt = chrono::DateTime::parse_from_rfc3339(&s)
                .map_err(serde::de::Error::custom)?
                .to_utc();
            Ok(Timestamp(prost_types::Timestamp {
                seconds: dt.timestamp(),
                nanos: dt.timestamp_subsec_nanos() as i32,
            }))
        }
    }

    impl schemars::JsonSchema for Timestamp {
        forward_schema!(chrono::DateTime<chrono::Utc>);
    }

    impl_serde_helper!(BytesVecU8, Vec<u8>);

    impl serde::Serialize for BytesVecU8 {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.0.serialize(serializer)
        }
    }

    impl<'de> serde::Deserialize<'de> for BytesVecU8 {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            serde::Deserialize::deserialize(deserializer).map(Self)
        }
    }

    impl schemars::JsonSchema for BytesVecU8 {
        forward_schema!(Vec<u8>);
    }

    impl_serde_helper!(Duration, prost_types::Duration);

    impl schemars::JsonSchema for Duration {
        fn always_inline_schema() -> bool {
            true
        }

        fn schema_id() -> std::borrow::Cow<'static, str> {
            "Duration".into()
        }

        fn schema_name() -> std::borrow::Cow<'static, str> {
            "Duration".into()
        }

        fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
            schemars::json_schema!({
                "type": "string",
                "pattern": r"^\d+(\.\d+)?s$",
            })
        }
    }

    impl serde::Serialize for Duration {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut buffer = itoa::Buffer::new();
            let seconds_str = buffer.format(self.0.seconds);

            if self.0.nanos == 0 {
                let mut output = String::with_capacity(seconds_str.len() + 1);
                output.push_str(seconds_str);
                output.push('s');
                return serializer.serialize_str(&output);
            }

            let mut buffer = ryu::Buffer::new();
            let mut nanos_str = buffer.format(self.0.nanos as f64 / 1_000_000_000.0);

            // Remove "0." prefix from fractional part (e.g., "0.456" -> "456")
            nanos_str = &nanos_str[2..];

            let mut output = String::with_capacity(seconds_str.len() + nanos_str.len() + 2);
            output.push_str(seconds_str);
            output.push('.');
            output.push_str(nanos_str);
            output.push('s');

            serializer.serialize_str(&output)
        }
    }

    impl<'de> serde::Deserialize<'de> for Duration {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s: std::borrow::Cow<'_, str> = serde::Deserialize::deserialize(deserializer)?;

            if !s.ends_with('s') {
                return Err(serde::de::Error::custom("Duration must end with 's'"));
            }

            let trimmed = &s[..s.len() - 1]; // Remove trailing 's'
            let mut iter = trimmed.split('.');

            let seconds = iter
                .next()
                .ok_or_else(|| serde::de::Error::custom("Invalid format"))?
                .parse::<i64>()
                .map_err(serde::de::Error::custom)?;

            let nanos = if let Some(nano_part) = iter.next() {
                let mut nano_str = nano_part.to_string();
                nano_str.push_str("000000000"); // Ensure at least 9 digits
                nano_str.truncate(9); // Trim excess digits
                nano_str.parse::<i32>().map_err(serde::de::Error::custom)?
            } else {
                0
            };

            Ok(Duration(prost_types::Duration { seconds, nanos }))
        }
    }

    impl_serde_helper!(Struct, prost_types::Struct);

    impl serde::Serialize for Struct {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            (&self.0.fields).cast().serialize(serializer)
        }
    }

    impl<'de> serde::Deserialize<'de> for Struct {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let fields: ::std::collections::BTreeMap<String, Value> = serde::Deserialize::deserialize(deserializer)?;
            Ok(Struct(prost_types::Struct { fields: fields.cast() }))
        }
    }

    impl schemars::JsonSchema for Struct {
        forward_schema!(std::collections::BTreeMap<String, Value>);
    }

    impl_serde_helper!(Value, prost_types::Value);

    impl serde::Serialize for Value {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match &self.0.kind {
                None | Some(prost_types::value::Kind::NullValue(_)) => serializer.serialize_none(),
                Some(prost_types::value::Kind::StringValue(s)) => serializer.serialize_str(s),
                Some(prost_types::value::Kind::NumberValue(n)) => {
                    if n.trunc() == *n {
                        if *n < 0.0 {
                            serializer.serialize_i64(*n as i64)
                        } else {
                            serializer.serialize_u64(*n as u64)
                        }
                    } else {
                        serializer.serialize_f64(*n)
                    }
                }
                Some(prost_types::value::Kind::BoolValue(b)) => serializer.serialize_bool(*b),
                Some(prost_types::value::Kind::StructValue(obj)) => (&obj.fields).cast().serialize(serializer),
                Some(prost_types::value::Kind::ListValue(list)) => (&list.values).cast().serialize(serializer),
            }
        }
    }

    impl<'de> serde::Deserialize<'de> for Value {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            struct ValueVisitor;

            impl<'de> serde::de::Visitor<'de> for ValueVisitor {
                type Value = Value;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("a Value")
                }

                fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
                {
                    let mut fields = ::std::collections::BTreeMap::new();
                    while let Some((key, value)) = visitor.next_entry::<String, Value>()? {
                        let key = key.to_string();
                        fields.insert(key, value.cast());
                    }

                    Ok(prost_types::Value {
                        kind: Some(prost_types::value::Kind::StructValue(prost_types::Struct { fields })),
                    }
                    .cast())
                }

                fn visit_seq<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
                where
                    V: serde::de::SeqAccess<'de>,
                {
                    let mut values = Vec::with_capacity(visitor.size_hint().unwrap_or(0));
                    while let Some(value) = visitor.next_element::<Value>()? {
                        values.push(value.cast());
                    }

                    Ok(prost_types::Value {
                        kind: Some(prost_types::value::Kind::ListValue(prost_types::ListValue { values })),
                    }
                    .cast())
                }

                fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(prost_types::Value {
                        kind: Some(prost_types::value::Kind::BoolValue(v)),
                    }
                    .cast())
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(prost_types::Value {
                        kind: Some(prost_types::value::Kind::StringValue(v.to_string())),
                    }
                    .cast())
                }

                fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(prost_types::Value {
                        kind: Some(prost_types::value::Kind::NumberValue(v)),
                    }
                    .cast())
                }

                fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    self.visit_f64(v as f64)
                }

                fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    self.visit_f64(v as f64)
                }

                fn visit_none<E>(self) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    Ok(prost_types::Value {
                        kind: Some(prost_types::value::Kind::NullValue(prost_types::NullValue::NullValue as i32)),
                    }
                    .cast())
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
                    Ok(prost_types::Value {
                        kind: Some(prost_types::value::Kind::NullValue(prost_types::NullValue::NullValue as i32)),
                    }
                    .cast())
                }
            }

            deserializer.deserialize_any(ValueVisitor)
        }
    }

    impl schemars::JsonSchema for Value {
        forward_schema!(serde_json::Value);
    }

    impl_serde_helper!(Empty, ());

    impl schemars::JsonSchema for Empty {
        forward_schema!(());
    }

    impl serde::Serialize for Empty {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            <() as ::serde::Serialize>::serialize(&self.0, serializer)
        }
    }

    impl<'de> serde::Deserialize<'de> for Empty {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            <() as ::serde::Deserialize<'de>>::deserialize(deserializer).map(Self)
        }
    }

    impl_serde_helper!(List, prost_types::ListValue);

    impl schemars::JsonSchema for List {
        forward_schema!(Vec<Value>);
    }

    impl serde::Serialize for List {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            (&self.0.values).cast().serialize(serializer)
        }
    }

    impl<'de> serde::Deserialize<'de> for List {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let values: Vec<Value> = serde::Deserialize::deserialize(deserializer)?;
            Ok(List(prost_types::ListValue { values: values.cast() }))
        }
    }
}

pub struct Enum<T>(PhantomData<T>);

const _: () = {
    #[repr(transparent)]
    struct HelperEnum<E>(i32, PhantomData<E>);

    macro_rules! impl_cast {
        ($type:ty, $helper:ty $(: $($tt:tt)*)?) => {
            impl<E, $($($tt)*)?> Cast<$type> for $helper {
                #[inline(always)]
                fn cast(self) -> $type {
                    cast_asserts!($helper, $type);
                    unsafe { std::mem::transmute::<$helper, $type>(self) }
                }
            }

            impl<'a, E, $($($tt)*)?> Cast<&'a $type> for &'a $helper {
                #[inline(always)]
                fn cast(self) -> &'a $type {
                    cast_asserts!($helper, $type);
                    unsafe { &*(self as *const $helper as *const $type) }
                }
            }

            impl<E, $($($tt)*)?> Cast<$helper> for $type {
                #[inline(always)]
                fn cast(self) -> $helper {
                    cast_asserts!($type, $helper);
                    unsafe { std::mem::transmute::<$type, $helper>(self) }
                }
            }

            impl<'a, E, $($($tt)*)?> Cast<&'a $helper> for &'a $type {
                #[inline(always)]
                fn cast(self) -> &'a $helper {
                    cast_asserts!($type, $helper);
                    unsafe { &*(self as *const $type as *const $helper) }
                }
            }

            impl<E, $($($tt)*)?> SerdeHelper<Enum<E>> for $type
            where
                E: serde::Serialize + TryFrom<i32> + for<'de> serde::Deserialize<'de> + Into<i32>,
                E::Error: std::fmt::Display,
            {
                #[inline(always)]
                fn serialize<S>(value: &Self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    let cast: &$helper = value.cast();
                    ::serde::Serialize::serialize(cast, serializer)
                }

                #[inline(always)]
                fn deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    let t: $helper = ::serde::Deserialize::deserialize(deserializer)?;
                    Ok(t.cast())
                }
            }
        };
    }

    const _: () = {
        impl_cast!(i32, HelperEnum<E>);
        impl_cast!(Option<i32>, Option<HelperEnum<E>>);
        impl_cast!(Vec<i32>, Vec<HelperEnum<E>>);
        impl_cast!(std::collections::HashMap<K, i32>, std::collections::HashMap<K, HelperEnum<E>>: K: std::hash::Hash + std::cmp::Eq + serde::Serialize + for<'de> serde::Deserialize<'de>);
        impl_cast!(std::collections::BTreeMap<K, i32>, std::collections::BTreeMap<K, HelperEnum<E>>: K: std::cmp::Ord + serde::Serialize + for<'de> serde::Deserialize<'de>);
    };

    impl<E> serde::Serialize for HelperEnum<E>
    where
        E: serde::Serialize + TryFrom<i32>,
        E::Error: std::fmt::Display,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            E::try_from(self.0).map_err(serde::ser::Error::custom)?.serialize(serializer)
        }
    }

    impl<'de, E> serde::Deserialize<'de> for HelperEnum<E>
    where
        E: serde::Deserialize<'de> + Into<i32>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let e = E::deserialize(deserializer)?;
            Ok(HelperEnum(e.into(), PhantomData))
        }
    }
};

impl<E> Enum<E> {
    #[allow(private_bounds)]
    #[inline(always)]
    pub fn serialize<I, S>(value: &I, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        I: SerdeHelper<Enum<E>>,
    {
        I::serialize(value, serializer)
    }

    #[allow(private_bounds)]
    #[inline(always)]
    pub fn deserialize<'de, I, D>(deserializer: D) -> Result<I, D::Error>
    where
        D: serde::Deserializer<'de>,
        I: SerdeHelper<Enum<E>>,
    {
        I::deserialize(deserializer)
    }
}

pub fn deserialize_non_omitable<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    T::deserialize(deserializer)
}

pub fn deserialize_non_null_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let value = T::deserialize(deserializer)?;
    Ok(Some(value))
}

pub fn schemars_non_omitable(schema: &mut ::schemars::Schema) {
    let Some(as_object) = schema.as_object_mut() else {
        return;
    };

    if let Some(ty) = as_object.get_mut("type") {
        match ty {
            serde_json::Value::String(s) => {
                if s != "null" {
                    *ty = serde_json::Value::Array(vec![
                        serde_json::Value::String(s.clone()),
                        serde_json::Value::String("null".to_string()),
                    ]);
                }
            }
            serde_json::Value::Array(a) => {
                if a.iter().all(|v| v != &serde_json::Value::String("null".to_string())) {
                    a.push(serde_json::Value::String("null".to_string()));
                }
            }
            _ => {}
        }
    } else if let Some(ty) = as_object.get_mut("oneOf") {
        let array = ty.as_array_mut().expect("oneOf must be an array");
        let null_type = serde_json::json!({
            "type": "null",
        });
        if !array.iter().any(|v| v == &null_type) {
            array.push(null_type);
        }
    }
}

pub async fn parse_path(parts: &mut Parts) -> Result<Value<'static>, axum::response::Response> {
    match axum::extract::Path::<ValueOwned>::from_request_parts(parts, &()).await {
        Ok(value) => Ok(value.0.0),
        Err(rejection) => todo!("todo handle error: {:?}", rejection),
    }
}

pub async fn parse_query(parts: &mut Parts) -> Result<Value<'static>, axum::response::Response> {
    match axum::extract::Query::<ValueOwned>::from_request_parts(parts, &()).await {
        Ok(value) => Ok(value.0.0),
        Err(rejection) => todo!("todo handle error: {:?}", rejection),
    }
}

pub fn decode_input<'de, T>(input: Value<'de>) -> Result<T, axum::response::Response>
where
    T: serde::Deserialize<'de>,
{
    let result: Result<T, serde::de::value::Error> = serde::Deserialize::deserialize(input.into_deserializer());
    match result {
        Ok(value) => Ok(value),
        Err(err) => todo!("todo handle error: {:?}", err),
    }
}

pub mod header_decode {
    use std::collections::HashMap;

    use mediatype::MediaType;
    use scuffle_bytes_util::StringCow;

    use crate::value::{Key, Value, ValueOwned};

    pub fn form_url_encoded<'a>(
        headers: &'a http::HeaderMap,
        header_name: &'static str,
        field_name: &'static str,
    ) -> Result<Value<'a>, axum::response::Response> {
        let mut values = Vec::new();
        for value in headers.get_all(header_name) {
            let header_str = match value.to_str() {
                Ok(s) => s,
                Err(err) => todo!("todo handle error: {:?}", err),
            };

            match serde_urlencoded::from_str(header_str) {
                Ok(ValueOwned(value)) => values.push(value),
                Err(err) => todo!("todo handle error: {:?}", err),
            }
        }

        let mut object = HashMap::new();
        match values.len() {
            0 => {}
            1 => {
                object.insert(Key::String(StringCow::from_static(field_name)), values.remove(0));
            }
            _ => {
                object.insert(Key::String(StringCow::from_static(field_name)), Value::Array(values));
            }
        }

        Ok(Value::Map(object))
    }

    pub fn json(
        headers: &http::HeaderMap,
        header_name: &'static str,
        field_name: &'static str,
    ) -> Result<Value<'static>, axum::response::Response> {
        let mut values = Vec::new();
        for value in headers.get_all(header_name) {
            let header_str = match value.to_str() {
                Ok(s) => s,
                Err(err) => todo!("todo handle error: {:?}", err),
            };

            match serde_json::from_str(header_str) {
                Ok(ValueOwned(value)) => values.push(value),
                Err(err) => todo!("todo handle error: {:?}", err),
            }
        }

        let mut object = HashMap::new();
        match values.len() {
            0 => {}
            1 => {
                object.insert(Key::String(StringCow::from_static(field_name)), values.remove(0));
            }
            _ => {
                object.insert(Key::String(StringCow::from_static(field_name)), Value::Array(values));
            }
        }

        Ok(Value::Map(object))
    }

    pub fn text<'a>(
        headers: &'a http::HeaderMap,
        header_name: &'static str,
        field_name: &'static str,
        delimiter: Option<&'static str>,
    ) -> Result<Value<'a>, axum::response::Response> {
        let mut values = Vec::new();
        for value in headers.get_all(header_name) {
            let header_str = match value.to_str() {
                Ok(s) => s,
                Err(err) => todo!("todo handle error: {:?}", err),
            };

            if let Some(delimiter) = delimiter {
                values.extend(header_str.split(delimiter).map(|s| Value::String(s.into())));
            } else {
                values.push(Value::String(header_str.into()));
            }
        }

        let mut object = HashMap::new();
        match values.len() {
            0 => {}
            1 => {
                object.insert(Key::String(StringCow::from_static(field_name)), values.remove(0));
            }
            _ => {
                object.insert(Key::String(StringCow::from_static(field_name)), Value::Array(values));
            }
        }

        Ok(Value::Map(object))
    }

    pub fn content_type(headers: &http::HeaderMap) -> Result<Option<MediaType<'_>>, axum::response::Response> {
        let Some(content_type) = headers.get("content-type") else {
            return Ok(None);
        };

        let content_type_str = match content_type.to_str() {
            Ok(s) => s,
            Err(err) => todo!("todo handle error: {:?}", err),
        };

        let media_type = match MediaType::parse(content_type_str) {
            Ok(media_type) => media_type,
            Err(err) => todo!("todo handle error: {:?}", err),
        };

        Ok(Some(media_type))
    }
}

pub fn no_valid_content_type(content_type: &MediaType<'_>, content_types: &[&Accept]) -> axum::response::Response {
    todo!(
        "todo handle error: no valid content type: {:?} - {:?}",
        content_type,
        content_types
    )
}

pub fn bad_request_not_object(body: Value<'_>) -> axum::response::Response {
    todo!("todo handle error: body is not an object: {:?}", body)
}

async fn read_body(body: axum::body::Body) -> Result<impl bytes::Buf, axum::response::Response> {
    match body.collect().await {
        Ok(bytes) => Ok(bytes.aggregate()),
        Err(err) => todo!("todo handle error: {:?}", err),
    }
}

async fn parse_body_string(
    content_type: &MediaType<'_>,
    body: axum::body::Body,
) -> Result<String, axum::response::Response> {
    let charset = content_type
        .params()
        .find(|(k, _)| k == "charset")
        .map(|(_, v)| v.as_str())
        .unwrap_or("utf-8");

    fn parse_utf8(bytes: Vec<u8>) -> Option<String> {
        String::from_utf8(bytes).ok()
    }

    fn parse_utf16(bytes: Vec<u8>) -> Option<String> {
        if bytes.len() % 2 != 0 {
            return None;
        }

        String::from_utf16({
            let ptr = bytes.as_ptr().cast::<u16>();
            let len = bytes.len() / 2;
            unsafe { std::slice::from_raw_parts(ptr, len) }
        })
        .ok()
    }

    let parse_fn = if charset.eq_ignore_ascii_case("utf-8") || charset.eq_ignore_ascii_case("us-ascii") {
        parse_utf8
    } else if charset.eq_ignore_ascii_case("utf-16") {
        parse_utf16
    } else {
        todo!("todo handle error: unsupported charset")
    };

    let body_str = match read_body(body).await {
        Ok(mut bytes) => {
            let mut data = vec![0; bytes.remaining()];
            bytes.copy_to_slice(&mut data);
            parse_fn(data)
        }
        Err(err) => return Err(err),
    };

    let Some(body_str) = body_str else {
        todo!("todo handle error: invalid body")
    };

    Ok(body_str)
}

pub async fn parse_body(
    content_type: &MediaType<'_>,
    body: axum::body::Body,
) -> Result<Value<'static>, axum::response::Response> {
    const JSON: MediaType<'_> = MediaType::new(mediatype::names::APPLICATION, mediatype::names::JSON);
    const FORM_URL_ENCODED: MediaType<'_> = MediaType::new(
        mediatype::names::APPLICATION,
        mediatype::Name::new_unchecked("x-www-form-urlencoded"),
    );
    const TEXT: MediaType<'_> = MediaType::new(mediatype::names::TEXT, mediatype::names::PLAIN);
    const MULTIPART: MediaType<'_> = MediaType::new(mediatype::names::MULTIPART, mediatype::names::FORM_DATA);

    let essence = content_type.essence();
    if essence == JSON {
        match serde_json::from_str(&parse_body_string(content_type, body).await?) {
            Ok(ValueOwned(value)) => Ok(value),
            Err(err) => todo!("todo handle error: {:?}", err),
        }
    } else if essence == FORM_URL_ENCODED {
        let body_str = parse_body_string(content_type, body).await?;
        match serde_urlencoded::from_str(&body_str) {
            Ok(ValueOwned(value)) => Ok(value),
            Err(err) => todo!("todo handle error: {:?}", err),
        }
    } else if essence == TEXT {
        let body_str = parse_body_string(content_type, body).await?;
        Ok(Value::String(body_str.into()))
    } else if essence == MULTIPART {
        let Some(boundary) = content_type.params().find(|(k, _)| k == "boundary").map(|(_, v)| v.as_str()) else {
            todo!("todo handle error: missing boundary")
        };

        let mut multipart = multer::Multipart::with_constraints(body.into_data_stream(), boundary, Constraints::new());
        let mut form = Vec::new();
        while let Some(field) = multipart.next_field().await.transpose() {
            let field = match field {
                Ok(field) => field,
                Err(err) => todo!("todo handle error: {:?}", err),
            };

            let Some(name) = field.name().map(|s| s.to_owned()) else {
                todo!("todo handle error: missing name")
            };

            let value = match field.bytes().await {
                Ok(value) => value,
                Err(err) => todo!("todo handle error: {:?}", err),
            };
            form.push((name, value));
        }

        match multipart::parse_form_fields(form) {
            Ok(form) => Ok(form),
            Err(err) => todo!("todo handle error: {:?}", err),
        }
    } else {
        let mut body_bytes = read_body(body).await?;
        Ok(Value::Bytes(BytesCow::from_bytes(
            body_bytes.copy_to_bytes(body_bytes.remaining()),
        )))
    }
}

mod multipart {
    use std::collections::HashMap;

    use bytes::Bytes;
    use scuffle_bytes_util::{BytesCow, StringCow};

    use crate::value::{Key, Value};

    fn parse_key(key: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut chars = key.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '[' => {
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                    while let Some(&next) = chars.peek() {
                        chars.next(); // consume
                        if next == ']' {
                            break;
                        } else {
                            current.push(next);
                        }
                    }
                    parts.push(current.clone());
                    current.clear();
                }
                _ => current.push(c),
            }
        }

        if !current.is_empty() {
            parts.push(current);
        }

        parts
    }

    fn insert_path(root: &mut Value<'static>, path: Vec<String>, value: Bytes) -> Result<(), &'static str> {
        let mut current = root;
        let length = path.len();

        for (i, key) in path.into_iter().enumerate() {
            let is_last = i == length - 1;

            match current {
                Value::Map(map) => {
                    if key.is_empty() {
                        return Err("empty key in object");
                    }
                    if is_last {
                        map.insert(
                            Key::String(StringCow::from_string(key)),
                            Value::Bytes(BytesCow::from_bytes(value)),
                        );
                        return Ok(());
                    }

                    current = map
                        .entry(Key::String(StringCow::from_string(key)))
                        .or_insert_with(|| Value::Map(HashMap::new()));
                }
                Value::Array(arr) => {
                    if key.is_empty() {
                        if is_last {
                            arr.push(Value::Bytes(BytesCow::from_bytes(value)));
                            return Ok(());
                        } else {
                            arr.push(Value::Map(HashMap::new()));
                            current = arr.last_mut().unwrap();
                        }
                    } else if let Ok(index) = key.parse::<usize>() {
                        if arr.len() <= index {
                            arr.resize_with(index + 1, || Value::Null);
                        }

                        if is_last {
                            arr[index] = Value::Bytes(BytesCow::from_bytes(value));
                            return Ok(());
                        } else {
                            if matches!(arr[index], Value::Null) {
                                arr[index] = Value::Map(HashMap::new());
                            }
                            current = &mut arr[index];
                        }
                    } else {
                        return Err("invalid array key");
                    }
                }
                _ => return Err("type conflict during insert"),
            }

            // Convert from string to object/array if needed
            if !is_last && matches!(current, Value::Null) {
                *current = Value::Map(HashMap::new());
            }
        }

        Ok(())
    }

    pub fn parse_form_fields(fields: Vec<(String, Bytes)>) -> Result<Value<'static>, &'static str> {
        let mut root = Value::Map(HashMap::new());

        for (key, value) in fields {
            insert_path(&mut root, parse_key(&key), value)?;
        }

        Ok(root)
    }
}

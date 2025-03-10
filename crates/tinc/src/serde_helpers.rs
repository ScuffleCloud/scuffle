//! This file is a bit of a hack to implement a general purpose Serde helper for
//! the generated code.
//!
//! The goal is to make it so that we only need to tell serde to use `::tinc::serde_helpers::_` to serialize / deserialize our types.
//! Even if they are nested in Option, Vec, HashMap, etc.
//!
//! It is subject to change at any time.

use std::collections::HashMap;
use std::marker::PhantomData;

macro_rules! defer_schemars_impl {
    ($defer:ty) => {
        // fn is_referenceable() -> bool {
        //     <$defer as ::schemars::JsonSchema>::is_referenceable()
        // }

        // fn schema_name() -> ::std::string::String {
        //     <$defer as ::schemars::JsonSchema>::schema_name()
        // }

        // fn schema_id() -> ::std::borrow::Cow<'static, str> {
        //     <$defer as ::schemars::JsonSchema>::schema_id()
        // }

        // fn json_schema(generator: &mut ::schemars::r#gen::SchemaGenerator) -> ::schemars::schema::Schema {
        //     <$defer as ::schemars::JsonSchema>::json_schema(generator)
        // }

        // fn _schemars_private_non_optional_json_schema(
        //     generator: &mut ::schemars::r#gen::SchemaGenerator,
        // ) -> ::schemars::schema::Schema {
        //     <$defer as ::schemars::JsonSchema>::_schemars_private_non_optional_json_schema(generator)
        // }

        // fn _schemars_private_is_option() -> bool {
        //     <$defer as ::schemars::JsonSchema>::_schemars_private_is_option()
        // }
    };
}

pub mod primitive_types {
    macro_rules! impl_primitive_type {
        ($name:ident, $type:ty) => {
            pub struct $name;

            // impl ::schemars::JsonSchema for $name {
            //     defer_schemars_impl!($type);
            // }
        };
    }

    impl_primitive_type!(I32, ::core::primitive::i32);
    impl_primitive_type!(I64, ::core::primitive::i64);
    impl_primitive_type!(U32, ::core::primitive::u32);
    impl_primitive_type!(U64, ::core::primitive::u64);
    impl_primitive_type!(F32, ::core::primitive::f32);
    impl_primitive_type!(F64, ::core::primitive::f64);
    impl_primitive_type!(String, ::std::string::String);
    // impl_primitive_type!(Bytes, ::std::vec::Vec<u8>);
}

pub struct SchemaMap<K, V>(PhantomData<HashMap<K, V>>);

// impl<K, V> schemars::JsonSchema for SchemaMap<K, V>
// where
//     K: schemars::JsonSchema,
//     V: schemars::JsonSchema,
// {
//     defer_schemars_impl!(::std::collections::HashMap<K, V>);
// }

pub struct SchemaList<T>(PhantomData<Vec<T>>);

// impl<T> schemars::JsonSchema for SchemaList<T>
// where
//     T: schemars::JsonSchema,
// {
//     defer_schemars_impl!(::std::vec::Vec<T>);
// }

pub struct SchemaOptional<T>(PhantomData<Option<T>>);

// impl<T> schemars::JsonSchema for SchemaOptional<T>
// where
//     T: schemars::JsonSchema,
// {
//     defer_schemars_impl!(::core::option::Option<T>);
// }

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
        debug_assert!(
            std::mem::size_of::<$helper>() == std::mem::size_of::<$type>(),
            "Size of {} must be the same as {}",
            std::any::type_name::<$helper>(),
            std::any::type_name::<$type>(),
        );
        debug_assert!(
            std::mem::align_of::<$helper>() == std::mem::align_of::<$type>(),
            "Alignment of {} must be the same as {}",
            std::any::type_name::<$helper>(),
            std::any::type_name::<$type>(),
        );
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
                    cast_asserts!(&$helper, &$type);
                    unsafe { std::mem::transmute::<&'a $helper, &'a $type>(self) }
                }
            }

            impl<$($($tt)*)?> Cast<$helper> for $type {
                #[inline(always)]
                fn cast(self) -> $helper {
                    cast_asserts!($type, $helper);
                    unsafe { std::mem::transmute::<$type, $helper>(self) }
                }
            }

            impl<'a, $($($tt)*)?> Cast<&'a $helper> for &'a $type {
                #[inline(always)]
                fn cast(self) -> &'a $helper {
                    cast_asserts!(&$type, &$helper);
                    unsafe { std::mem::transmute::<&'a $type, &'a $helper>(self) }
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

    // impl schemars::JsonSchema for Timestamp {
    //     defer_schemars_impl!(::chrono::DateTime<chrono::Utc>);
    // }

    impl_serde_helper!(Duration, prost_types::Duration);

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

    // impl schemars::JsonSchema for Duration {
    //     fn is_referenceable() -> bool {
    //         false
    //     }

    //     fn schema_name() -> String {
    //         "Duration".to_string()
    //     }

    //     fn schema_id() -> std::borrow::Cow<'static, str> {
    //         std::borrow::Cow::Borrowed("google.protobuf.Duration")
    //     }

    //     fn json_schema(_: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
    //         schemars::schema::SchemaObject {
    //             instance_type: Some(schemars::schema::InstanceType::String.into()),
    //             ..Default::default()
    //         }
    //         .into()
    //     }
    // }

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

    // impl schemars::JsonSchema for Struct {
    //     defer_schemars_impl!(::std::collections::HashMap<::std::string::String, Value>);
    // }

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

    // impl schemars::JsonSchema for Value {
    //     defer_schemars_impl!(::serde_json::Value);
    // }

    impl_serde_helper!(Empty, ());

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

    // impl schemars::JsonSchema for Empty {
    //     defer_schemars_impl!(());
    // }

    impl_serde_helper!(List, prost_types::ListValue);

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

    // impl schemars::JsonSchema for List {
    //     defer_schemars_impl!(::std::vec::Vec<Value>);
    // }
}

pub struct Enum<T>(PhantomData<T>);

// impl<T: schemars::JsonSchema> schemars::JsonSchema for Enum<T> {
//     defer_schemars_impl!(T);
// }

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
                    cast_asserts!(&$helper, &$type);
                    unsafe { std::mem::transmute::<&'a $helper, &'a $type>(self) }
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
                    cast_asserts!(&$type, &$helper);
                    unsafe { std::mem::transmute::<&'a $type, &'a $helper>(self) }
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

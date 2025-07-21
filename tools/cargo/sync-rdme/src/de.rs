use std::{convert::Infallible, fmt, marker::PhantomData, str::FromStr};

use serde::{de::Visitor, Deserialize, Deserializer};

pub(super) fn bool_or_map<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de> + Default,
    D: Deserializer<'de>,
{
    struct BoolOrMap<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for BoolOrMap<T>
    where
        T: Deserialize<'de> + Default,
    {
        type Value = Option<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a boolean or a map")
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v.then(T::default))
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: serde::de::MapAccess<'de>,
        {
            let v = T::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
            Ok(Some(v))
        }
    }

    let map = deserializer.deserialize_any(BoolOrMap(PhantomData))?;
    Ok(map)
}

pub(super) fn string_or_seq<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrSeq;

    impl<'de> Visitor<'de> for StringOrSeq {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a string or a seq")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut values = vec![];
            while let Some(value) = seq.next_element::<String>()? {
                values.push(value);
            }
            Ok(values)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![v.to_owned()])
        }
    }

    let seq = deserializer.deserialize_any(StringOrSeq)?;
    Ok(seq)
}

pub(super) fn string_or_map_or_seq<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Infallible>,
    D: Deserializer<'de>,
{
    struct StringOrMapOrSeq<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for StringOrMapOrSeq<T>
    where
        T: Deserialize<'de> + FromStr<Err = Infallible>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a string or a map or a seq")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            struct StringOrMap<T>(T);
            impl<'de, T> Deserialize<'de> for StringOrMap<T>
            where
                T: Deserialize<'de> + FromStr<Err = Infallible>,
            {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    string_or_map(deserializer).map(Self)
                }
            }

            let mut values = vec![];
            while let Some(value) = seq.next_element::<StringOrMap<T>>()? {
                values.push(value.0);
            }
            Ok(values)
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![v.parse().unwrap()])
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: serde::de::MapAccess<'de>,
        {
            let v = T::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
            Ok(vec![v])
        }
    }

    let map = deserializer.deserialize_any(StringOrMapOrSeq(PhantomData))?;
    Ok(map)
}

pub(super) fn string_or_map<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Infallible>,
    D: Deserializer<'de>,
{
    struct StringOrMap<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for StringOrMap<T>
    where
        T: Deserialize<'de> + FromStr<Err = Infallible>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a string or a map")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v.parse().unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: serde::de::MapAccess<'de>,
        {
            let v = T::deserialize(serde::de::value::MapAccessDeserializer::new(map))?;
            Ok(v)
        }
    }

    let map = deserializer.deserialize_any(StringOrMap(PhantomData))?;
    Ok(map)
}

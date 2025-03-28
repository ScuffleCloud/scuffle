//! AMF0 value types.

use std::collections::BTreeMap;

use serde::ser::{SerializeMap, SerializeSeq};

/// Represents any AMF0 object.
pub type Amf0Object = BTreeMap<String, Amf0Value>;

/// Represents any AMF0 value.
#[derive(Debug, PartialEq, Clone)]
pub enum Amf0Value {
    /// AMF0 Number.
    Number(f64),
    /// AMF0 Boolean.
    Boolean(bool),
    /// AMF0 String.
    String(String),
    /// AMF0 Object.
    Object(Amf0Object),
    /// AMF0 Null.
    Null,
    /// AMF0 Array.
    Array(Vec<Amf0Value>),
}

impl From<f64> for Amf0Value {
    fn from(value: f64) -> Self {
        Amf0Value::Number(value)
    }
}

impl From<bool> for Amf0Value {
    fn from(value: bool) -> Self {
        Amf0Value::Boolean(value)
    }
}

impl From<String> for Amf0Value {
    fn from(value: String) -> Self {
        Amf0Value::String(value)
    }
}

impl From<&str> for Amf0Value {
    fn from(value: &str) -> Self {
        Amf0Value::String(value.to_owned())
    }
}

impl From<Amf0Object> for Amf0Value {
    fn from(value: Amf0Object) -> Self {
        Amf0Value::Object(value)
    }
}

impl From<Vec<Amf0Value>> for Amf0Value {
    fn from(value: Vec<Amf0Value>) -> Self {
        Amf0Value::Array(value)
    }
}

impl serde::ser::Serialize for Amf0Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Amf0Value::Number(v) => serializer.serialize_f64(*v),
            Amf0Value::Boolean(v) => serializer.serialize_bool(*v),
            Amf0Value::String(v) => serializer.serialize_str(v),
            Amf0Value::Object(v) => {
                let mut map = serializer.serialize_map(Some(v.len()))?;

                for (key, value) in v {
                    map.serialize_entry(key, value)?;
                }

                map.end()
            }
            Amf0Value::Null => serializer.serialize_none(),
            Amf0Value::Array(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;

                for value in v {
                    seq.serialize_element(value)?;
                }

                seq.end()
            }
        }
    }
}

impl<'de> serde::de::Deserialize<'de> for Amf0Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Amf0ValueVisitor;

        impl<'de> serde::de::Visitor<'de> for Amf0ValueVisitor {
            type Value = Amf0Value;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an AMF0 value")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::Boolean(v))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::Number(v))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::String(v.to_owned()))
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::String(v))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::Null)
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Amf0Value::Null)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                serde::Deserialize::deserialize(deserializer)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(value) = seq.next_element()? {
                    vec.push(value);
                }

                Ok(Amf0Value::Array(vec))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut object = BTreeMap::new();

                while let Some((key, value)) = map.next_entry()? {
                    object.insert(key, value);
                }

                Ok(Amf0Value::Object(object))
            }
        }

        deserializer.deserialize_any(Amf0ValueVisitor)
    }
}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use super::Amf0Value;
    use crate::{Amf0Marker, from_bytes};

    #[test]
    fn string() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::String as u8,
            0, 3, // length
            b'a', b'b', b'c',
        ];

        let value: Amf0Value = from_bytes(&bytes).unwrap();
        assert_eq!(value, Amf0Value::String("abc".to_owned()));
    }

    #[test]
    fn bool() {
        let bytes = [Amf0Marker::Boolean as u8, 0];

        let value: Amf0Value = from_bytes(&bytes).unwrap();
        assert_eq!(value, Amf0Value::Boolean(false));
    }

    #[test]
    fn object() {
        #[rustfmt::skip]
        let bytes = [
            Amf0Marker::Object as u8,
            0, 1,
            b'a',
            Amf0Marker::Boolean as u8,
            1,
            0, 0, Amf0Marker::ObjectEnd as u8,
        ];

        let value: Amf0Value = from_bytes(&bytes).unwrap();
        assert_eq!(
            value,
            Amf0Value::Object([("a".to_owned(), Amf0Value::Boolean(true))].into_iter().collect())
        );
    }
}

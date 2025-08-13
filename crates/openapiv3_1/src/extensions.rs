//! Implements [OpenAPI Extensions][extensions].
//!
//! [extensions]: https://spec.openapis.org/oas/latest.html#specification-extensions
use std::ops::{Deref, DerefMut};

use indexmap::IndexMap;

const EXTENSION_PREFIX: &str = "x-";

/// Additional [data for extending][extensions] the OpenAPI specification.
///
/// [extensions]: https://spec.openapis.org/oas/latest.html#specification-extensions
#[derive(Default, serde_derive::Serialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Extensions {
    #[serde(flatten)]
    extensions: IndexMap<String, serde_json::Value>,
}

impl Extensions {
    /// Create a new extension from an iterator
    pub fn new<K: Into<String>, V: Into<serde_json::Value>>(items: impl IntoIterator<Item = (K, V)>) -> Self {
        items.into_iter().fold(Self::default(), |this, (k, v)| this.add(k, v))
    }

    /// Merge other [`Extensions`] into _`self`_.
    pub fn merge(&mut self, other: Extensions) {
        self.extensions.extend(other.extensions);
    }

    /// Add an extension to the list
    pub fn add(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        let mut key = key.into();
        if !key.starts_with(EXTENSION_PREFIX) {
            key = format!("{EXTENSION_PREFIX}{key}");
        }
        self.extensions.insert(key, value.into());
        self
    }
}

impl Deref for Extensions {
    type Target = IndexMap<String, serde_json::Value>;

    fn deref(&self) -> &Self::Target {
        &self.extensions
    }
}

impl DerefMut for Extensions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.extensions
    }
}

impl<K, V> FromIterator<(K, V)> for Extensions
where
    K: Into<String>,
    V: Into<serde_json::Value>,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self::new(iter)
    }
}

impl From<Extensions> for IndexMap<String, serde_json::Value> {
    fn from(value: Extensions) -> Self {
        value.extensions
    }
}

impl<'de> serde::de::Deserialize<'de> for Extensions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let extensions: IndexMap<String, _> = IndexMap::deserialize(deserializer)?;
        let extensions = extensions
            .into_iter()
            .filter(|(k, _)| k.starts_with(EXTENSION_PREFIX))
            .collect();
        Ok(Self { extensions })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn extensions_builder() {
        let expected = json!("value");
        let extensions = Extensions::default()
            .add("x-some-extension", expected.clone())
            .add("another-extension", expected.clone());

        let value = serde_json::to_value(&extensions).unwrap();
        assert_eq!(value.get("x-some-extension"), Some(&expected));
        assert_eq!(value.get("x-another-extension"), Some(&expected));
    }

    #[test]
    fn extensions_from_iter() {
        let expected = json!("value");
        let extensions: Extensions = [
            ("x-some-extension", expected.clone()),
            ("another-extension", expected.clone()),
        ]
        .into_iter()
        .collect();

        assert_eq!(extensions.get("x-some-extension"), Some(&expected));
        assert_eq!(extensions.get("x-another-extension"), Some(&expected));
    }
}

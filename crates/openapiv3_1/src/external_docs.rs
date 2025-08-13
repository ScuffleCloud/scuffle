//! Implements [OpenAPI External Docs Object][external_docs] types.
//!
//! [external_docs]: https://spec.openapis.org/oas/latest.html#xml-object
use serde_derive::{Deserialize, Serialize};

use super::extensions::Extensions;

/// Reference of external resource allowing extended documentation.
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
#[builder(on(_, into))]
pub struct ExternalDocs {
    /// Target url for external documentation location.
    pub url: String,
    /// Additional description supporting markdown syntax of the external documentation.
    #[serde(default)]
    pub description: Option<String>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", flatten, default)]
    pub extensions: Option<Extensions>,
}

impl ExternalDocs {
    /// Construct a new [`ExternalDocs`].
    ///
    /// Function takes target url argument for the external documentation location.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use openapiv3_1::external_docs::ExternalDocs;
    /// let external_docs = ExternalDocs::new("https://pet-api.external.docs");
    /// ```
    pub fn new<S: AsRef<str>>(url: S) -> Self {
        Self {
            url: url.as_ref().to_string(),
            ..Default::default()
        }
    }
}

impl<S: external_docs_builder::IsComplete> From<ExternalDocsBuilder<S>> for ExternalDocs {
    fn from(builder: ExternalDocsBuilder<S>) -> Self {
        builder.build()
    }
}

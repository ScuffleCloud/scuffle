//! Implements [OpenAPI Example Object][example] can be used to define examples for [`Response`][response]s and
//! [`RequestBody`][request_body]s.
//!
//! [example]: https://spec.openapis.org/oas/latest.html#example-object
//! [response]: response/struct.Response.html
//! [request_body]: request_body/struct.RequestBody.html
use serde_derive::{Deserialize, Serialize};

use super::RefOr;

/// Implements [OpenAPI Example Object][example].
///
/// Example is used on path operations to describe possible response bodies.
///
/// [example]: https://spec.openapis.org/oas/latest.html#example-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
#[builder(on(_, into))]
pub struct Example {
    /// Short description for the [`Example`].
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub summary: String,

    /// Long description for the [`Example`]. Value supports markdown syntax for rich text
    /// representation.
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub description: String,

    /// Embedded literal example value. [`Example::value`] and [`Example::external_value`] are
    /// mutually exclusive.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub value: Option<serde_json::Value>,

    /// An URI that points to a literal example value. [`Example::external_value`] provides the
    /// capability to references an example that cannot be easily included in JSON or YAML.
    /// [`Example::value`] and [`Example::external_value`] are mutually exclusive.
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub external_value: String,
}

impl Example {
    /// Construct a new empty [`Example`]. This is effectively same as calling
    /// [`Example::default`].
    pub fn new() -> Self {
        Self::default()
    }
}

impl<S: example_builder::IsComplete> From<ExampleBuilder<S>> for Example {
    fn from(builder: ExampleBuilder<S>) -> Self {
        builder.build()
    }
}

impl<S: example_builder::IsComplete> From<ExampleBuilder<S>> for RefOr<Example> {
    fn from(builder: ExampleBuilder<S>) -> Self {
        Self::T(builder.build())
    }
}

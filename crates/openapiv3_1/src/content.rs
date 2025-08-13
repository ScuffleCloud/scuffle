//! Implements content object for request body and response.
use indexmap::IndexMap;
use serde_json::Value;

use super::encoding::Encoding;
use super::example::Example;
use super::extensions::Extensions;
use super::{RefOr, Schema};

/// Content holds request body content or response content.
///
/// [`Content`] implements OpenAPI spec [Media Type Object][media_type]
///
/// [media_type]: <https://spec.openapis.org/oas/latest.html#media-type-object>
#[derive(serde_derive::Serialize, serde_derive::Deserialize, Default, Clone, PartialEq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[builder(on(_, into))]
#[non_exhaustive]
pub struct Content {
    /// Schema used in response body or request body.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub schema: Option<Schema>,

    /// Example for request body or response body.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub example: Option<Value>,

    /// Examples of the request body or response body. [`Content::examples`] should match to
    /// media type and specified schema if present. [`Content::examples`] and
    /// [`Content::example`] are mutually exclusive. If both are defined `examples` will
    /// override value in `example`.
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    #[builder(default)]
    pub examples: IndexMap<String, RefOr<Example>>,

    /// A map between a property name and its encoding information.
    ///
    /// The key, being the property name, MUST exist in the [`Content::schema`] as a property, with
    /// `schema` being a [`Schema::Object`] and this object containing the same property key in
    /// [`Object::properties`](crate::schema::Object::properties).
    ///
    /// The encoding object SHALL only apply to `request_body` objects when the media type is
    /// multipart or `application/x-www-form-urlencoded`.
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    #[builder(default)]
    pub encoding: IndexMap<String, Encoding>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", default, flatten)]
    pub extensions: Option<Extensions>,
}

impl Content {
    /// Construct a new [`Content`] object for provided _`schema`_.
    pub fn new<I: Into<Schema>>(schema: Option<I>) -> Self {
        Self {
            schema: schema.map(|schema| schema.into()),
            ..Self::default()
        }
    }
}

impl<S: content_builder::IsComplete> From<ContentBuilder<S>> for Content {
    fn from(builder: ContentBuilder<S>) -> Self {
        builder.build()
    }
}

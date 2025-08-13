//! Implements [OpenAPI Request Body][request_body] types.
//!
//! [request_body]: https://spec.openapis.org/oas/latest.html#request-body-object
use indexmap::IndexMap;
use serde_derive::{Deserialize, Serialize};

use super::Content;
use super::extensions::Extensions;

/// Implements [OpenAPI Request Body][request_body].
///
/// [request_body]: https://spec.openapis.org/oas/latest.html#request-body-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
#[builder(on(_, into))]
pub struct RequestBody {
    /// Map of request body contents mapped by content type e.g. `application/json`.
    #[builder(field)]
    pub content: IndexMap<String, Content>,

    /// Additional description of [`RequestBody`] supporting markdown syntax.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Determines whether request body is required in the request or not.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

impl RequestBody {
    /// Construct a new [`RequestBody`].
    pub fn new() -> Self {
        Default::default()
    }
}

impl<S: request_body_builder::State> RequestBodyBuilder<S> {
    /// Add [`Content`] by content type e.g `application/json` to [`RequestBody`].
    pub fn content(mut self, content_type: impl Into<String>, content: impl Into<Content>) -> Self {
        self.content.insert(content_type.into(), content.into());
        self
    }

    /// Add [`Content`] by content type e.g `application/json` to [`RequestBody`].
    pub fn contents<T: Into<String>, C: Into<Content>>(self, contents: impl IntoIterator<Item = (T, C)>) -> Self {
        contents.into_iter().fold(self, |this, (t, c)| this.content(t, c))
    }
}

impl<S: request_body_builder::IsComplete> From<RequestBodyBuilder<S>> for RequestBody {
    fn from(value: RequestBodyBuilder<S>) -> Self {
        value.build()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use insta::assert_json_snapshot;

    use super::{Content, RequestBody};
    use crate::Ref;

    #[test]
    fn request_body_new() {
        let request_body = RequestBody::new();

        assert!(request_body.content.is_empty());
        assert_eq!(request_body.description, None);
        assert!(request_body.required.is_none());
    }

    #[test]
    fn request_body_builder() {
        let request_body = RequestBody::builder()
            .description("A sample requestBody")
            .required(true)
            .content("application/json", Content::new(Some(Ref::from_schema_name("EmailPayload"))))
            .build();
        assert_json_snapshot!(request_body);
    }
}

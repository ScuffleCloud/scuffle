//! Implements [OpenApi Responses][responses].
//!
//! [responses]: https://spec.openapis.org/oas/latest.html#responses-object
use indexmap::IndexMap;
use serde_derive::{Deserialize, Serialize};

use super::Content;
use super::extensions::Extensions;
use super::header::Header;
use super::link::Link;
use crate::{Ref, RefOr};

/// Implements [OpenAPI Responses Object][responses].
///
/// Responses is a map holding api operation responses identified by their status code.
///
/// [responses]: https://spec.openapis.org/oas/latest.html#responses-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
#[builder(on(_, into))]
pub struct Responses {
    /// Map containing status code as a key with represented response as a value.
    #[serde(flatten)]
    #[builder(field)]
    pub responses: IndexMap<String, RefOr<Response>>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

impl Responses {
    /// Construct a new [`Responses`].
    pub fn new() -> Self {
        Default::default()
    }
}

impl<S: responses_builder::State> ResponsesBuilder<S> {
    /// Add a [`Response`].
    pub fn response(mut self, code: impl Into<String>, response: impl Into<RefOr<Response>>) -> Self {
        self.responses.insert(code.into(), response.into());

        self
    }

    /// Add responses from an iterator over a pair of `(status_code, response): (String, Response)`.
    pub fn responses_from_iter<I: IntoIterator<Item = (C, R)>, C: Into<String>, R: Into<RefOr<Response>>>(
        mut self,
        iter: I,
    ) -> Self {
        self.responses
            .extend(iter.into_iter().map(|(code, response)| (code.into(), response.into())));
        self
    }
}

impl From<Responses> for IndexMap<String, RefOr<Response>> {
    fn from(responses: Responses) -> Self {
        responses.responses
    }
}

impl<C, R> FromIterator<(C, R)> for Responses
where
    C: Into<String>,
    R: Into<RefOr<Response>>,
{
    fn from_iter<T: IntoIterator<Item = (C, R)>>(iter: T) -> Self {
        Self {
            responses: IndexMap::from_iter(iter.into_iter().map(|(code, response)| (code.into(), response.into()))),
            ..Default::default()
        }
    }
}

/// Implements [OpenAPI Response Object][response].
///
/// Response is api operation response.
///
/// [response]: https://spec.openapis.org/oas/latest.html#response-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
#[builder(on(_, into))]
pub struct Response {
    /// Map of headers identified by their name. `Content-Type` header will be ignored.
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    #[builder(field)]
    pub headers: IndexMap<String, Header>,

    /// Map of response [`Content`] objects identified by response body content type e.g `application/json`.
    ///
    /// [`Content`]s are stored within [`IndexMap`] to retain their insertion order. Swagger UI
    /// will create and show default example according to the first entry in `content` map.
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    #[builder(field)]
    pub content: IndexMap<String, Content>,

    /// A map of operations links that can be followed from the response. The key of the
    /// map is a short name for the link.
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    #[builder(field)]
    pub links: IndexMap<String, RefOr<Link>>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,

    /// Description of the response. Response support markdown syntax.
    pub description: String,
}

impl Response {
    /// Construct a new [`Response`].
    ///
    /// Function takes description as argument.
    pub fn new<S: Into<String>>(description: S) -> Self {
        Self {
            description: description.into(),
            ..Default::default()
        }
    }
}

impl<S: response_builder::State> ResponseBuilder<S> {
    /// Add [`Content`] of the [`Response`] with content type e.g `application/json`.
    pub fn content(mut self, content_type: impl Into<String>, content: impl Into<Content>) -> Self {
        self.content.insert(content_type.into(), content.into());
        self
    }

    /// Add response [`Header`].
    pub fn header(mut self, name: impl Into<String>, header: impl Into<Header>) -> Self {
        self.headers.insert(name.into(), header.into());
        self
    }

    /// Add link that can be followed from the response.
    pub fn link(mut self, name: impl Into<String>, link: impl Into<RefOr<Link>>) -> Self {
        self.links.insert(name.into(), link.into());
        self
    }

    /// Add [`Content`] of the [`Response`] with content type e.g `application/json`.
    pub fn contents<A: Into<String>, B: Into<Content>>(self, contents: impl IntoIterator<Item = (A, B)>) -> Self {
        contents.into_iter().fold(self, |this, (a, b)| this.content(a, b))
    }

    /// Add response [`Header`].
    pub fn headers<A: Into<String>, B: Into<Header>>(self, headers: impl IntoIterator<Item = (A, B)>) -> Self {
        headers.into_iter().fold(self, |this, (a, b)| this.header(a, b))
    }

    /// Add link that can be followed from the response.
    pub fn links<A: Into<String>, B: Into<RefOr<Link>>>(self, links: impl IntoIterator<Item = (A, B)>) -> Self {
        links.into_iter().fold(self, |this, (a, b)| this.link(a, b))
    }
}

impl<S: response_builder::IsComplete> From<ResponseBuilder<S>> for Response {
    fn from(builder: ResponseBuilder<S>) -> Self {
        builder.build()
    }
}

impl<S: response_builder::IsComplete> From<ResponseBuilder<S>> for RefOr<Response> {
    fn from(builder: ResponseBuilder<S>) -> Self {
        Self::T(builder.build())
    }
}

impl From<Ref> for RefOr<Response> {
    fn from(r: Ref) -> Self {
        Self::Ref(r)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use insta::assert_json_snapshot;

    use super::{Content, Responses};
    use crate::Response;

    #[test]
    fn responses_new() {
        let responses = Responses::new();

        assert!(responses.responses.is_empty());
    }

    #[test]
    fn response_builder() {
        let request_body = Response::builder()
            .description("A sample response")
            .content(
                "application/json",
                Content::new(Some(crate::Ref::from_schema_name("MySchemaPayload"))),
            )
            .build();
        assert_json_snapshot!(request_body);
    }
}

//! Implements [Open API Link Object][link_object] for responses.
//!
//! [link_object]: https://spec.openapis.org/oas/latest.html#link-object
use indexmap::IndexMap;
use serde_derive::{Deserialize, Serialize};

use super::Server;
use super::extensions::Extensions;

/// Implements [Open API Link Object][link_object] for responses.
///
/// The `Link` represents possible design time link for a response. It does not guarantee
/// callers ability to invoke it but rather provides known relationship between responses and
/// other operations.
///
/// For computing links, and providing instructions to execute them,
/// a runtime [expression][expression] is used for accessing values in an operation
/// and using them as parameters while invoking the linked operation.
///
/// [expression]: https://spec.openapis.org/oas/latest.html#runtime-expressions
/// [link_object]: https://spec.openapis.org/oas/latest.html#link-object
#[derive(Serialize, Deserialize, Clone, PartialEq, Default, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[builder(on(_, into))]
#[non_exhaustive]
pub struct Link {
    /// A map representing parameters to pass to an operation as specified with _`operation_id`_
    /// or identified by _`operation_ref`_. The key is parameter name to be used and value can
    /// be any value supported by JSON or an [expression][expression] e.g. `$path.id`
    ///
    /// [expression]: https://spec.openapis.org/oas/latest.html#runtime-expressions
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    #[builder(field)]
    pub parameters: IndexMap<String, serde_json::Value>,

    /// A relative or absolute URI reference to an OAS operation. This field is
    /// mutually exclusive of the _`operation_id`_ field, and **must** point to an [Operation
    /// Object][operation].
    /// Relative _`operation_ref`_ values may be used to locate an existing [Operation
    /// Object][operation] in the OpenAPI definition. See the rules for resolving [Relative
    /// References][relative_references].
    ///
    /// [relative_references]: https://spec.openapis.org/oas/latest.html#relative-references-in-uris
    /// [operation]: ../path/struct.Operation.html
    #[serde(skip_serializing_if = "String::is_empty", default)]
    #[builder(default)]
    pub operation_ref: String,

    /// The name of an existing, resolvable OAS operation, as defined with a unique
    /// _`operation_id`_.
    /// This field is mutually exclusive of the _`operation_ref`_ field.
    #[serde(skip_serializing_if = "String::is_empty", default)]
    #[builder(default)]
    pub operation_id: String,

    /// A literal value or an [expression][expression] to be used as request body when operation is called.
    ///
    /// [expression]: https://spec.openapis.org/oas/latest.html#runtime-expressions
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub request_body: Option<serde_json::Value>,

    /// Description of the link. Value supports Markdown syntax.
    #[serde(skip_serializing_if = "String::is_empty", default)]
    #[builder(default)]
    pub description: String,

    /// A [`Server`][server] object to be used by the target operation.
    ///
    /// [server]: ../server/struct.Server.html
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub server: Option<Server>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", default, flatten)]
    pub extensions: Option<Extensions>,
}

impl<S: link_builder::State> LinkBuilder<S> {
    /// Add parameters to be passed to [Operation][operation] upon execution.
    ///
    /// [operation]: ../path/struct.Operation.html
    pub fn parameters<N: Into<String>, V: Into<serde_json::Value>>(self, items: impl IntoIterator<Item = (N, V)>) -> Self {
        items.into_iter().fold(self, |this, (n, v)| this.parameter(n, v))
    }

    /// Add parameter to be passed to [Operation][operation] upon execution.
    ///
    /// [operation]: ../path/struct.Operation.html
    pub fn parameter<N: Into<String>, V: Into<serde_json::Value>>(mut self, name: N, value: V) -> Self {
        self.parameters.insert(name.into(), value.into());
        self
    }
}

impl<S: link_builder::IsComplete> From<LinkBuilder<S>> for Link {
    fn from(builder: LinkBuilder<S>) -> Self {
        builder.build()
    }
}

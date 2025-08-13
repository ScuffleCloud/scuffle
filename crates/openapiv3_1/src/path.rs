//! Implements [OpenAPI Path Object][paths] types.
//!
//! [paths]: https://spec.openapis.org/oas/latest.html#paths-object
use indexmap::IndexMap;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

use super::extensions::Extensions;
use super::request_body::RequestBody;
use super::response::{Response, Responses};
use super::security::SecurityRequirement;
use super::{Deprecated, ExternalDocs, RefOr, Schema, Server};

/// Implements [OpenAPI Paths Object][paths].
///
/// Holds relative paths to matching endpoints and operations. The path is appended to the url
/// from [`Server`] object to construct a full url for endpoint.
///
/// [paths]: https://spec.openapis.org/oas/latest.html#paths-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[builder(on(_, into))]
pub struct Paths {
    /// Map of relative paths with [`PathItem`]s holding [`Operation`]s matching
    /// api endpoints.
    #[serde(flatten)]
    #[builder(field)]
    pub paths: IndexMap<String, PathItem>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

impl Paths {
    /// Construct a new [`Paths`] object.
    pub fn new() -> Self {
        Default::default()
    }

    /// Return _`Option`_ of reference to [`PathItem`] by given relative path _`P`_ if one exists
    /// in [`Paths::paths`] map. Otherwise will return `None`.
    ///
    /// # Examples
    ///
    /// _**Get user path item.**_
    /// ```rust
    /// # use openapiv3_1::path::{Paths, HttpMethod};
    /// # let paths = Paths::new();
    /// let path_item = paths.get_path_item("/api/v1/user");
    /// ```
    pub fn get_path_item<P: AsRef<str>>(&self, path: P) -> Option<&PathItem> {
        self.paths.get(path.as_ref())
    }

    /// Return _`Option`_ of reference to [`Operation`] from map of paths or `None` if not found.
    ///
    /// * First will try to find [`PathItem`] by given relative path _`P`_ e.g. `"/api/v1/user"`.
    /// * Then tries to find [`Operation`] from [`PathItem`]'s operations by given [`HttpMethod`].
    ///
    /// # Examples
    ///
    /// _**Get user operation from paths.**_
    /// ```rust
    /// # use openapiv3_1::path::{Paths, HttpMethod};
    /// # let paths = Paths::new();
    /// let operation = paths.get_path_operation("/api/v1/user", HttpMethod::Get);
    /// ```
    pub fn get_path_operation<P: AsRef<str>>(&self, path: P, http_method: HttpMethod) -> Option<&Operation> {
        self.paths.get(path.as_ref()).and_then(|path| match http_method {
            HttpMethod::Get => path.get.as_ref(),
            HttpMethod::Put => path.put.as_ref(),
            HttpMethod::Post => path.post.as_ref(),
            HttpMethod::Delete => path.delete.as_ref(),
            HttpMethod::Options => path.options.as_ref(),
            HttpMethod::Head => path.head.as_ref(),
            HttpMethod::Patch => path.patch.as_ref(),
            HttpMethod::Trace => path.trace.as_ref(),
        })
    }

    /// Append path operation to the list of paths.
    ///
    /// Method accepts three arguments; `path` to add operation for, `http_methods` list of
    /// allowed HTTP methods for the [`Operation`] and `operation` to be added under the _`path`_.
    ///
    /// If _`path`_ already exists, the provided [`Operation`] will be set to existing path item for
    /// given list of [`HttpMethod`]s.
    pub fn add_path_operation<P: AsRef<str>, O: Into<Operation>>(
        &mut self,
        path: P,
        http_methods: Vec<HttpMethod>,
        operation: O,
    ) {
        let path = path.as_ref();
        let operation = operation.into();
        if let Some(existing_item) = self.paths.get_mut(path) {
            for http_method in http_methods {
                match http_method {
                    HttpMethod::Get => existing_item.get = Some(operation.clone()),
                    HttpMethod::Put => existing_item.put = Some(operation.clone()),
                    HttpMethod::Post => existing_item.post = Some(operation.clone()),
                    HttpMethod::Delete => existing_item.delete = Some(operation.clone()),
                    HttpMethod::Options => existing_item.options = Some(operation.clone()),
                    HttpMethod::Head => existing_item.head = Some(operation.clone()),
                    HttpMethod::Patch => existing_item.patch = Some(operation.clone()),
                    HttpMethod::Trace => existing_item.trace = Some(operation.clone()),
                };
            }
        } else {
            self.paths
                .insert(String::from(path), PathItem::from_http_methods(http_methods, operation));
        }
    }

    /// Merge _`other_paths`_ into `self`. On conflicting path the path item operations will be
    /// merged into existing [`PathItem`]. Otherwise path with [`PathItem`] will be appended to
    /// `self`. All [`Extensions`] will be merged from _`other_paths`_ into `self`.
    pub fn merge(&mut self, other_paths: Paths) {
        for (path, that) in other_paths.paths {
            if let Some(this) = self.paths.get_mut(&path) {
                this.merge_operations(that);
            } else {
                self.paths.insert(path, that);
            }
        }

        if let Some(other_paths_extensions) = other_paths.extensions {
            let paths_extensions = self.extensions.get_or_insert(Extensions::default());
            paths_extensions.merge(other_paths_extensions);
        }
    }
}

impl<S: paths_builder::State> PathsBuilder<S> {
    /// Append [`PathItem`] with path to map of paths. If path already exists it will merge [`Operation`]s of
    /// [`PathItem`] with already found path item operations.
    pub fn path(mut self, path: impl Into<String>, item: impl Into<PathItem>) -> Self {
        let path_string = path.into();
        let item = item.into();
        if let Some(existing_item) = self.paths.get_mut(&path_string) {
            existing_item.merge_operations(item);
        } else {
            self.paths.insert(path_string, item);
        }

        self
    }

    /// Append [`PathItem`]s with path to map of paths. If path already exists it will merge [`Operation`]s of
    /// [`PathItem`] with already found path item operations.
    pub fn paths<I: Into<String>, P: Into<PathItem>>(self, items: impl IntoIterator<Item = (I, P)>) -> Self {
        items.into_iter().fold(self, |this, (i, p)| this.path(i, p))
    }
}

impl<S: paths_builder::IsComplete> From<PathsBuilder<S>> for Paths {
    fn from(builder: PathsBuilder<S>) -> Self {
        builder.build()
    }
}

/// Implements [OpenAPI Path Item Object][path_item] what describes [`Operation`]s available on
/// a single path.
///
/// [path_item]: https://spec.openapis.org/oas/latest.html#path-item-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
#[builder(on(_, into))]
pub struct PathItem {
    /// Optional summary intended to apply all operations in this [`PathItem`].
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub summary: Option<String>,

    /// Optional description intended to apply all operations in this [`PathItem`].
    /// Description supports markdown syntax.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,

    /// Alternative [`Server`] array to serve all [`Operation`]s in this [`PathItem`] overriding
    /// the global server array.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub servers: Option<Vec<Server>>,

    /// List of [`Parameter`]s common to all [`Operation`]s in this [`PathItem`]. Parameters cannot
    /// contain duplicate parameters. They can be overridden in [`Operation`] level but cannot be
    /// removed there.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub parameters: Option<Vec<Parameter>>,

    /// Get [`Operation`] for the [`PathItem`].
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub get: Option<Operation>,

    /// Put [`Operation`] for the [`PathItem`].
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub put: Option<Operation>,

    /// Post [`Operation`] for the [`PathItem`].
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub post: Option<Operation>,

    /// Delete [`Operation`] for the [`PathItem`].
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub delete: Option<Operation>,

    /// Options [`Operation`] for the [`PathItem`].
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<Operation>,

    /// Head [`Operation`] for the [`PathItem`].
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub head: Option<Operation>,

    /// Patch [`Operation`] for the [`PathItem`].
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub patch: Option<Operation>,

    /// Trace [`Operation`] for the [`PathItem`].
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub trace: Option<Operation>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

impl<S: path_item_builder::IsComplete> From<PathItemBuilder<S>> for PathItem {
    fn from(builder: PathItemBuilder<S>) -> Self {
        builder.build()
    }
}

impl PathItem {
    /// Construct a new [`PathItem`] with provided [`Operation`] mapped to given [`HttpMethod`].
    pub fn new<O: Into<Operation>>(http_method: HttpMethod, operation: O) -> Self {
        let mut path_item = Self::default();
        match http_method {
            HttpMethod::Get => path_item.get = Some(operation.into()),
            HttpMethod::Put => path_item.put = Some(operation.into()),
            HttpMethod::Post => path_item.post = Some(operation.into()),
            HttpMethod::Delete => path_item.delete = Some(operation.into()),
            HttpMethod::Options => path_item.options = Some(operation.into()),
            HttpMethod::Head => path_item.head = Some(operation.into()),
            HttpMethod::Patch => path_item.patch = Some(operation.into()),
            HttpMethod::Trace => path_item.trace = Some(operation.into()),
        };

        path_item
    }

    /// Constructs a new [`PathItem`] with given [`Operation`] set for provided [`HttpMethod`]s.
    pub fn from_http_methods<I: IntoIterator<Item = HttpMethod>, O: Into<Operation>>(http_methods: I, operation: O) -> Self {
        let mut path_item = Self::default();
        let operation = operation.into();
        for method in http_methods {
            match method {
                HttpMethod::Get => path_item.get = Some(operation.clone()),
                HttpMethod::Put => path_item.put = Some(operation.clone()),
                HttpMethod::Post => path_item.post = Some(operation.clone()),
                HttpMethod::Delete => path_item.delete = Some(operation.clone()),
                HttpMethod::Options => path_item.options = Some(operation.clone()),
                HttpMethod::Head => path_item.head = Some(operation.clone()),
                HttpMethod::Patch => path_item.patch = Some(operation.clone()),
                HttpMethod::Trace => path_item.trace = Some(operation.clone()),
            };
        }

        path_item
    }

    /// Merge all defined [`Operation`]s from given [`PathItem`] to `self` if `self` does not have
    /// existing operation.
    pub fn merge_operations(&mut self, path_item: PathItem) {
        if path_item.get.is_some() && self.get.is_none() {
            self.get = path_item.get;
        }
        if path_item.put.is_some() && self.put.is_none() {
            self.put = path_item.put;
        }
        if path_item.post.is_some() && self.post.is_none() {
            self.post = path_item.post;
        }
        if path_item.delete.is_some() && self.delete.is_none() {
            self.delete = path_item.delete;
        }
        if path_item.options.is_some() && self.options.is_none() {
            self.options = path_item.options;
        }
        if path_item.head.is_some() && self.head.is_none() {
            self.head = path_item.head;
        }
        if path_item.patch.is_some() && self.patch.is_none() {
            self.patch = path_item.patch;
        }
        if path_item.trace.is_some() && self.trace.is_none() {
            self.trace = path_item.trace;
        }
    }
}

/// HTTP method of the operation.
///
/// List of supported HTTP methods <https://spec.openapis.org/oas/latest.html#path-item-object>
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord, Clone)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum HttpMethod {
    /// Type mapping for HTTP _GET_ request.
    Get,
    /// Type mapping for HTTP _POST_ request.
    Post,
    /// Type mapping for HTTP _PUT_ request.
    Put,
    /// Type mapping for HTTP _DELETE_ request.
    Delete,
    /// Type mapping for HTTP _OPTIONS_ request.
    Options,
    /// Type mapping for HTTP _HEAD_ request.
    Head,
    /// Type mapping for HTTP _PATCH_ request.
    Patch,
    /// Type mapping for HTTP _TRACE_ request.
    Trace,
}

impl HttpMethod {
    /// Returns the lower-case string representation of tghe http method
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Get => "get",
            Self::Post => "post",
            Self::Put => "put",
            Self::Delete => "delete",
            Self::Options => "options",
            Self::Head => "head",
            Self::Patch => "patch",
            Self::Trace => "trace",
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Implements [OpenAPI Operation Object][operation] object.
///
/// [operation]: https://spec.openapis.org/oas/latest.html#operation-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
#[builder(on(_, into))]
pub struct Operation {
    /// List of tags used for grouping operations.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(field)]
    pub tags: Option<Vec<String>>,

    /// List of applicable parameters for this [`Operation`].
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(field)]
    pub parameters: Option<Vec<Parameter>>,

    /// List of possible responses returned by the [`Operation`].
    #[builder(field)]
    pub responses: Responses,

    /// Alternative [`Server`]s for this [`Operation`].
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(field)]
    pub servers: Option<Vec<Server>>,

    /// Declaration which security mechanisms can be used for for the operation. Only one
    /// [`SecurityRequirement`] must be met.
    ///
    /// Security for the [`Operation`] can be set to optional by adding empty security with
    /// [`SecurityRequirement::default`].
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[builder(field)]
    pub security: Option<Vec<SecurityRequirement>>,

    /// Short summary what [`Operation`] does.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub summary: Option<String>,

    /// Long explanation of [`Operation`] behaviour. Markdown syntax is supported.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,

    /// Unique identifier for the API [`Operation`]. Most typically this is mapped to handler function name.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub operation_id: Option<String>,

    /// Additional external documentation for this operation.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub external_docs: Option<ExternalDocs>,

    /// Optional request body for this [`Operation`].
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub request_body: Option<RequestBody>,

    // TODO
    #[allow(missing_docs)]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub callbacks: Option<String>,

    /// Define whether the operation is deprecated or not and thus should be avoided consuming.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub deprecated: Option<Deprecated>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", default, flatten)]
    pub extensions: Option<Extensions>,
}

impl<S: operation_builder::IsComplete> From<OperationBuilder<S>> for Operation {
    fn from(builder: OperationBuilder<S>) -> Self {
        builder.build()
    }
}

impl Operation {
    /// Construct a new API [`Operation`].
    pub fn new() -> Self {
        Default::default()
    }
}

impl<S: operation_builder::State> OperationBuilder<S> {
    /// Append tag to [`Operation`] tags.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.get_or_insert_default().push(tag.into());
        self
    }

    /// Append tag to [`Operation`] tags.
    pub fn tags<T: Into<String>>(self, tags: impl IntoIterator<Item = T>) -> Self {
        tags.into_iter().fold(self, |this, t| this.tag(t))
    }

    /// Add or change parameters of the [`Operation`].
    pub fn parameters<P: Into<Parameter>>(self, parameters: impl IntoIterator<Item = P>) -> Self {
        parameters.into_iter().fold(self, |this, p| this.parameter(p))
    }

    /// Append parameter to [`Operation`] parameters.
    pub fn parameter(mut self, parameter: impl Into<Parameter>) -> Self {
        self.parameters.get_or_insert_default().push(parameter.into());
        self
    }

    /// Add or change responses of the [`Operation`].
    pub fn responses<R: Into<RefOr<Response>>, C: Into<String>>(self, responses: impl IntoIterator<Item = (C, R)>) -> Self {
        responses.into_iter().fold(self, |this, (c, r)| this.response(c, r))
    }

    /// Append status code and a [`Response`] to the [`Operation`] responses map.
    ///
    /// * `code` must be valid HTTP status code.
    /// * `response` is instances of [`Response`].
    pub fn response(mut self, code: impl Into<String>, response: impl Into<RefOr<Response>>) -> Self {
        self.responses.responses.insert(code.into(), response.into());

        self
    }

    /// Append [`SecurityRequirement`] to [`Operation`] security requirements.
    pub fn security(mut self, security: impl Into<SecurityRequirement>) -> Self {
        self.security.get_or_insert_default().push(security.into());
        self
    }

    /// Append [`SecurityRequirement`] to [`Operation`] security requirements.
    pub fn securities<R: Into<SecurityRequirement>>(self, securities: impl IntoIterator<Item = R>) -> Self {
        securities.into_iter().fold(self, |this, s| this.security(s))
    }

    /// Append [`Server`]s to the [`Operation`].
    pub fn servers<E: Into<Server>>(self, servers: impl IntoIterator<Item = E>) -> Self {
        servers.into_iter().fold(self, |this, e| this.server(e))
    }

    /// Append a new [`Server`] to the [`Operation`] servers.
    pub fn server(mut self, server: impl Into<Server>) -> Self {
        self.servers.get_or_insert_default().push(server.into());
        self
    }
}

impl Operation {
    /// Append tag to [`Operation`] tags.
    pub fn tag(&mut self, tag: impl Into<String>) -> &mut Self {
        self.tags.get_or_insert_default().push(tag.into());
        self
    }

    /// Append tag to [`Operation`] tags.
    pub fn tags<T: Into<String>>(&mut self, tags: impl IntoIterator<Item = T>) -> &mut Self {
        tags.into_iter().fold(self, |this, t| this.tag(t))
    }

    /// Add or change parameters of the [`Operation`].
    pub fn parameters<P: Into<Parameter>>(&mut self, parameters: impl IntoIterator<Item = P>) -> &mut Self {
        parameters.into_iter().fold(self, |this, p| this.parameter(p))
    }

    /// Append parameter to [`Operation`] parameters.
    pub fn parameter(&mut self, parameter: impl Into<Parameter>) -> &mut Self {
        self.parameters.get_or_insert_default().push(parameter.into());
        self
    }

    /// Add or change responses of the [`Operation`].
    pub fn responses<R: Into<RefOr<Response>>, C: Into<String>>(
        &mut self,
        responses: impl IntoIterator<Item = (C, R)>,
    ) -> &mut Self {
        responses.into_iter().fold(self, |this, (c, r)| this.response(c, r))
    }

    /// Append status code and a [`Response`] to the [`Operation`] responses map.
    ///
    /// * `code` must be valid HTTP status code.
    /// * `response` is instances of [`Response`].
    pub fn response(&mut self, code: impl Into<String>, response: impl Into<RefOr<Response>>) -> &mut Self {
        self.responses.responses.insert(code.into(), response.into());

        self
    }

    /// Append [`SecurityRequirement`] to [`Operation`] security requirements.
    pub fn security(&mut self, security: impl Into<SecurityRequirement>) -> &mut Self {
        self.security.get_or_insert_default().push(security.into());
        self
    }

    /// Append [`SecurityRequirement`] to [`Operation`] security requirements.
    pub fn securities<R: Into<SecurityRequirement>>(&mut self, securities: impl IntoIterator<Item = R>) -> &mut Self {
        securities.into_iter().fold(self, |this, s| this.security(s))
    }

    /// Append [`Server`]s to the [`Operation`].
    pub fn servers<E: Into<Server>>(&mut self, servers: impl IntoIterator<Item = E>) -> &mut Self {
        servers.into_iter().fold(self, |this, e| this.server(e))
    }

    /// Append a new [`Server`] to the [`Operation`] servers.
    pub fn server(&mut self, server: impl Into<Server>) -> &mut Self {
        self.servers.get_or_insert_default().push(server.into());
        self
    }
}

/// Implements [OpenAPI Parameter Object][parameter] for [`Operation`].
///
/// [parameter]: https://spec.openapis.org/oas/latest.html#parameter-object
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
#[builder(on(_, into))]
pub struct Parameter {
    /// Name of the parameter.
    ///
    /// * For [`ParameterIn::Path`] this must in accordance to path templating.
    /// * For [`ParameterIn::Query`] `Content-Type` or `Authorization` value will be ignored.
    pub name: String,

    /// Parameter location.
    #[serde(rename = "in")]
    pub parameter_in: ParameterIn,

    /// Markdown supported description of the parameter.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,

    /// Declares whether the parameter is required or not for api.
    ///
    /// * For [`ParameterIn::Path`] this must and will be [`true`].
    pub required: bool,

    /// Declares the parameter deprecated status.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub deprecated: Option<Deprecated>,
    // pub allow_empty_value: bool, this is going to be removed from further open api spec releases
    /// Schema of the parameter. Typically [`Schema::Object`] is used.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub schema: Option<Schema>,

    /// Describes how [`Parameter`] is being serialized depending on [`Parameter::schema`] (type of a content).
    /// Default value is based on [`ParameterIn`].
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub style: Option<ParameterStyle>,

    /// When _`true`_ it will generate separate parameter value for each parameter with _`array`_ and _`object`_ type.
    /// This is also _`true`_ by default for [`ParameterStyle::Form`].
    ///
    /// With explode _`false`_:
    /// ```text
    /// color=blue,black,brown
    /// ```
    ///
    /// With explode _`true`_:
    /// ```text
    /// color=blue&color=black&color=brown
    /// ```
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub explode: Option<bool>,

    /// Defines whether parameter should allow reserved characters defined by
    /// [RFC3986](https://tools.ietf.org/html/rfc3986#section-2.2) _`:/?#[]@!$&'()*+,;=`_.
    /// This is only applicable with [`ParameterIn::Query`]. Default value is _`false`_.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub allow_reserved: Option<bool>,

    /// Example of [`Parameter`]'s potential value. This examples will override example
    /// within [`Parameter::schema`] if defined.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    example: Option<Value>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", default, flatten)]
    pub extensions: Option<Extensions>,
}

impl Parameter {
    /// Constructs a new required [`Parameter`] with given name.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            required: true,
            ..Default::default()
        }
    }
}

/// In definition of [`Parameter`].
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ParameterIn {
    /// Declares that parameter is used as query parameter.
    Query,
    /// Declares that parameter is used as path parameter.
    Path,
    /// Declares that parameter is used as header value.
    Header,
    /// Declares that parameter is used as cookie value.
    Cookie,
}

impl Default for ParameterIn {
    fn default() -> Self {
        Self::Path
    }
}

/// Defines how [`Parameter`] should be serialized.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
pub enum ParameterStyle {
    /// Path style parameters defined by [RFC6570](https://tools.ietf.org/html/rfc6570#section-3.2.7)
    /// e.g _`;color=blue`_.
    /// Allowed with [`ParameterIn::Path`].
    Matrix,
    /// Label style parameters defined by [RFC6570](https://datatracker.ietf.org/doc/html/rfc6570#section-3.2.5)
    /// e.g _`.color=blue`_.
    /// Allowed with [`ParameterIn::Path`].
    Label,
    /// Form style parameters defined by [RFC6570](https://datatracker.ietf.org/doc/html/rfc6570#section-3.2.8)
    /// e.g. _`color=blue`_. Default value for [`ParameterIn::Query`] [`ParameterIn::Cookie`].
    /// Allowed with [`ParameterIn::Query`] or [`ParameterIn::Cookie`].
    Form,
    /// Default value for [`ParameterIn::Path`] [`ParameterIn::Header`]. e.g. _`blue`_.
    /// Allowed with [`ParameterIn::Path`] or [`ParameterIn::Header`].
    Simple,
    /// Space separated array values e.g. _`blue%20black%20brown`_.
    /// Allowed with [`ParameterIn::Query`].
    SpaceDelimited,
    /// Pipe separated array values e.g. _`blue|black|brown`_.
    /// Allowed with [`ParameterIn::Query`].
    PipeDelimited,
    /// Simple way of rendering nested objects using form parameters .e.g. _`color[B]=150`_.
    /// Allowed with [`ParameterIn::Query`].
    DeepObject,
}

#[cfg(test)]
#[cfg(feature = "debug")]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{HttpMethod, Operation};
    use crate::security::SecurityRequirement;
    use crate::server::Server;
    use crate::{PathItem, Paths};

    #[test]
    fn test_path_order() {
        let paths_list = Paths::builder()
            .path("/todo", PathItem::new(HttpMethod::Get, Operation::new()))
            .path("/todo", PathItem::new(HttpMethod::Post, Operation::new()))
            .path("/todo/{id}", PathItem::new(HttpMethod::Delete, Operation::new()))
            .path("/todo/{id}", PathItem::new(HttpMethod::Get, Operation::new()))
            .path("/todo/{id}", PathItem::new(HttpMethod::Put, Operation::new()))
            .path("/todo/search", PathItem::new(HttpMethod::Get, Operation::new()))
            .build();

        let actual_value = paths_list
            .paths
            .iter()
            .flat_map(|(path, path_item)| {
                let mut path_methods = Vec::<(&str, &HttpMethod)>::with_capacity(paths_list.paths.len());
                if path_item.get.is_some() {
                    path_methods.push((path, &HttpMethod::Get));
                }
                if path_item.put.is_some() {
                    path_methods.push((path, &HttpMethod::Put));
                }
                if path_item.post.is_some() {
                    path_methods.push((path, &HttpMethod::Post));
                }
                if path_item.delete.is_some() {
                    path_methods.push((path, &HttpMethod::Delete));
                }
                if path_item.options.is_some() {
                    path_methods.push((path, &HttpMethod::Options));
                }
                if path_item.head.is_some() {
                    path_methods.push((path, &HttpMethod::Head));
                }
                if path_item.patch.is_some() {
                    path_methods.push((path, &HttpMethod::Patch));
                }
                if path_item.trace.is_some() {
                    path_methods.push((path, &HttpMethod::Trace));
                }

                path_methods
            })
            .collect::<Vec<_>>();

        let get = HttpMethod::Get;
        let post = HttpMethod::Post;
        let put = HttpMethod::Put;
        let delete = HttpMethod::Delete;

        let expected_value = vec![
            ("/todo", &get),
            ("/todo", &post),
            ("/todo/{id}", &get),
            ("/todo/{id}", &put),
            ("/todo/{id}", &delete),
            ("/todo/search", &get),
        ];
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn operation_new() {
        let operation = Operation::new();

        assert!(operation.tags.is_none());
        assert!(operation.summary.is_none());
        assert!(operation.description.is_none());
        assert!(operation.operation_id.is_none());
        assert!(operation.external_docs.is_none());
        assert!(operation.parameters.is_none());
        assert!(operation.request_body.is_none());
        assert!(operation.responses.responses.is_empty());
        assert!(operation.callbacks.is_none());
        assert!(operation.deprecated.is_none());
        assert!(operation.security.is_none());
        assert!(operation.servers.is_none());
    }

    #[test]
    fn operation_builder_security() {
        let security_requirement1 = SecurityRequirement::new("api_oauth2_flow", ["edit:items", "read:items"]);
        let security_requirement2 = SecurityRequirement::new("api_oauth2_flow", ["remove:items"]);
        let operation = Operation::builder()
            .security(security_requirement1)
            .security(security_requirement2)
            .build();

        assert!(operation.security.is_some());
    }

    #[test]
    fn operation_builder_server() {
        let server1 = Server::new("/api");
        let server2 = Server::new("/admin");
        let operation = Operation::builder().server(server1).server(server2).build();

        assert!(operation.servers.is_some());
    }
}

//! Implements [OpenAPI Server Object][server] types to configure target servers.
//!
//! OpenAPI will implicitly add [`Server`] with `url = "/"` to [`OpenApi`][openapi] when no servers
//! are defined.
//!
//! [`Server`] can be used to alter connection url for _**path operations**_. It can be a
//! relative path e.g `/api/v1` or valid http url e.g. `http://alternative.api.com/api/v1`.
//!
//! Relative path will append to the **sever address** so the connection url for _**path operations**_
//! will become `server address + relative path`.
//!
//! Optionally it also supports parameter substitution with `{variable}` syntax.
//!
//! See [`Modify`][modify] trait for details how add servers to [`OpenApi`][openapi].
//!
//! # Examples
//!
//! Create new server with relative path.
//! ```rust
//! # use openapiv3_1::server::Server;
//! Server::new("/api/v1");
//! ```
//!
//! Create server with custom url using a builder.
//! ```rust
//! # use openapiv3_1::server::Server;
//! Server::builder().url("https://alternative.api.url.test/api").build();
//! ```
//!
//! Create server with builder and variable substitution.
//! ```rust
//! # use openapiv3_1::server::{Server, ServerVariable};
//! Server::builder().url("/api/{version}/{username}")
//!     .parameter("version", ServerVariable::builder()
//!         .enum_values(["v1".into(), "v2".into()])
//!         .default_value("v1"))
//!     .parameter("username", ServerVariable::builder()
//!         .default_value("the_user")).build();
//! ```
//!
//! [server]: https://spec.openapis.org/oas/latest.html#server-object
//! [openapi]: ../struct.OpenApi.html
//! [modify]: ../../trait.Modify.html
use indexmap::IndexMap;

use super::extensions::Extensions;

/// Represents target server object. It can be used to alter server connection for
/// _**path operations**_.
///
/// By default OpenAPI will implicitly implement [`Server`] with `url = "/"` if no servers is provided to
/// the [`OpenApi`][openapi].
///
/// [openapi]: ../struct.OpenApi.html
#[non_exhaustive]
#[derive(serde_derive::Serialize, serde_derive::Deserialize, Default, Clone, PartialEq, Eq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
#[builder(on(_, into))]
pub struct Server {
    /// Optional map of variable name and its substitution value used in [`Server::url`].
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(field)]
    pub variables: Option<IndexMap<String, ServerVariable>>,

    /// Target url of the [`Server`]. It can be valid http url or relative path.
    ///
    /// Url also supports variable substitution with `{variable}` syntax. The substitutions
    /// then can be configured with [`Server::variables`] map.
    pub url: String,

    /// Optional description describing the target server url. Description supports markdown syntax.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

impl Server {
    /// Construct a new [`Server`] with given url. Url can be valid http url or context path of the url.
    ///
    /// If url is valid http url then all path operation request's will be forwarded to the selected [`Server`].
    ///
    /// If url is path of url e.g. `/api/v1` then the url will be appended to the servers address and the
    /// operations will be forwarded to location `server address + url`.
    ///
    ///
    /// # Examples
    ///
    /// Create new server with url path.
    /// ```rust
    /// # use openapiv3_1::server::Server;
    ///  Server::new("/api/v1");
    /// ```
    ///
    /// Create new server with alternative server.
    /// ```rust
    /// # use openapiv3_1::server::Server;
    ///  Server::new("https://alternative.pet-api.test/api/v1");
    /// ```
    pub fn new<S: Into<String>>(url: S) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }
}

impl<S: server_builder::State> ServerBuilder<S> {
    /// Add parameter to [`Server`] which is used to substitute values in [`Server::url`].
    ///
    /// * `name` Defines name of the parameter which is being substituted within the url. If url has
    ///   `{username}` substitution then the name should be `username`.
    /// * `parameter` Use [`ServerVariableBuilder`] to define how the parameter is being substituted
    ///   within the url.
    pub fn parameter(mut self, name: impl Into<String>, variable: impl Into<ServerVariable>) -> Self {
        self.variables.get_or_insert_default().insert(name.into(), variable.into());
        self
    }
}

/// Implements [OpenAPI Server Variable][server_variable] used to substitute variables in [`Server::url`].
///
/// [server_variable]: https://spec.openapis.org/oas/latest.html#server-variable-object
#[non_exhaustive]
#[derive(serde_derive::Serialize, serde_derive::Deserialize, Default, Clone, PartialEq, Eq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[builder(on(_, into))]
pub struct ServerVariable {
    /// Default value used to substitute parameter if no other value is being provided.
    #[serde(rename = "default")]
    pub default_value: String,

    /// Optional description describing the variable of substitution. Markdown syntax is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Enum values can be used to limit possible options for substitution. If enum values is used
    /// the [`ServerVariable::default_value`] must contain one of the enum values.
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

impl<S: server_variable_builder::IsComplete> From<ServerVariableBuilder<S>> for ServerVariable {
    fn from(value: ServerVariableBuilder<S>) -> Self {
        value.build()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    macro_rules! test_fn {
        ($name:ident : $schema:expr; $expected:literal) => {
            #[test]
            fn $name() {
                let value = serde_json::to_value($schema).unwrap();
                let expected_value: serde_json::Value = serde_json::from_str($expected).unwrap();

                assert_eq!(
                    value,
                    expected_value,
                    "testing serializing \"{}\": \nactual:\n{}\nexpected:\n{}",
                    stringify!($name),
                    value,
                    expected_value
                );

                println!("{}", &serde_json::to_string_pretty(&$schema).unwrap());
            }
        };
    }

    test_fn! {
    create_server_with_builder_and_variable_substitution:
    Server::builder().url("/api/{version}/{username}")
        .parameter("version", ServerVariable::builder()
            .enum_values(["v1".into(), "v2".into()])
            .description("api version")
            .default_value("v1"))
        .parameter("username", ServerVariable::builder()
            .default_value("the_user")).build();
    r###"{
  "url": "/api/{version}/{username}",
  "variables": {
      "version": {
          "enum": ["v1", "v2"],
          "default": "v1",
          "description": "api version"
      },
      "username": {
          "default": "the_user"
      }
  }
}"###
    }
}

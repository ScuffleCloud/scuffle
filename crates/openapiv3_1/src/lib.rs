//! Rust implementation of OpenAPI Spec v3.1.x
//!
//! A lof the code was taken from [`utoipa`](https://crates.io/crates/utoipa).
//!
//! The main difference is the full JSON Schema 2020-12 Definitions.
#![cfg_attr(feature = "docs", doc = "\n\nSee the [changelog][changelog] for a full release history.")]
#![cfg_attr(feature = "docs", doc = "## Feature flags")]
#![cfg_attr(feature = "docs", doc = document_features::document_features!())]
//! ## Alternatives
//!
//! - [`openapiv3`](https://crates.io/crates/openapiv3): Implements the openapi v3.0.x spec, does not implement full json schema draft 2020-12 spec.
//! - [`utoipa`](https://crates.io/crates/utoipa): A fully fletched openapi-type-generator implementing some of the v3.1.x spec.
//! - [`schemars`](https://crates.io/crates/schemars): A fully fletched jsonschema-type-generator implementing some of the json schema draft 2020-12 spec.
//!
//! ## License
//!
//! This project is licensed under the MIT or Apache-2.0 license.
//! You can choose between one of them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

use std::fmt::Formatter;

use indexmap::IndexMap;
use serde::de::{Error, Expected, Visitor};
use serde::{Deserializer, Serializer};
use serde_derive::{Deserialize, Serialize};

pub use self::content::{Content, ContentBuilder};
pub use self::external_docs::ExternalDocs;
pub use self::header::{Header, HeaderBuilder};
pub use self::info::{Contact, ContactBuilder, Info, InfoBuilder, License, LicenseBuilder};
pub use self::path::{HttpMethod, PathItem, Paths, PathsBuilder};
pub use self::response::{Response, ResponseBuilder, Responses, ResponsesBuilder};
pub use self::schema::{Components, ComponentsBuilder, Discriminator, Object, Ref, Schema, Type};
pub use self::security::SecurityRequirement;
pub use self::server::{Server, ServerBuilder, ServerVariable, ServerVariableBuilder};
pub use self::tag::Tag;

pub mod content;
pub mod encoding;
pub mod example;
pub mod extensions;
pub mod external_docs;
pub mod header;
pub mod info;
pub mod link;
pub mod path;
pub mod request_body;
pub mod response;
pub mod schema;
pub mod security;
pub mod server;
pub mod tag;
pub mod xml;

/// Root object of the OpenAPI document.
///
/// You can use [`OpenApi::new`] function to construct a new [`OpenApi`] instance and then
/// use the fields with mutable access to modify them. This is quite tedious if you are not simply
/// just changing one thing thus you can also use the [`OpenApi::builder`] to use builder to
/// construct a new [`OpenApi`] object.
///
/// See more details at <https://spec.openapis.org/oas/latest.html#openapi-object>.
#[non_exhaustive]
#[derive(serde_derive::Serialize, serde_derive::Deserialize, Default, Clone, PartialEq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
#[builder(on(_, into))]
pub struct OpenApi {
    /// OpenAPI document version.
    #[builder(default)]
    pub openapi: OpenApiVersion,

    /// Provides metadata about the API.
    ///
    /// See more details at <https://spec.openapis.org/oas/latest.html#info-object>.
    #[builder(default)]
    pub info: Info,

    /// Optional list of servers that provides the connectivity information to target servers.
    ///
    /// This is implicitly one server with `url` set to `/`.
    ///
    /// See more details at <https://spec.openapis.org/oas/latest.html#server-object>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<Server>>,

    /// Available paths and operations for the API.
    ///
    /// See more details at <https://spec.openapis.org/oas/latest.html#paths-object>.
    #[builder(default)]
    pub paths: Paths,

    /// Holds various reusable schemas for the OpenAPI document.
    ///
    /// Few of these elements are security schemas and object schemas.
    ///
    /// See more details at <https://spec.openapis.org/oas/latest.html#components-object>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Components>,

    /// Declaration of global security mechanisms that can be used across the API. The individual operations
    /// can override the declarations. You can use `SecurityRequirement::default()` if you wish to make security
    /// optional by adding it to the list of securities.
    ///
    /// See more details at <https://spec.openapis.org/oas/latest.html#security-requirement-object>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<SecurityRequirement>>,

    /// Optional list of tags can be used to add additional documentation to matching tags of operations.
    ///
    /// See more details at <https://spec.openapis.org/oas/latest.html#tag-object>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,

    /// Optional global additional documentation reference.
    ///
    /// See more details at <https://spec.openapis.org/oas/latest.html#external-documentation-object>.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_docs: Option<ExternalDocs>,

    /// Schema keyword can be used to override default _`$schema`_ dialect which is by default
    /// "<https://spec.openapis.org/oas/3.1/dialect/base>".
    ///
    /// All the references and individual files could use their own schema dialect.
    #[serde(rename = "$schema", default, skip_serializing_if = "String::is_empty")]
    #[builder(default)]
    pub schema: String,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", flatten)]
    pub extensions: Option<Extensions>,
}

impl OpenApi {
    /// Construct a new [`OpenApi`] object.
    ///
    /// Function accepts two arguments one which is [`Info`] metadata of the API; two which is [`Paths`]
    /// containing operations for the API.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use openapiv3_1::{Info, Paths, OpenApi};
    /// #
    /// let openapi = OpenApi::new(Info::new("pet api", "0.1.0"), Paths::new());
    /// ```
    pub fn new(info: impl Into<Info>, paths: impl Into<Paths>) -> Self {
        Self {
            info: info.into(),
            paths: paths.into(),
            ..Default::default()
        }
    }

    /// Converts this [`OpenApi`] to JSON String. This method essentially calls [`serde_json::to_string`] method.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Converts this [`OpenApi`] to pretty JSON String. This method essentially calls [`serde_json::to_string_pretty`] method.
    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Converts this [`OpenApi`] to YAML String. This method essentially calls [`serde_norway::to_string`] method.
    #[cfg(feature = "yaml")]
    #[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
    pub fn to_yaml(&self) -> Result<String, serde_norway::Error> {
        serde_norway::to_string(self)
    }

    /// Merge `other` [`OpenApi`] moving `self` and returning combined [`OpenApi`].
    ///
    /// In functionality wise this is exactly same as calling [`OpenApi::merge`] but but provides
    /// leaner API for chaining method calls.
    pub fn merge_from(mut self, other: OpenApi) -> OpenApi {
        self.merge(other);
        self
    }

    /// Merge `other` [`OpenApi`] consuming it and resuming it's content.
    ///
    /// Merge function will take all `self` nonexistent _`servers`, `paths`, `schemas`, `responses`,
    /// `security_schemes`, `security_requirements` and `tags`_ from _`other`_ [`OpenApi`].
    ///
    /// This function performs a shallow comparison for `paths`, `schemas`, `responses` and
    /// `security schemes` which means that only _`name`_ and _`path`_ is used for comparison. When
    /// match occurs the whole item will be ignored from merged results. Only items not
    /// found will be appended to `self`.
    ///
    /// For _`servers`_, _`tags`_ and _`security_requirements`_ the whole item will be used for
    /// comparison. Items not found from `self` will be appended to `self`.
    ///
    /// **Note!** `info`, `openapi`, `external_docs` and `schema` will not be merged.
    pub fn merge(&mut self, mut other: OpenApi) {
        if let Some(other_servers) = &mut other.servers {
            let servers = self.servers.get_or_insert(Vec::new());
            other_servers.retain(|server| !servers.contains(server));
            servers.append(other_servers);
        }

        if !other.paths.paths.is_empty() {
            self.paths.merge(other.paths);
        };

        if let Some(other_components) = &mut other.components {
            let components = self.components.get_or_insert(Components::default());

            other_components
                .schemas
                .retain(|name, _| !components.schemas.contains_key(name));
            components.schemas.append(&mut other_components.schemas);

            other_components
                .responses
                .retain(|name, _| !components.responses.contains_key(name));
            components.responses.append(&mut other_components.responses);

            other_components
                .security_schemes
                .retain(|name, _| !components.security_schemes.contains_key(name));
            components.security_schemes.append(&mut other_components.security_schemes);
        }

        if let Some(other_security) = &mut other.security {
            let security = self.security.get_or_insert(Vec::new());
            other_security.retain(|requirement| !security.contains(requirement));
            security.append(other_security);
        }

        if let Some(other_tags) = &mut other.tags {
            let tags = self.tags.get_or_insert(Vec::new());
            other_tags.retain(|tag| !tags.contains(tag));
            tags.append(other_tags);
        }
    }

    /// Nest `other` [`OpenApi`] to this [`OpenApi`].
    ///
    /// Nesting performs custom [`OpenApi::merge`] where `other` [`OpenApi`] paths are prepended with given
    /// `path` and then appended to _`paths`_ of this [`OpenApi`] instance. Rest of the  `other`
    /// [`OpenApi`] instance is merged to this [`OpenApi`] with [`OpenApi::merge_from`] method.
    ///
    /// **If multiple** APIs are being nested with same `path` only the **last** one will be retained.
    ///
    /// Method accepts two arguments, first is the path to prepend .e.g. _`/user`_. Second argument
    /// is the [`OpenApi`] to prepend paths for.
    ///
    /// # Examples
    ///
    /// _**Merge `user_api` to `api` nesting `user_api` paths under `/api/v1/user`**_
    /// ```rust
    ///  # use openapiv3_1::{OpenApi, OpenApiBuilder};
    ///  # use openapiv3_1::path::{Paths, PathItem,
    ///  # HttpMethod, Operation};
    ///  let api = OpenApi::builder()
    ///      .paths(
    ///          Paths::builder().path(
    ///              "/api/v1/status",
    ///              PathItem::new(
    ///                  HttpMethod::Get,
    ///                  Operation::builder()
    ///                      .description("Get status")
    ///                      .build(),
    ///              ),
    ///          ),
    ///      )
    ///      .build();
    ///  let user_api = OpenApi::builder()
    ///     .paths(
    ///         Paths::builder().path(
    ///             "/",
    ///             PathItem::new(HttpMethod::Post, Operation::builder().build()),
    ///         )
    ///     )
    ///     .build();
    ///  let nested = api.nest("/api/v1/user", user_api);
    /// ```
    pub fn nest<P: Into<String>, O: Into<OpenApi>>(self, path: P, other: O) -> Self {
        self.nest_with_path_composer(path, other, |base, path| format!("{base}{path}"))
    }

    /// Nest `other` [`OpenApi`] with custom path composer.
    ///
    /// In most cases you should use [`OpenApi::nest`] instead.
    /// Only use this method if you need custom path composition for a specific use case.
    ///
    /// `composer` is a function that takes two strings, the base path and the path to nest, and returns the composed path for the API Specification.
    pub fn nest_with_path_composer<P: Into<String>, O: Into<OpenApi>, F: Fn(&str, &str) -> String>(
        mut self,
        path: P,
        other: O,
        composer: F,
    ) -> Self {
        let path: String = path.into();
        let mut other_api: OpenApi = other.into();

        let nested_paths = other_api.paths.paths.into_iter().map(|(item_path, item)| {
            let path = composer(&path, &item_path);
            (path, item)
        });

        self.paths.paths.extend(nested_paths);

        // paths are already merged, thus we can ignore them
        other_api.paths.paths = IndexMap::new();
        self.merge_from(other_api)
    }
}

/// Represents available [OpenAPI versions][version].
///
/// [version]: <https://spec.openapis.org/oas/latest.html#versions>
#[derive(Serialize, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum OpenApiVersion {
    /// Will serialize to `3.1.0` the latest released OpenAPI version.
    #[serde(rename = "3.1.0")]
    #[default]
    Version31,
}

impl<'de> serde::Deserialize<'de> for OpenApiVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VersionVisitor;

        impl<'v> Visitor<'v> for VersionVisitor {
            type Value = OpenApiVersion;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a version string in 3.1.x format")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_string(v.to_string())
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let version = v.split('.').flat_map(|digit| digit.parse::<i8>()).collect::<Vec<_>>();

                if version.len() == 3 && version.first() == Some(&3) && version.get(1) == Some(&1) {
                    Ok(OpenApiVersion::Version31)
                } else {
                    let expected: &dyn Expected = &"3.1.0";
                    Err(Error::invalid_value(serde::de::Unexpected::Str(&v), expected))
                }
            }
        }

        deserializer.deserialize_string(VersionVisitor)
    }
}

/// Value used to indicate whether reusable schema, parameter or operation is deprecated.
///
/// The value will serialize to boolean.
#[derive(PartialEq, Eq, Clone, Default)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[allow(missing_docs)]
pub enum Deprecated {
    True,
    #[default]
    False,
}

impl serde::Serialize for Deprecated {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(matches!(self, Self::True))
    }
}

impl<'de> serde::Deserialize<'de> for Deprecated {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct BoolVisitor;
        impl<'de> Visitor<'de> for BoolVisitor {
            type Value = Deprecated;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a bool true or false")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    true => Ok(Deprecated::True),
                    false => Ok(Deprecated::False),
                }
            }
        }
        deserializer.deserialize_bool(BoolVisitor)
    }
}

/// A [`Ref`] or some other type `T`.
///
/// Typically used in combination with [`Components`] and is an union type between [`Ref`] and any
/// other given type such as [`Schema`] or [`Response`].
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(untagged)]
pub enum RefOr<T> {
    /// Represents [`Ref`] reference to another OpenAPI object instance. e.g.
    /// `$ref: #/components/schemas/Hello`
    Ref(Ref),
    /// Represents any value that can be added to the [`struct@Components`] e.g. [`enum@Schema`]
    /// or [`struct@Response`].
    T(T),
}

use crate::extensions::Extensions;

/// Changelogs generated by [scuffle_changelog]
#[cfg(feature = "docs")]
#[scuffle_changelog::changelog]
pub mod changelog {}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use insta::assert_json_snapshot;

    use super::response::Response;
    use super::*;
    use crate::path::Operation;

    #[test]
    fn serialize_deserialize_openapi_version_success() -> Result<(), serde_json::Error> {
        assert_eq!(serde_json::to_value(&OpenApiVersion::Version31)?, "3.1.0");
        Ok(())
    }

    #[test]
    fn serialize_openapi_json_minimal_success() {
        let openapi = OpenApi::new(
            Info::builder()
                .title("My api")
                .version("1.0.0")
                .description("My api description")
                .license(License::builder().name("MIT").url("http://mit.licence")),
            Paths::new(),
        );

        assert_json_snapshot!(openapi);
    }

    #[test]
    fn serialize_openapi_json_with_paths_success() {
        let openapi = OpenApi::new(
            Info::new("My big api", "1.1.0"),
            Paths::builder()
                .path(
                    "/api/v1/users",
                    PathItem::new(
                        HttpMethod::Get,
                        Operation::builder().response("200", Response::new("Get users list")),
                    ),
                )
                .path(
                    "/api/v1/users",
                    PathItem::new(
                        HttpMethod::Post,
                        Operation::builder().response("200", Response::new("Post new user")),
                    ),
                )
                .path(
                    "/api/v1/users/{id}",
                    PathItem::new(
                        HttpMethod::Get,
                        Operation::builder().response("200", Response::new("Get user by id")),
                    ),
                ),
        );

        assert_json_snapshot!(openapi);
    }

    #[test]
    fn merge_2_openapi_documents() {
        let mut api_1 = OpenApi::new(
            Info::new("Api", "v1"),
            Paths::builder()
                .path(
                    "/api/v1/user",
                    PathItem::new(
                        HttpMethod::Get,
                        Operation::builder().response("200", Response::new("Get user success")),
                    ),
                )
                .build(),
        );

        let api_2 = OpenApi::builder()
            .info(Info::new("Api", "v2"))
            .paths(
                Paths::builder()
                    .path(
                        "/api/v1/user",
                        PathItem::new(
                            HttpMethod::Get,
                            Operation::builder().response("200", Response::new("This will not get added")),
                        ),
                    )
                    .path(
                        "/ap/v2/user",
                        PathItem::new(
                            HttpMethod::Get,
                            Operation::builder().response("200", Response::new("Get user success 2")),
                        ),
                    )
                    .path(
                        "/api/v2/user",
                        PathItem::new(
                            HttpMethod::Post,
                            Operation::builder().response("200", Response::new("Get user success")),
                        ),
                    )
                    .build(),
            )
            .components(
                Components::builder().schema(
                    "User2",
                    Object::builder()
                        .schema_type(Type::Object)
                        .property("name", Object::builder().schema_type(Type::String)),
                ),
            )
            .build();

        api_1.merge(api_2);

        assert_json_snapshot!(api_1, {
            ".paths" => insta::sorted_redaction()
        });
    }

    #[test]
    fn merge_same_path_diff_methods() {
        let mut api_1 = OpenApi::new(
            Info::new("Api", "v1"),
            Paths::builder()
                .path(
                    "/api/v1/user",
                    PathItem::new(
                        HttpMethod::Get,
                        Operation::builder().response("200", Response::new("Get user success 1")),
                    ),
                )
                .extensions(Extensions::from_iter([("x-v1-api", true)]))
                .build(),
        );

        let api_2 = OpenApi::builder()
            .info(Info::new("Api", "v2"))
            .paths(
                Paths::builder()
                    .path(
                        "/api/v1/user",
                        PathItem::new(
                            HttpMethod::Get,
                            Operation::builder().response("200", Response::new("This will not get added")),
                        ),
                    )
                    .path(
                        "/api/v1/user",
                        PathItem::new(
                            HttpMethod::Post,
                            Operation::builder().response("200", Response::new("Post user success 1")),
                        ),
                    )
                    .path(
                        "/api/v2/user",
                        PathItem::new(
                            HttpMethod::Get,
                            Operation::builder().response("200", Response::new("Get user success 2")),
                        ),
                    )
                    .path(
                        "/api/v2/user",
                        PathItem::new(
                            HttpMethod::Post,
                            Operation::builder().response("200", Response::new("Post user success 2")),
                        ),
                    )
                    .extensions(Extensions::from_iter([("x-random", "Value")])),
            )
            .components(
                Components::builder().schema(
                    "User2",
                    Object::builder()
                        .schema_type(Type::Object)
                        .property("name", Object::builder().schema_type(Type::String)),
                ),
            )
            .build();

        api_1.merge(api_2);

        assert_json_snapshot!(api_1, {
            ".paths" => insta::sorted_redaction()
        });
    }

    #[test]
    fn test_nest_open_apis() {
        let api = OpenApi::builder()
            .paths(Paths::builder().path(
                "/api/v1/status",
                PathItem::new(HttpMethod::Get, Operation::builder().description("Get status")),
            ))
            .build();

        let user_api = OpenApi::builder()
            .paths(
                Paths::builder()
                    .path(
                        "/",
                        PathItem::new(HttpMethod::Get, Operation::builder().description("Get user details").build()),
                    )
                    .path("/foo", PathItem::new(HttpMethod::Post, Operation::builder().build())),
            )
            .build();

        let nest_merged = api.nest("/api/v1/user", user_api);
        let value = serde_json::to_value(nest_merged).expect("should serialize as json");
        let paths = value.pointer("/paths").expect("paths should exits in openapi");

        assert_json_snapshot!(paths);
    }

    #[test]
    fn openapi_custom_extension() {
        let mut api = OpenApi::builder().build();
        let extensions = api.extensions.get_or_insert(Default::default());
        extensions.insert(
            String::from("x-tagGroup"),
            String::from("anything that serializes to Json").into(),
        );

        assert_json_snapshot!(api);
    }
}

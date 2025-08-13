//! Implements [OpenAPI Header Object][header] types.
//!
//! [header]: https://spec.openapis.org/oas/latest.html#header-object

use super::{Object, Schema, Type};

/// Implements [OpenAPI Header Object][header] for response headers.
///
/// [header]: https://spec.openapis.org/oas/latest.html#header-object
#[non_exhaustive]
#[derive(serde_derive::Serialize, serde_derive::Deserialize, Clone, PartialEq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[builder(on(_, into))]
pub struct Header {
    /// Schema of header type.
    pub schema: Schema,

    /// Additional description of the header value.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
}

impl Header {
    /// Construct a new [`Header`] with custom schema. If you wish to construct a default
    /// header with `String` type you can use [`Header::default`] function.
    ///
    /// # Examples
    ///
    /// Create new [`Header`] with integer type.
    /// ```rust
    /// # use openapiv3_1::header::Header;
    /// # use openapiv3_1::{Object, Type};
    /// let header = Header::new(Object::with_type(Type::Integer));
    /// ```
    ///
    /// Create a new [`Header`] with default type `String`
    /// ```rust
    /// # use openapiv3_1::header::Header;
    /// let header = Header::default();
    /// ```
    pub fn new<C: Into<Schema>>(component: C) -> Self {
        Self {
            schema: component.into(),
            ..Default::default()
        }
    }
}

impl Default for Header {
    fn default() -> Self {
        Self {
            description: Default::default(),
            schema: Object::builder().schema_type(Type::String).into(),
        }
    }
}

impl<S: header_builder::IsComplete> From<HeaderBuilder<S>> for Header {
    fn from(builder: HeaderBuilder<S>) -> Self {
        builder.build()
    }
}

//! Implements [OpenAPI Metadata][info] types.
//!
//! [info]: <https://spec.openapis.org/oas/latest.html#info-object>

use serde_derive::{Deserialize, Serialize};

use super::extensions::Extensions;

/// OpenAPI [Info][info] object represents metadata of the API.
///
/// You can use [`Info::new`] to construct a new [`Info`] object or alternatively use [`Info::builder`]
/// to construct a new [`Info`] with chainable configuration methods.
///
/// [info]: <https://spec.openapis.org/oas/latest.html#info-object>
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
#[builder(on(_, into))]
pub struct Info {
    /// Title of the API.
    pub title: String,

    /// Optional description of the API.
    ///
    /// Value supports markdown syntax.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,

    /// Optional url for terms of service.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub terms_of_service: Option<String>,

    /// Contact information of exposed API.
    ///
    /// See more details at: <https://spec.openapis.org/oas/latest.html#contact-object>.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub contact: Option<Contact>,

    /// License of the API.
    ///
    /// See more details at: <https://spec.openapis.org/oas/latest.html#license-object>.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub license: Option<License>,

    /// Document version typically the API version.
    pub version: String,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", default, flatten)]
    pub extensions: Option<Extensions>,
}

impl<S: info_builder::IsComplete> From<InfoBuilder<S>> for Info {
    fn from(builder: InfoBuilder<S>) -> Self {
        builder.build()
    }
}

impl Info {
    /// Construct a new [`Info`] object.
    ///
    /// This function accepts two arguments. One which is the title of the API and two the
    /// version of the api document typically the API version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use openapiv3_1::Info;
    /// let info = Info::new("Pet api", "1.1.0");
    /// ```
    pub fn new<S: Into<String>>(title: S, version: S) -> Self {
        Self {
            title: title.into(),
            version: version.into(),
            ..Default::default()
        }
    }
}

/// OpenAPI [`Contact`][contact] object represents metadata of the API.
///
/// You can use [`Contact::new`] to construct a new [`Contact`] object or alternatively use [`Contact::builder`]
/// to construct a new [`Contact`] with chainable configuration methods.
///
/// [contact]: <https://spec.openapis.org/oas/latest.html#contact-object>
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
#[builder(on(_, into))]
pub struct Contact {
    /// Identifying name of the contact person or organization of the API.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,

    /// Url pointing to contact information of the API.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub url: Option<String>,

    /// Email of the contact person or the organization of the API.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub email: Option<String>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", default, flatten)]
    pub extensions: Option<Extensions>,
}

impl Contact {
    /// Construct a new [`Contact`].
    pub fn new() -> Self {
        Default::default()
    }
}

impl<S: contact_builder::IsComplete> From<ContactBuilder<S>> for Contact {
    fn from(builder: ContactBuilder<S>) -> Self {
        builder.build()
    }
}

/// OpenAPI [License][license] information of the API.
///
/// [license]: <https://spec.openapis.org/oas/latest.html#license-object>
#[non_exhaustive]
#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Eq, bon::Builder)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[serde(rename_all = "camelCase")]
#[builder(on(_, into))]
pub struct License {
    /// Name of the license used e.g MIT or Apache-2.0.
    pub name: String,

    /// Optional url pointing to the license.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub url: Option<String>,

    /// An [SPDX-Licenses][spdx_licence] expression for the API. The _`identifier`_ field
    /// is mutually exclusive of the _`url`_ field. E.g. Apache-2.0
    ///
    /// [spdx_licence]: <https://spdx.org/licenses/>
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub identifier: Option<String>,

    /// Optional extensions "x-something".
    #[serde(skip_serializing_if = "Option::is_none", default, flatten)]
    pub extensions: Option<Extensions>,
}

impl License {
    /// Construct a new [`License`] object.
    ///
    /// Function takes name of the license as an argument e.g MIT.
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
}

impl<S: license_builder::IsComplete> From<LicenseBuilder<S>> for License {
    fn from(builder: LicenseBuilder<S>) -> Self {
        builder.build()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::Contact;

    #[test]
    fn contact_new() {
        let contact = Contact::new();

        assert!(contact.name.is_none());
        assert!(contact.url.is_none());
        assert!(contact.email.is_none());
    }
}

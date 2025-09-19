//! Core email template rendering.
//!
//! ## License
//!
//! This project is licensed under the [AGPL-3.0](./LICENSE.AGPL-3.0).
//!
//! `SPDX-License-Identifier: AGPL-3.0`
#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
// #![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]
#![deny(clippy::mod_module_files)]

use chrono::Datelike;
use sailfish::{TemplateOnce, TemplateSimple};

pub struct Email {
    pub to_address: String,
    pub from_address: String,
    pub subject: String,
    pub text: String,
    pub html: String,
}

#[derive(sailfish::TemplateSimple)]
#[template(path = "emails/register_with_email/subject.stpl")]
struct RegisterWithEmailSubjectTemplate;

#[derive(sailfish::Template)]
#[template(path = "emails/register_with_email/text.stpl")]
struct RegisterWithEmailTextTemplate {
    url: String,
    timeout_minutes: u32,
    copyright_year: u32,
}

#[derive(sailfish::Template)]
#[template(path = "emails/register_with_email/html.stpl")]
struct RegisterWithEmailHtmlTemplate {
    url: String,
    timeout_minutes: u32,
    copyright_year: u32,
}

pub fn register_with_email_email(
    from_address: String,
    to_address: String,
    dashboard_origin: &url::Url,
    code: String,
    timeout: std::time::Duration,
) -> Result<Email, sailfish::RenderError> {
    let url = dashboard_origin
        .join(&format!("/register/confirm?code={code}"))
        .unwrap()
        .to_string();

    let timeout_minutes = timeout.as_secs() as u32 / 60;
    let copyright_year = chrono::Utc::now().year() as u32;

    let subject = RegisterWithEmailSubjectTemplate.render_once()?;
    let text = RegisterWithEmailTextTemplate {
        url: url.clone(),
        timeout_minutes,
        copyright_year,
    }
    .render_once()?;
    let html = RegisterWithEmailHtmlTemplate {
        url,
        timeout_minutes,
        copyright_year,
    }
    .render_once()?;

    Ok(Email {
        to_address,
        from_address,
        subject,
        text,
        html,
    })
}

#[derive(sailfish::TemplateSimple)]
#[template(path = "emails/add_new_email/subject.stpl")]
struct AddNewEmailSubjectTemplate;

#[derive(sailfish::Template)]
#[template(path = "emails/add_new_email/text.stpl")]
struct AddNewEmailTextTemplate {
    url: String,
    timeout_minutes: u32,
    copyright_year: u32,
}

#[derive(sailfish::Template)]
#[template(path = "emails/add_new_email/html.stpl")]
struct AddNewEmailHtmlTemplate {
    url: String,
    timeout_minutes: u32,
    copyright_year: u32,
}

pub fn add_new_email_email(
    from_address: String,
    to_address: String,
    dashboard_origin: &url::Url,
    code: String,
    timeout: std::time::Duration,
) -> Result<Email, sailfish::RenderError> {
    let url = dashboard_origin
        .join(&format!("/settings/emails/confirm?code={code}"))
        .unwrap()
        .to_string();
    let timeout_minutes = timeout.as_secs() as u32 / 60;
    let copyright_year = chrono::Utc::now().year() as u32;

    let subject = AddNewEmailSubjectTemplate.render_once()?;
    let text = AddNewEmailTextTemplate {
        url: url.clone(),
        timeout_minutes,
        copyright_year,
    }
    .render_once()?;
    let html = AddNewEmailHtmlTemplate {
        url,
        timeout_minutes,
        copyright_year,
    }
    .render_once()?;

    Ok(Email {
        to_address,
        from_address,
        subject,
        text,
        html,
    })
}

#[derive(sailfish::TemplateSimple)]
#[template(path = "emails/magic_link/subject.stpl")]
struct MagicLinkSubjectTemplate;

#[derive(sailfish::Template)]
#[template(path = "emails/magic_link/text.stpl")]
struct MagicLinkTextTemplate {
    url: String,
    timeout_minutes: u32,
    copyright_year: u32,
}

#[derive(sailfish::Template)]
#[template(path = "emails/magic_link/html.stpl")]
struct MagicLinkHtmlTemplate {
    url: String,
    timeout_minutes: u32,
    copyright_year: u32,
}

pub fn magic_link_email(
    from_address: String,
    to_address: String,
    dashboard_origin: &url::Url,
    code: String,
    timeout: std::time::Duration,
) -> Result<Email, sailfish::RenderError> {
    let url = dashboard_origin
        .join(&format!("/login/magic-link?code={code}"))
        .unwrap()
        .to_string();
    let timeout_minutes = timeout.as_secs() as u32 / 60;
    let copyright_year = chrono::Utc::now().year() as u32;

    let subject = MagicLinkSubjectTemplate.render_once()?;
    let text = MagicLinkTextTemplate {
        url: url.clone(),
        timeout_minutes,
        copyright_year,
    }
    .render_once()?;
    let html = MagicLinkHtmlTemplate {
        url,
        timeout_minutes,
        copyright_year,
    }
    .render_once()?;

    Ok(Email {
        to_address,
        from_address,
        subject,
        text,
        html,
    })
}

#[derive(sailfish::TemplateSimple)]
#[template(path = "emails/new_device/subject.stpl")]
struct NewDeviceSubjectTemplate;

#[derive(Clone, Debug, Default)]
pub struct GeoInfo {
    pub city: Option<String>,
    pub country: Option<String>,
}

impl GeoInfo {
    fn is_empty(&self) -> bool {
        self.city.is_none() && self.country.is_none()
    }
}

impl From<maxminddb::geoip2::City<'_>> for GeoInfo {
    fn from(value: maxminddb::geoip2::City) -> GeoInfo {
        let city = value
            .city
            .and_then(|c| c.names)
            .and_then(|names| names.get("en").map(|s| s.to_string()));
        let country = value
            .country
            .and_then(|c| c.names)
            .and_then(|names| names.get("en").map(|s| s.to_string()));

        GeoInfo { city, country }
    }
}

#[derive(sailfish::Template)]
#[template(path = "emails/new_device/text.stpl")]
struct NewDeviceTextTemplate {
    activity_url: String,
    ip_address: String,
    geo_info: GeoInfo,
    copyright_year: u32,
}

#[derive(sailfish::Template)]
#[template(path = "emails/new_device/html.stpl")]
struct NewDeviceHtmlTemplate {
    activity_url: String,
    ip_address: String,
    geo_info: GeoInfo,
    copyright_year: u32,
}

pub fn new_device_email(
    from_address: String,
    to_address: String,
    dashboard_origin: &url::Url,
    ip_addr: std::net::IpAddr,
    geo_info: GeoInfo,
) -> Result<Email, sailfish::RenderError> {
    let ip_address = ip_addr.to_string();
    // TODO: replace with actual link
    let activity_url = dashboard_origin.join("/activity").unwrap().to_string();
    let copyright_year = chrono::Utc::now().year() as u32;

    let subject = NewDeviceSubjectTemplate.render_once()?;

    let text = NewDeviceTextTemplate {
        ip_address: ip_address.clone(),
        geo_info: geo_info.clone(),
        activity_url: activity_url.clone(),
        copyright_year,
    }
    .render_once()?;

    let html = NewDeviceHtmlTemplate {
        ip_address,
        geo_info,
        activity_url,
        copyright_year,
    }
    .render_once()?;

    Ok(Email {
        to_address,
        from_address,
        subject,
        text,
        html,
    })
}

use std::sync::Arc;

use core_traits::ResultExt;
use geo_ip::maxminddb;
use geo_ip::middleware::IpAddressInfo;
use sailfish::{TemplateOnce, TemplateSimple};

#[derive(sailfish::TemplateSimple)]
#[template(path = "emails/register_with_email/subject.stpl")]
struct RegisterWithEmailSubjectTemplate;

#[derive(sailfish::Template)]
#[template(path = "emails/register_with_email/text.stpl")]
struct RegisterWithEmailTextTemplate {
    pub url: String,
    pub timeout_minutes: u32,
}

#[derive(sailfish::Template)]
#[template(path = "emails/register_with_email/html.stpl")]
struct RegisterWithEmailHtmlTemplate {
    pub url: String,
    pub timeout_minutes: u32,
}

pub(crate) async fn register_with_email_email<G: core_traits::Global>(
    global: &Arc<G>,
    to_address: String,
    code: String,
) -> Result<pb::scufflecloud::email::v1::Email, sailfish::RenderError> {
    let url = global
        .dashboard_origin()
        .join(&format!("/register/confirm?code={code}"))
        .unwrap()
        .to_string();

    let timeout_minutes = global.timeout_config().email_registration_request.num_minutes().max(0) as u32;

    let subject = RegisterWithEmailSubjectTemplate.render_once()?;
    let text = RegisterWithEmailTextTemplate {
        url: url.clone(),
        timeout_minutes,
    }
    .render_once()?;
    let html = RegisterWithEmailHtmlTemplate { url, timeout_minutes }.render_once()?;

    Ok(pb::scufflecloud::email::v1::Email {
        to_address,
        source_address: global.email_from_address().to_string(),
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
    pub url: String,
}

#[derive(sailfish::Template)]
#[template(path = "emails/add_new_email/html.stpl")]
struct AddNewEmailHtmlTemplate {
    pub url: String,
}

pub(crate) async fn add_new_email_email<G: core_traits::Global>(
    global: &Arc<G>,
    to_address: String,
    code: String,
) -> Result<pb::scufflecloud::email::v1::Email, sailfish::RenderError> {
    let url = global
        .dashboard_origin()
        .join(&format!("/settings/emails/confirm?code={code}"))
        .unwrap()
        .to_string();

    let subject = AddNewEmailSubjectTemplate.render_once()?;
    let text = AddNewEmailTextTemplate { url: url.clone() }.render_once()?;
    let html = AddNewEmailHtmlTemplate { url }.render_once()?;

    Ok(pb::scufflecloud::email::v1::Email {
        to_address,
        source_address: global.email_from_address().to_string(),
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
    pub url: String,
    pub timeout_minutes: u32,
}

#[derive(sailfish::Template)]
#[template(path = "emails/magic_link/html.stpl")]
struct MagicLinkHtmlTemplate {
    pub url: String,
    pub timeout_minutes: u32,
}

pub(crate) async fn magic_link_email<G: core_traits::Global>(
    global: &Arc<G>,
    to_address: String,
    code: String,
) -> Result<pb::scufflecloud::email::v1::Email, sailfish::RenderError> {
    let url = global
        .dashboard_origin()
        .join(&format!("/login/magic-link?code={code}"))
        .unwrap()
        .to_string();
    let timeout_minutes = global.timeout_config().magic_link_request.num_minutes().max(0) as u32;

    let subject = MagicLinkSubjectTemplate.render_once()?;
    let text = MagicLinkTextTemplate {
        url: url.clone(),
        timeout_minutes,
    }
    .render_once()?;
    let html = MagicLinkHtmlTemplate { url, timeout_minutes }.render_once()?;

    Ok(pb::scufflecloud::email::v1::Email {
        to_address,
        source_address: global.email_from_address().to_string(),
        subject,
        text,
        html,
    })
}

#[derive(sailfish::TemplateSimple)]
#[template(path = "emails/new_device/subject.stpl")]
struct NewDeviceSubjectTemplate;

#[derive(Clone)]
struct GeoInfo {
    city: Option<String>,
    country: Option<String>,
}

impl GeoInfo {
    fn from_city(value: maxminddb::geoip2::City) -> Option<GeoInfo> {
        let city = value
            .city
            .and_then(|c| c.names)
            .and_then(|names| names.get("en").map(|s| s.to_string()));
        let country = value
            .country
            .and_then(|c| c.names)
            .and_then(|names| names.get("en").map(|s| s.to_string()));

        match (city, country) {
            (None, None) => None,
            (city, country) => Some(GeoInfo { city, country }),
        }
    }
}

#[derive(sailfish::Template)]
#[template(path = "emails/new_device/text.stpl")]
struct NewDeviceTextTemplate {
    pub activity_url: String,
    pub ip_address: String,
    pub geo_info: Option<GeoInfo>,
}

#[derive(sailfish::Template)]
#[template(path = "emails/new_device/html.stpl")]
struct NewDeviceHtmlTemplate {
    pub activity_url: String,
    pub ip_address: String,
    pub geo_info: Option<GeoInfo>,
}

pub(crate) async fn new_device_email<G: core_traits::Global>(
    global: &Arc<G>,
    to_address: String,
    ip_info: &IpAddressInfo,
) -> Result<pb::scufflecloud::email::v1::Email, tonic::Status> {
    let geo_info: Option<GeoInfo> = ip_info
        .lookup_geoip_info::<maxminddb::geoip2::City>(global.as_ref())
        .into_tonic_internal_err("failed to lookup geo location data")?
        .and_then(GeoInfo::from_city);
    let ip_address = ip_info.ip_address.to_string();
    // TODO: replace with actual link
    let activity_url = global
        .dashboard_origin()
        .join("/activity")
        .unwrap()
        .to_string();

    let subject = NewDeviceSubjectTemplate
        .render_once()
        .into_tonic_internal_err("failed to render email")?;
    let text = NewDeviceTextTemplate {
        ip_address: ip_address.clone(),
        geo_info: geo_info.clone(),
        activity_url: activity_url.clone(),
    }
    .render_once()
    .into_tonic_internal_err("failed to render email")?;
    let html = NewDeviceHtmlTemplate { ip_address, geo_info, activity_url }
        .render_once()
        .into_tonic_internal_err("failed to render email")?;

    Ok(pb::scufflecloud::email::v1::Email {
        to_address,
        source_address: global.email_from_address().to_string(),
        subject,
        text,
        html,
    })
}

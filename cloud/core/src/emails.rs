use std::sync::Arc;

use sailfish::{TemplateOnce, TemplateSimple};

use crate::CoreConfig;

#[derive(sailfish::TemplateSimple)]
#[template(path = "emails/register_with_email/subject.stpl")]
struct RegisterWithEmailSubjectTemplate;

#[derive(sailfish::Template)]
#[template(path = "emails/register_with_email/text.stpl")]
#[allow(dead_code)]
struct RegisterWithEmailTextTemplate {
    pub url: String,
}

#[derive(sailfish::Template)]
#[template(path = "emails/register_with_email/html.stpl")]
#[allow(dead_code)]
struct RegisterWithEmailHtmlTemplate {
    pub url: String,
}

pub(crate) async fn register_with_email_email<G: CoreConfig>(
    global: &Arc<G>,
    to_address: String,
    code: String,
) -> Result<pb::scufflecloud::email::v1::Email, sailfish::RenderError> {
    let url = format!("{}/register/confirm?code={}", global.dashboard_origin(), code);

    let subject = RegisterWithEmailSubjectTemplate.render_once()?;
    let text = RegisterWithEmailTextTemplate { url: url.to_string() }.render_once()?;
    let html = RegisterWithEmailHtmlTemplate { url }.render_once()?;

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
#[allow(dead_code)]
struct AddNewEmailTextTemplate {
    pub url: String,
}

#[derive(sailfish::Template)]
#[template(path = "emails/add_new_email/html.stpl")]
#[allow(dead_code)]
struct AddNewEmailHtmlTemplate {
    pub url: String,
}

pub(crate) async fn add_new_email_email<G: CoreConfig>(
    global: &Arc<G>,
    to_address: String,
    code: String,
) -> Result<pb::scufflecloud::email::v1::Email, sailfish::RenderError> {
    let url = format!("{}/settings/emails/confirm?code={}", global.dashboard_origin(), code);

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
#[allow(dead_code)]
struct MagicLinkTextTemplate {
    pub url: String,
}

#[derive(sailfish::Template)]
#[template(path = "emails/magic_link/html.stpl")]
#[allow(dead_code)]
struct MagicLinkHtmlTemplate {
    pub url: String,
}

pub(crate) async fn magic_link_email<G: CoreConfig>(
    global: &Arc<G>,
    to_address: String,
    code: String,
) -> Result<pb::scufflecloud::email::v1::Email, sailfish::RenderError> {
    let url = format!("{}/login/magic-link?code={}", global.dashboard_origin(), code);

    let subject = MagicLinkSubjectTemplate.render_once()?;
    let text = MagicLinkTextTemplate { url: url.clone() }.render_once()?;
    let html = MagicLinkHtmlTemplate { url }.render_once()?;

    Ok(pb::scufflecloud::email::v1::Email {
        to_address,
        source_address: global.email_from_address().to_string(),
        subject,
        text,
        html,
    })
}

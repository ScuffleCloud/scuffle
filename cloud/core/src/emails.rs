use std::sync::Arc;

use sailfish::{TemplateOnce, TemplateSimple};

use crate::CoreConfig;

#[derive(sailfish::TemplateSimple)]
#[template(path = "emails/register_with_email/subject.stpl")]
struct RegisterWithEmailSubjectTemplate;

#[derive(sailfish::Template)]
#[template(path = "emails/register_with_email/text.stpl")]
struct RegisterWithEmailTextTemplate {
    pub url: String,
}

#[derive(sailfish::Template)]
#[template(path = "emails/register_with_email/html.stpl")]
struct RegisterWithEmailHtmlTemplate {
    pub url: String,
}

pub(crate) async fn register_with_email_email<G: CoreConfig>(
    global: &Arc<G>,
    to_address: String,
    url: String,
) -> Result<pb::scufflecloud::email::v1::Email, sailfish::RenderError> {
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
#[template(path = "emails/magic_link/subject.stpl")]
struct MagicLinkSubjectTemplate;

#[derive(sailfish::Template)]
#[template(path = "emails/magic_link/text.stpl")]
struct MagicLinkTextTemplate {
    pub url: String,
}

#[derive(sailfish::Template)]
#[template(path = "emails/magic_link/html.stpl")]
struct MagicLinkHtmlTemplate {
    pub url: String,
}

pub(crate) async fn magic_link_email<G: CoreConfig>(
    global: &Arc<G>,
    to_address: String,
    url: String,
) -> Result<pb::scufflecloud::email::v1::Email, sailfish::RenderError> {
    let subject = MagicLinkSubjectTemplate.render_once()?;
    let text = MagicLinkTextTemplate { url: url.to_string() }.render_once()?;
    let html = MagicLinkHtmlTemplate { url }.render_once()?;

    Ok(pb::scufflecloud::email::v1::Email {
        to_address,
        source_address: global.email_from_address().to_string(),
        subject,
        text,
        html,
    })
}

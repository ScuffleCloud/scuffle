use std::path::Path;

use sailfish::TemplateOnce;
use scufflecloud_core_emails::GeoInfo;

#[derive(sailfish::Template)]
#[template(path = "preview/index.stpl")]
struct IndexTemplate {
    email_names: Vec<String>,
}

#[derive(sailfish::Template)]
#[template(path = "preview/email_index.stpl")]
struct EmailIndexTemplate {
    name: String,
    from_address: String,
    to_address: String,
    subject: String,
    text: String,
}

impl EmailIndexTemplate {
    fn from_email(name: String, email: scufflecloud_core_emails::Email) -> Self {
        Self {
            name,
            from_address: email.from_address,
            to_address: email.to_address,
            subject: email.subject,
            text: email.text,
        }
    }
}

fn save_email(output_path: &Path, name: &str, email: scufflecloud_core_emails::Email) {
    std::fs::create_dir_all(output_path.join(name)).expect("failed to create dir");
    std::fs::write(output_path.join(name).join("html.html"), email.html.as_bytes()).expect("failed to write");
    let email_index = EmailIndexTemplate::from_email(name.to_string(), email)
        .render_once()
        .expect("failed to render");
    std::fs::write(output_path.join(name).join("index.html"), email_index.as_bytes()).expect("failed to write");
}

fn main() {
    let args = std::env::args_os().nth(1).expect("missing output path");
    let output_path = std::path::PathBuf::from(args);
    std::fs::create_dir_all(&output_path).expect("failed to create output dir");

    let from_address = "no-reply@scuffle.cloud".to_string();
    let to_address = "test@example.com".to_string();
    let dashboard_origin = url::Url::parse("https://dashboard.scuffle.cloud").unwrap();
    let timeout = std::time::Duration::from_secs(15 * 60);

    let add_new_email = scufflecloud_core_emails::add_new_email_email(
        from_address.clone(),
        to_address.clone(),
        &dashboard_origin,
        "code".to_string(),
        timeout,
    )
    .expect("failed to render");
    save_email(&output_path, "add_new_email", add_new_email);

    let magic_link = scufflecloud_core_emails::magic_link_email(
        from_address.clone(),
        to_address.clone(),
        &dashboard_origin,
        "code".to_string(),
        timeout,
    )
    .expect("failed to render");
    save_email(&output_path, "magic_link", magic_link);

    let new_device = scufflecloud_core_emails::new_device_email(
        from_address.clone(),
        to_address.clone(),
        &dashboard_origin,
        "198.200.12.0".parse().unwrap(),
        GeoInfo {
            city: Some("Scuffle City".to_string()),
            country: Some("Scuffland".to_string()),
        },
    )
    .expect("failed to render");
    save_email(&output_path, "new_device", new_device);

    let register_with_email = scufflecloud_core_emails::register_with_email_email(
        from_address.clone(),
        to_address.clone(),
        &dashboard_origin,
        "code".to_string(),
        timeout,
    )
    .expect("failed to render");
    save_email(&output_path, "register_with_email", register_with_email);

    let index = IndexTemplate {
        email_names: vec![
            "add_new_email".to_string(),
            "magic_link".to_string(),
            "new_device".to_string(),
            "register_with_email".to_string(),
        ],
    };
    let index = index.render_once().expect("failed to render");
    std::fs::write(output_path.join("index.html"), index.as_bytes()).expect("failed to write index");
}

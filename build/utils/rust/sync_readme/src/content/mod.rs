use crate::config::Package;

mod badge;
mod rustdoc;
mod title;

pub struct Content {
    pub title: String,
    pub rustdoc: String,
    pub badge: String,
}

pub fn create(package: &Package) -> anyhow::Result<Content> {
    Ok(Content {
        title: title::create(package),
        rustdoc: rustdoc::create(package)?,
        badge: badge::create(package),
    })
}

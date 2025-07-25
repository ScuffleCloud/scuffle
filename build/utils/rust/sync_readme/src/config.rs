use std::collections::BTreeMap;

use camino::Utf8PathBuf;

#[derive(serde_derive::Deserialize, Debug, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct Metadata {
    #[serde(alias = "rustdoc-mappings")]
    pub rustdoc_mappings: BTreeMap<String, String>,
    #[serde(alias = "rustdoc-html-root-url")]
    pub rustdoc_html_root_url: Option<String>,
    #[serde(alias = "badge-style")]
    pub badge_style: String,
    pub badges: Badges,
    #[serde(alias = "custom-badges")]
    pub custom_badges: Vec<CustomBadge>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            badge_style: "flat-square".to_string(),
            rustdoc_html_root_url: Default::default(),
            badges: Default::default(),
            rustdoc_mappings: Default::default(),
            custom_badges: Default::default(),
        }
    }
}

#[derive(serde_derive::Deserialize, Debug, Default, Clone)]
#[serde(deny_unknown_fields)]
pub struct CustomBadge {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub link: Option<String>,
}

#[derive(serde_derive::Deserialize, Debug, Default, Clone)]
#[serde(default, deny_unknown_fields)]
pub struct Badges {
    #[serde(alias = "docs-rs")]
    pub docs_rs: bool,
    pub license: bool,
    #[serde(alias = "crates-io")]
    pub crates_io: CratesIo,
    pub codecov: Codecov,
}

#[derive(serde_derive::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum CratesIo {
    Simple(bool),
    Complex {
        #[serde(default)]
        release: bool,
        #[serde(default)]
        size: bool,
        #[serde(default)]
        downloads: bool,
    },
}

impl CratesIo {
    pub fn release(&self) -> bool {
        match self {
            Self::Simple(t) => *t,
            Self::Complex { release, .. } => *release,
        }
    }

    pub fn size(&self) -> bool {
        match self {
            Self::Simple(t) => *t,
            Self::Complex { size, .. } => *size,
        }
    }

    pub fn downloads(&self) -> bool {
        match self {
            Self::Simple(t) => *t,
            Self::Complex { downloads, .. } => *downloads,
        }
    }
}

impl Default for CratesIo {
    fn default() -> Self {
        Self::Simple(false)
    }
}

#[derive(serde_derive::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Codecov {
    Simple(bool),
    Complex {
        component: String,
    },
}

impl Default for Codecov {
    fn default() -> Self {
        Self::Simple(false)
    }
}

pub struct Package {
    pub name: String,
    pub version: String,
    pub license: Option<String>,
    pub metadata: Metadata,
    pub rustdoc_json: Utf8PathBuf,
}

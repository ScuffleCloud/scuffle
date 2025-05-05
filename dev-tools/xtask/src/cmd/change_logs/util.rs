use std::path::{Path, PathBuf};

use anyhow::Context;
use serde::Deserialize as _;
use serde::de::IntoDeserializer;
use serde_derive::Deserialize;

#[derive(Debug, Clone)]
pub struct Fragment {
    pub path: PathBuf,
    pub pr_number: u64,
    pub toml: toml_edit::DocumentMut,
    pub changed: bool,
    pub deleted: bool,
}

impl Fragment {
    pub fn new(pr_number: u64, path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path).context("change log not found")?;
        Ok(Fragment {
            pr_number,
            path: path.to_path_buf(),
            toml: content
                .parse::<toml_edit::DocumentMut>()
                .context("change log is not valid toml")?,
            changed: false,
            deleted: false,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackageChangeLog {
    #[serde(skip, default)]
    pub pr_number: u64,
    #[serde(alias = "cat")]
    pub category: String,
    #[serde(alias = "desc")]
    pub description: String,
    #[serde(default)]
    #[serde(alias = "author")]
    pub authors: Vec<String>,
    #[serde(default)]
    #[serde(alias = "break", alias = "major")]
    pub breaking: bool,
}

impl Fragment {
    pub fn remove_package(&mut self, package: &str) -> anyhow::Result<Vec<PackageChangeLog>> {
        let Some(items) = self.toml.remove(package) else {
            return Ok(Vec::new());
        };

        self.changed = true;

        package_to_logs(self.pr_number, items)
    }

    pub fn packages(&self) -> impl IntoIterator<Item = (&str, &toml_edit::Item)> {
        self.toml.as_table()
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        if !self.changed {
            return Ok(());
        }

        if self.toml.is_empty() {
            std::fs::remove_file(&self.path).context("remove")?;
            self.deleted = true;
        } else {
            std::fs::write(&self.path, self.toml.to_string()).context("write")?;
        }

        self.changed = false;

        Ok(())
    }
}

pub fn package_to_logs(pr_number: u64, items: toml_edit::Item) -> anyhow::Result<Vec<PackageChangeLog>> {
    let value = items.into_value().expect("items must be a value").into_deserializer();
    let mut logs = Vec::<PackageChangeLog>::deserialize(value).context("deserialize")?;

    logs.iter_mut().for_each(|log| {
        log.pr_number = pr_number;
    });

    Ok(logs)
}

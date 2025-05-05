use std::collections::{HashMap, HashSet};

use anyhow::Context;
use cargo_metadata::DependencyKind;
use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};

use crate::cmd::IGNORED_PACKAGES;

#[derive(Debug, Clone, clap::Parser)]
pub struct WorkspaceDeps {
    #[clap(long, short, value_delimiter = ',')]
    #[clap(alias = "package")]
    /// Packages to test
    packages: Vec<String>,
    #[clap(long, short, value_delimiter = ',')]
    #[clap(alias = "exclude-package")]
    /// Packages to exclude from testing
    exclude_packages: Vec<String>,
}

// the path that would need to be added to start to get to end
fn relative_path(start: &Utf8Path, end: &Utf8Path) -> Utf8PathBuf {
    // Break down the paths into components
    let start_components: Vec<&str> = start.components().map(|c| c.as_str()).collect();
    let end_components: Vec<&str> = end.components().map(|c| c.as_str()).collect();

    // Find the common prefix length
    let mut i = 0;
    while i < start_components.len() && i < end_components.len() && start_components[i] == end_components[i] {
        i += 1;
    }

    // Start building the relative path
    let mut result = Utf8PathBuf::new();

    // For each remaining component in `start`, add ".."
    for _ in i..start_components.len() {
        result.push("..");
    }

    // Append the remaining components from `end`
    for comp in &end_components[i..] {
        result.push(comp);
    }

    // If the resulting path is empty, use "." to represent the current directory
    if result.as_str().is_empty() {
        result.push(".");
    }

    result
}

impl WorkspaceDeps {
    pub fn run(self) -> anyhow::Result<()> {
        let start = std::time::Instant::now();

        let metadata = crate::utils::metadata()?;

        let workspace_package_ids = metadata.workspace_members.iter().cloned().collect::<HashSet<_>>();

        let workspace_packages = metadata
            .packages
            .iter()
            .filter(|p| workspace_package_ids.contains(&p.id))
            .map(|p| (&p.id, p))
            .collect::<HashMap<_, _>>();

        let path_to_package = workspace_packages
            .values()
            .map(|p| (p.manifest_path.parent().unwrap(), &p.id))
            .collect::<HashMap<_, _>>();

        for package in metadata.packages.iter().filter(|p| workspace_package_ids.contains(&p.id)) {
            if (IGNORED_PACKAGES.contains(&package.name.as_str()) || self.exclude_packages.contains(&package.name))
                && (self.packages.is_empty() || !self.packages.contains(&package.name))
            {
                continue;
            }

            let toml = std::fs::read_to_string(&package.manifest_path)
                .with_context(|| format!("failed to read manifest for {}", package.name))?;
            let mut doc = toml
                .parse::<toml_edit::DocumentMut>()
                .with_context(|| format!("failed to parse manifest for {}", package.name))?;
            let mut changes = false;

            for dependency in package.dependencies.iter() {
                if dependency.kind != DependencyKind::Development {
                    continue;
                }

                let Some(path) = dependency.path.as_deref() else {
                    continue;
                };

                if path_to_package.get(path).and_then(|id| workspace_packages.get(id)).is_none() {
                    continue;
                }

                let dep = doc["dev-dependencies"][&dependency.name]
                    .as_table_like_mut()
                    .expect("expected table");

                dep.insert(
                    "path",
                    toml_edit::value(relative_path(package.manifest_path.parent().unwrap(), path).to_string()),
                );
                if let Some(rename) = dependency.rename.clone() {
                    dep.insert("rename", toml_edit::value(rename));
                }

                if !dependency.features.is_empty() {
                    let mut array = toml_edit::Array::new();
                    for feature in dependency.features.iter().cloned() {
                        array.push(feature);
                    }
                    dep.insert("features", toml_edit::value(array));
                }
                if dependency.optional {
                    dep.insert("optional", toml_edit::value(true));
                }

                dep.remove("workspace");

                println!("Replaced path in {} for '{}' to '{}'", package.name, dependency.name, path);
                changes = true;
            }

            if changes {
                std::fs::write(&package.manifest_path, doc.to_string())
                    .with_context(|| format!("failed to write manifest for {}", package.name))?;
                println!("Replaced paths in {} for {}", package.name, package.manifest_path);
            }
        }

        println!("Done in {:?}", start.elapsed());

        Ok(())
    }
}

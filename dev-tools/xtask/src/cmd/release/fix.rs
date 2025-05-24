use std::collections::{BTreeMap, BTreeSet, HashSet};

use anyhow::Context;
use cargo_metadata::{DependencyKind, semver};

use super::utils::{
    Fragment, LicenseKind, Package, PackageChangeLog, PackageError, PackageErrorMissing, PackageFile, concurrently,
    dep_kind_to_name, git_workdir_clean, relative_to,
};
use crate::utils;

#[derive(Debug, Clone, clap::Parser)]
pub struct Fix {
    /// The pull request number
    #[arg(long, short = 'n')]
    pr_number: u64,
    /// The base branch to compare against to determine
    /// if something has changed.
    #[arg(long, default_value = "origin/main")]
    base_branch: String,
    /// Check everything, even if there are no changes
    /// from this branch to the base branch.
    #[arg(long)]
    all: bool,
    /// Packages to include in the check
    /// by default all packages are included
    #[arg(long = "package", short = 'p')]
    packages: Vec<String>,
    /// Allow to run when there are uncomitted changes.
    #[arg(long)]
    allow_dirty: bool,
    // Concurrency to run at
    #[arg(long, default_value_t = num_cpus::get())]
    concurrency: usize,
    /// Author to use for the changelog entries
    #[arg(long = "author")]
    authors: Vec<String>,
}

impl Fix {
    pub fn run(mut self) -> anyhow::Result<()> {
        if !self.allow_dirty {
            git_workdir_clean()?;
        }

        self.authors.iter_mut().for_each(|author| {
            if !author.starts_with("@") {
                *author = format!("@{author}");
            }
        });

        let metadata = utils::metadata().context("metadata")?;

        let members = metadata.workspace_members.iter().cloned().collect::<HashSet<_>>();
        let packages = metadata
            .packages
            .iter()
            .filter(|p| members.contains(&p.id))
            .map(|p| Ok((p.name.clone(), Package::new(p.clone())?)))
            .collect::<anyhow::Result<BTreeMap<_, _>>>()?;

        let accepted_groups = packages
            .values()
            .filter(|p| self.packages.contains(&p.name) || self.packages.is_empty())
            .map(|p| p.group())
            .collect::<BTreeSet<_>>();

        let groups = packages.values().fold(BTreeMap::<_, Vec<_>>::new(), |mut groups, package| {
            if !accepted_groups.contains(package.group()) {
                return groups;
            }

            let entry = groups.entry(package.group()).or_default();
            if package.name == package.group() {
                entry.insert(0, package);
            } else {
                entry.push(package);
            }

            groups
        });

        concurrently::<_, _, anyhow::Result<()>>(self.concurrency, packages.values().map(|p| || p.fetch_published()))?;
        concurrently::<_, _, anyhow::Result<()>>(
            self.concurrency,
            groups.values().flatten().map(|p| {
                || {
                    p.check(
                        &packages,
                        &metadata.workspace_root,
                        if self.all { None } else { Some(&self.base_branch) },
                    )
                }
            }),
        )?;

        let dependants = packages.values().fold(BTreeMap::<_, Vec<_>>::new(), |mut deps, package| {
            for dep in package.dependencies.iter() {
                let Some(pkg) = packages.get(&dep.name) else {
                    continue;
                };

                if accepted_groups.contains(pkg.group()) {
                    continue;
                }

                deps.entry(dep.name.as_str()).or_default().push((pkg, dep));
            }

            deps
        });

        let mut fragment = Fragment::new(self.pr_number, &metadata.workspace_root)?;

        // we do this twice because we need to account
        // for dependencies that depend on eachother requiring
        // version bumps.
        for idx in 0..2 {
            groups.iter().for_each(|(name, group)| {
                tracing::debug!("checking group {name} ({idx})");
                let first = &group.first().unwrap().version;

                // if any of the packages in the group has a new version or has any errors
                if group
                    .iter()
                    .any(|p| &p.version != first || p.has_errors() || p.next_version().is_some())
                {
                    group.iter().for_each(|p| {
                        p.report_change();
                    });

                    if let Some(next_version) = group.iter().filter_map(|p| p.next_version()).max() {
                        group.iter().for_each(|p| {
                            p.set_next_version(next_version.clone());
                            dependants.get(p.name.as_str()).into_iter().flatten().for_each(|(pkg, dep)| {
                                if matches!(dep.kind, DependencyKind::Normal | DependencyKind::Build)
                                    && !dep.req.matches(&next_version)
                                {
                                    pkg.report_change();
                                }
                            });
                        });
                    }
                }
            });
        }

        for package in groups.values().flatten() {
            let _span = tracing::info_span!("edit", package = package.name).entered();
            let cargo_toml_raw = std::fs::read_to_string(&package.manifest_path).context("read cargo toml")?;
            let mut cargo_toml = cargo_toml_raw.parse::<toml_edit::DocumentMut>().context("parse toml")?;
            let mut changelogs = Vec::new();

            if let Some(min_versions_output) = package.min_versions_output() {
                tracing::error!("min version error cannot be automatically fixed.");
                eprintln!("{min_versions_output}");
            }

            for error in package.errors() {
                match error {
                    PackageError::DevDependencyHasVersion { name, target } => {
                        let deps = if let Some(target) = target {
                            &mut cargo_toml["target"][target.to_string()]
                        } else {
                            cargo_toml.as_item_mut()
                        };

                        deps["dev-dependencies"][&name]
                            .as_table_like_mut()
                            .expect("table like")
                            .remove("version");
                        changelogs.push(PackageChangeLog::new(
                            "chore",
                            format!("removed version from dev dependency `{name}`"),
                        ));
                    }
                    PackageError::DependencyMissingVersion { .. } => {}
                    PackageError::DependencyGroupedVersion { .. } => {}
                    PackageError::DependencyNotPublishable { .. } => {}
                    PackageError::Missing(PackageErrorMissing::Author) => {
                        cargo_toml["package"]["authors"] =
                            toml_edit::Array::from_iter(["Scuffle <opensource@scuffle.cloud>"]).into();
                        changelogs.push(PackageChangeLog::new("docs", "added authors to Cargo.toml"));
                    }
                    PackageError::Missing(PackageErrorMissing::Description) => {
                        cargo_toml["package"]["description"] = format!("{} is a work-in-progress!", package.name).into();
                        changelogs.push(PackageChangeLog::new("docs", "added description to Cargo.toml"));
                    }
                    PackageError::Missing(PackageErrorMissing::Documentation) => {
                        cargo_toml["package"]["documentation"] = format!("https://docs.rs/{}", package.name).into();
                        changelogs.push(PackageChangeLog::new("docs", "added documentation to Cargo.toml"));
                    }
                    PackageError::Missing(PackageErrorMissing::License) => {
                        cargo_toml["package"]["license"] = "MIT OR Apache-2.0".into();
                        for file in [
                            PackageFile::License(LicenseKind::Mit),
                            PackageFile::License(LicenseKind::Apache2),
                        ] {
                            let path = package.manifest_path.with_file_name(file.to_string());
                            let file_path = metadata.workspace_root.join(file.to_string());
                            let relative_path = relative_to(&file_path, path.parent().unwrap());
                            tracing::info!("creating {path}");
                            std::os::unix::fs::symlink(relative_path, path).context("license symlink")?;
                        }
                        changelogs.push(PackageChangeLog::new("docs", "added license to Cargo.toml"));
                    }
                    PackageError::Missing(PackageErrorMissing::ChangelogEntry) => {}
                    PackageError::Missing(PackageErrorMissing::Readme) => {
                        cargo_toml["package"]["readme"] = "README.md".into();
                        changelogs.push(PackageChangeLog::new("docs", "added readme to Cargo.toml"));
                    }
                    PackageError::Missing(PackageErrorMissing::Repopository) => {
                        cargo_toml["package"]["repository"] = "https://github.com/scufflecloud/scuffle".into();
                        changelogs.push(PackageChangeLog::new("docs", "added repository to Cargo.toml"));
                    }
                    PackageError::MissingFile(file @ PackageFile::Changelog) => {
                        const CHANGELOG_TEMPLATE: &str = include_str!("./changelog_template.md");
                        let path = package.manifest_path.with_file_name(file.to_string());
                        tracing::info!("creating {}", relative_to(&path, &metadata.workspace_root));
                        std::fs::write(path, CHANGELOG_TEMPLATE).context("changelog write")?;
                        changelogs.push(PackageChangeLog::new("docs", format!("created file {file}")));
                    }
                    PackageError::MissingFile(file @ PackageFile::Readme) => {
                        const README_TEMPLATE: &str = include_str!("./readme_template.md");
                        let path = package.manifest_path.with_file_name(file.to_string());
                        tracing::info!("creating {}", relative_to(&path, &metadata.workspace_root));
                        std::fs::write(path, README_TEMPLATE).context("readme write")?;
                        changelogs.push(PackageChangeLog::new("docs", format!("created file {file}")));
                    }
                    PackageError::MissingFile(file @ PackageFile::License(_)) => {
                        let path = package.manifest_path.with_file_name(file.to_string());
                        let file_path = metadata.workspace_root.join(file.to_string());
                        let relative_path = relative_to(&file_path, path.parent().unwrap());
                        tracing::info!("creating {}", relative_to(&path, &metadata.workspace_root));
                        std::os::unix::fs::symlink(relative_path, path).context("license symlink")?;
                        changelogs.push(PackageChangeLog::new("docs", format!("created file {file}")));
                    }
                }
            }

            for dep in &package.dependencies {
                if !matches!(dep.kind, DependencyKind::Normal | DependencyKind::Build) {
                    continue;
                }

                let Some(dep_pkg) = packages.get(&dep.name) else {
                    continue;
                };

                let next_dep_version = dep_pkg.next_version().unwrap_or_else(|| dep_pkg.version.clone());
                let req = if dep_pkg.group() == package.group() {
                    semver::VersionReq {
                        comparators: vec![semver::Comparator {
                            major: next_dep_version.major,
                            minor: Some(next_dep_version.minor),
                            patch: Some(next_dep_version.patch),
                            pre: next_dep_version.pre.clone(),
                            op: semver::Op::Exact,
                        }],
                    }
                } else if !dep.req.matches(&next_dep_version) {
                    semver::VersionReq {
                        comparators: vec![semver::Comparator {
                            major: next_dep_version.major,
                            minor: Some(next_dep_version.minor),
                            patch: Some(next_dep_version.patch),
                            pre: next_dep_version.pre.clone(),
                            op: semver::Op::Caret,
                        }],
                    }
                } else {
                    continue;
                };

                if req == dep.req {
                    continue;
                }

                let table = if let Some(target) = &dep.target {
                    &mut cargo_toml["target"][target.to_string()][dep_kind_to_name(&dep.kind)]
                } else {
                    &mut cargo_toml[dep_kind_to_name(&dep.kind)]
                };

                changelogs.push(
                    PackageChangeLog::new("chore", format!("bumped `{}` to `{req}`", dep.name))
                        .with_version_bump(&dep.name, next_dep_version),
                );
                table[&dep.name]["version"] = req.to_string().into();
            }

            if let Some(version) = package.next_version() {
                cargo_toml["package"]["version"] = version.to_string().into();
            }

            if package.changelog_path().is_some() {
                for mut changelog in changelogs {
                    changelog.authors = self.authors.clone();
                    fragment.add_log(&package.name, &changelog);
                }
            }

            let cargo_toml_updated = cargo_toml.to_string();
            if cargo_toml_updated != cargo_toml_raw {
                tracing::info!(
                    "{}",
                    fmtools::fmt(|f| {
                        f.write_str("updating ")?;
                        f.write_str(relative_to(&package.manifest_path, &metadata.workspace_root).as_str())?;
                        if let Some(version) = package.next_version() {
                            write!(f, " bumping to {version}")?;
                        }
                        Ok(())
                    })
                );
                std::fs::write(&package.manifest_path, cargo_toml.to_string()).context("manifest write")?;
            }
        }

        if fragment.changed() {
            tracing::info!(
                "{} {}",
                if fragment.deleted() { "creating" } else { "updating" },
                relative_to(fragment.path(), &metadata.workspace_root),
            );
            fragment.save().context("save changelog")?;
        }

        tracing::info!("complete");

        Ok(())
    }
}

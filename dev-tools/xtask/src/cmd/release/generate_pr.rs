use std::collections::{BTreeMap, HashSet};
use std::fmt::Write;

use anyhow::Context;
use cargo_metadata::camino::Utf8Path;
use cargo_metadata::semver::Version;

use super::utils::{Fragment, PackageChangeLog};
use crate::cmd::release::utils::{Package, concurrently};

#[derive(Debug, Clone, clap::Parser)]
pub struct GeneratePr {
    // Concurrency to run at
    #[arg(long, default_value_t = num_cpus::get())]
    concurrency: usize,
    /// Do not bump the version in the changelog
    #[arg(long)]
    no_changelog_bump: bool,
}

struct ReleasePackage {
    package: Package,
    is_published: bool,
}

fn update_change_log(
    packages: &BTreeMap<String, ReleasePackage>,
    logs: &[PackageChangeLog],
    change_log_path_md: &Utf8Path,
    name: &str,
    version: &Version,
    previous_version: Option<&Version>,
    no_changelog_bump: bool,
) -> anyhow::Result<()> {
    let mut change_log = std::fs::read_to_string(change_log_path_md).context("failed to read CHANGELOG.md")?;

    // Find the # [Unreleased] section
    // So we can insert the new logs after it
    let mut seen_bumps = HashSet::new();
    let (mut breaking_changes, mut other_changes) = logs
        .iter()
        .filter(|log| {
            log.version_bump.as_ref().is_none_or(|(package, version)| {
                packages
                    .get(package)
                    .is_some_and(|p| &p.package.version == version && seen_bumps.insert((package, version)))
            })
        })
        .partition::<Vec<_>, _>(|log| log.breaking);
    breaking_changes.sort_by_key(|log| &log.category);
    other_changes.sort_by_key(|log| &log.category);

    fn make_logs(logs: &[&PackageChangeLog]) -> String {
        fmtools::fmt(|f| {
            let mut first = false;
            for log in logs {
                if !first {
                    f.write_char('\n')?;
                }
                first = false;

                let (tag, desc) = log.description.split_once('\n').unwrap_or((&log.description, ""));
                write!(
                    f,
                    "- {category}: {tag} ([#{pr_number}](https://github.com/scufflecloud/scuffle/pull/{pr_number}))",
                    category = log.category,
                    tag = tag.trim(),
                    pr_number = log.pr_number,
                )?;

                if !log.authors.is_empty() {
                    f.write_str(" (")?;
                    let mut first = true;
                    for author in &log.authors {
                        if !first {
                            f.write_str(", ")?;
                        }
                        first = false;
                        f.write_str(author)?;
                    }
                    f.write_char(')')?;
                }

                let desc = desc.trim();

                if !desc.is_empty() {
                    f.write_str("\n\n")?;
                    f.write_str(desc)?;
                    f.write_char('\n')?;
                }
            }

            Ok(())
        })
        .to_string()
    }

    let breaking_changes = make_logs(&breaking_changes);
    let other_changes = make_logs(&other_changes);

    let mut replaced = String::new();

    replaced.push_str("## [Unreleased]\n\n");

    if !no_changelog_bump {
        replaced.push_str(&format!(
            "## [{version}](https://github.com/ScuffleCloud/scuffle/releases/tag/{name}-v{version}) - {date}\n\n",
            date = chrono::Utc::now().date_naive().format("%Y-%m-%d")
        ));
        if let Some(previous_version) = &previous_version {
            replaced.push_str(&format!(
                "[View diff on diff.rs](https://diff.rs/{name}/{previous_version}/{name}/{version}/Cargo.toml)\n",
            ));
        }
    }

    if !breaking_changes.is_empty() {
        replaced.push_str("\n### ⚠️ Breaking changes\n\n");
        replaced.push_str(&breaking_changes);
        replaced.push('\n');
    }

    if !other_changes.is_empty() {
        replaced.push_str("\n### 🛠️ Non-breaking changes\n\n");
        replaced.push_str(&other_changes);
        replaced.push('\n');
    }

    change_log = change_log.replace("## [Unreleased]\n", &replaced);

    std::fs::write(change_log_path_md, change_log).context("failed to write CHANGELOG.md")?;

    Ok(())
}

fn generate_change_logs(
    package: &str,
    change_fragments: &mut BTreeMap<u64, Fragment>,
) -> anyhow::Result<Vec<PackageChangeLog>> {
    let mut logs = Vec::new();

    for fragment in change_fragments.values_mut() {
        logs.extend(fragment.remove_package(package).context("parse")?);
    }

    Ok(logs)
}

fn save_change_fragments(fragments: &mut BTreeMap<u64, Fragment>) -> anyhow::Result<()> {
    fragments
        .values_mut()
        .filter(|fragment| fragment.changed())
        .try_for_each(|fragment| fragment.save().context("save"))?;

    fragments.retain(|_, fragment| !fragment.deleted());

    Ok(())
}

impl GeneratePr {
    pub fn run(self) -> anyhow::Result<()> {
        let metadata = crate::utils::metadata()?;

        let workspace_package_ids = metadata.workspace_members.iter().cloned().collect::<HashSet<_>>();

        let packages = concurrently::<_, _, anyhow::Result<BTreeMap<_, _>>>(
            self.concurrency,
            metadata
                .packages
                .iter()
                .filter(|p| workspace_package_ids.contains(&p.id))
                .map(|package| {
                    move || {
                        let _span = tracing::info_span!("package", package = &package.name).entered();

                        let package = Package::new(package.clone())?;
                        package.fetch_published()?;

                        let previous_version = package.last_published_version();

                        let is_published = !(previous_version.as_ref().is_none_or(|p| p.vers != package.version)
                            && package.should_publish());

                        if is_published {
                            tracing::info!("already published");
                        } else {
                            tracing::info!("not published");
                        }

                        anyhow::Ok((package.name.clone(), ReleasePackage { package, is_published }))
                    }
                }),
        )?;

        if packages.is_empty() {
            tracing::info!("no packages need to be published");
            return Ok(());
        }

        let mut change_fragments = std::fs::read_dir(metadata.workspace_root.join("changes.d"))?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    let file_name = entry_path.file_name()?.to_str()?;
                    file_name.strip_prefix("pr-")?.strip_suffix(".toml")?.parse().ok()
                } else {
                    None
                }
            })
            .try_fold(BTreeMap::new(), |mut fragments, pr_number| {
                let fragment = Fragment::new(pr_number, &metadata.workspace_root)?;

                fragments.insert(pr_number, fragment);

                anyhow::Ok(fragments)
            })?;

        let mut pr_body = String::from("## 🤖 New release\n\n");

        for (name, release) in &packages {
            if release.is_published {
                continue;
            }

            if let Some(change_log_path_md) = release.package.changelog_path() {
                let change_logs = generate_change_logs(name.as_str(), &mut change_fragments).context("generate")?;
                update_change_log(
                    &packages,
                    &change_logs,
                    &change_log_path_md,
                    name,
                    &release.package.version,
                    release.package.last_published_version().map(|v| v.vers).as_ref(),
                    self.no_changelog_bump,
                )
                .context("update")?;
                save_change_fragments(&mut change_fragments).context("save")?;
                tracing::info!("updated change logs",);
            }

            pr_body.push_str(&format!("* `{name}` -> {}\n", release.package.version));
        }

        println!("{pr_body}");

        Ok(())
    }
}

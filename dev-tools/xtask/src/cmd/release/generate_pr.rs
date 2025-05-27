use std::collections::{BTreeMap, HashSet};
use std::fmt::Write;
use std::process::{Command, Stdio};

use anyhow::Context;
use cargo_metadata::camino::Utf8Path;
use cargo_metadata::semver::Version;

use super::utils::{Fragment, PackageChangeLog, XTaskWorkspaceMetadata, git_workdir_clean};
use crate::cmd::release::utils::{Package, concurrently};

#[derive(Debug, Clone, clap::Parser)]
pub struct GeneratePr {
    // Concurrency to run at
    #[arg(long, default_value_t = num_cpus::get())]
    concurrency: usize,
    /// Do not bump the version in the changelog
    #[arg(long)]
    no_changelog_bump: bool,
    /// Create & open a GitHub PR
    #[arg(long)]
    create_pr: bool,
    /// Allow to run when there are uncomitted changes.
    #[arg(long)]
    allow_dirty: bool,
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
            let mut first = true;
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

#[derive(serde_derive::Deserialize)]
struct PrView {
    number: u32,
    state: String,
    labels: Vec<PrLabel>,
}

#[derive(serde_derive::Deserialize)]
struct PrLabel {
    name: String,
}

struct RunDrop<F: FnOnce() -> ()> {
    f: Option<F>,
}

impl<F: FnOnce() -> ()> RunDrop<F> {
    fn new(f: F) -> Self {
        Self { f: Some(f) }
    }
}

impl<F: FnOnce() -> ()> Drop for RunDrop<F> {
    fn drop(&mut self) {
        if let Some(f) = self.f.take() {
            (f)()
        }
    }
}

impl GeneratePr {
    pub fn run(self) -> anyhow::Result<()> {
        if !self.allow_dirty {
            git_workdir_clean()?;
        }

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
                tracing::debug!(package = %name, "skipping package because already released");
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
                tracing::info!(package = %name, "updated change logs");
            }

            pr_body.push_str(&format!("* `{name}` -> {}\n", release.package.version));
        }

        if !self.create_pr {
            println!("{pr_body}");
            return Ok(());
        }

        let current_branch = Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .output()
            .context("get current branch name")?;

        if !current_branch.status.success() {
            anyhow::bail!(
                "failed to get current branch: {}",
                String::from_utf8_lossy(&current_branch.stderr)
            )
        }

        let xtask_metadata = XTaskWorkspaceMetadata::from_metadata(&metadata).context("parse workspace metadata")?;

        let mut branch_name = String::from_utf8_lossy(&current_branch.stdout);
        let _checkout_previous = if !branch_name.eq_ignore_ascii_case(&xtask_metadata.pr_branch) {
            let cmd = Command::new("git")
                .arg("checkout")
                .arg("-B")
                .arg(&xtask_metadata.pr_branch)
                .output()
                .context("checkout new branch")?;
            if !cmd.status.success() {
                anyhow::bail!("failed to checkout new branch: {}", String::from_utf8_lossy(&cmd.stderr));
            }

            Some(RunDrop::new(|| match Command::new("git").arg("checkout").arg("-").output() {
                Ok(output) if output.status.success() => {}
                Ok(output) => {
                    tracing::error!(
                        "failed to checkout previous branch: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                }
                Err(err) => {
                    tracing::error!("failed to checkout previous branch: {err}");
                }
            }))
        } else {
            branch_name = "main".into();
            None
        };

        let git_add_commit = Command::new("git")
            .arg("commit")
            .arg("-am")
            .arg("update changelog entries")
            .output()
            .context("git add")?;

        if !git_add_commit.status.success() {
            anyhow::bail!("failed to git commit: {}", String::from_utf8_lossy(&git_add_commit.stderr));
        }

        let git_push = Command::new("git")
            .arg("push")
            .arg("origin")
            .arg("--force")
            .output()
            .context("git push")?;

        if !git_push.status.success() {
            anyhow::bail!("failed to git push: {}", String::from_utf8_lossy(&git_push.stderr));
        }

        let pr = Command::new("gh")
            .arg("pr")
            .arg("view")
            .arg(&xtask_metadata.pr_branch)
            .arg("--json")
            .arg("number,state,labels")
            .output()
            .context("get-current-pr")?;

        let current_pr = if pr.status.success() {
            let out = serde_json::from_slice::<PrView>(&pr.stdout).context("gh pr output")?;
            out.state.eq_ignore_ascii_case("open").then_some(out)
        } else {
            let stderr = String::from_utf8_lossy(&pr.stdout);
            if !stderr.contains("no pull requests found for branch") {
                anyhow::bail!("failed to get previous pr: {stderr}")
            }

            None
        };

        let mut command = Command::new("gh");

        if let Some(current_pr) = current_pr.as_ref() {
            command.arg("pr").arg("edit").arg(current_pr.number.to_string());
        } else {
            command.arg("pr").arg("create").arg("--head").arg(&xtask_metadata.pr_branch);
        };

        command.arg("--base").arg(branch_name.as_ref());
        command.arg("--title").arg(&xtask_metadata.pr_title);
        command.arg("--body").arg(pr_body);

        if !xtask_metadata.pr_labels.is_empty() {
            if let Some(current_pr) = current_pr.as_ref() {
                for label in xtask_metadata
                    .pr_labels
                    .iter()
                    .filter(|label| !current_pr.labels.iter().any(|l| l.name.eq_ignore_ascii_case(label)))
                {
                    command.arg("--add-label").arg(label);
                }
            } else {
                for label in &xtask_metadata.pr_labels {
                    command.arg("--label").arg(label);
                }
            }
        }

        let make_update = command.spawn().context("create pr")?.wait().context("create pr wait")?;
        if !make_update.success() {
            anyhow::bail!("failed to update/create pr");
        }

        Ok(())
    }
}

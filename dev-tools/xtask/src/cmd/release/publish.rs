use std::collections::HashSet;

use anyhow::Context;

use super::utils::{GitReleaseArtifact, Package};
use crate::cmd::release::utils::WorkspaceReleaseMetadata;
use crate::utils::{self, Command, cargo_cmd, concurrently, git_workdir_clean};

#[derive(Debug, Clone, clap::Parser)]
pub struct Publish {
    /// Packages to include in the check
    /// by default all packages are included
    #[arg(long = "package", short = 'p')]
    packages: Vec<String>,
    /// Allow the command to execute even if there are uncomitted changes in the workspace
    #[arg(long)]
    allow_dirty: bool,
    /// Concurrency to run at. By default, this is the total number of cpus on the host.
    #[arg(long, default_value_t = num_cpus::get())]
    concurrency: usize,
    /// Do not release anything.
    #[arg(long)]
    dry_run: bool,
    /// Token to use when uploading to crates.io
    #[arg(long)]
    crates_io_token: Option<String>,
}

impl Publish {
    pub fn run(self) -> anyhow::Result<()> {
        if !self.allow_dirty {
            git_workdir_clean()?;
        }

        // get current commit
        let current_commit = Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .output()
            .context("git rev-parse HEAD")?;

        if !current_commit.status.success() {
            anyhow::bail!(
                "failed to get current commit sha: {}",
                String::from_utf8_lossy(&current_commit.stderr)
            );
        }

        let current_commit = String::from_utf8_lossy(&current_commit.stdout);
        let current_commit = current_commit.trim();

        let metadata = utils::metadata().context("metadata")?;

        let workspace_relese_metadata =
            WorkspaceReleaseMetadata::from_metadadata(&metadata).context("workspace metadata")?;

        let packages = {
            let members = metadata.workspace_members.iter().collect::<HashSet<_>>();
            metadata
                .packages
                .iter()
                .filter(|p| members.contains(&p.id))
                .filter(|p| self.packages.contains(&p.name) || self.packages.is_empty())
                .map(|p| Package::new(&workspace_relese_metadata, p.clone()))
                .collect::<anyhow::Result<Vec<_>>>()?
        };

        concurrently::<_, _, anyhow::Result<()>>(self.concurrency, packages.iter(), |p| p.fetch_published())?;

        let mut crates_io_publish = Vec::new();
        let mut git_releases = Vec::new();

        for package in &packages {
            if !package.slated_for_release() {
                tracing::info!("{} is not slated for release", package.name);
                continue;
            }

            if package.last_published_version().is_some_and(|p| p.vers == package.version) {
                tracing::info!("{}@{} has already been released on crates.io", package.name, package.version);
            } else if package.should_publish() {
                tracing::info!("{}@{} has not yet been published", package.name, package.version);
                crates_io_publish.push(&package.name);
            }

            if let Some(git) = package.git_release().context("git release")? {
                git_releases.push((package, git));
            }
        }

        if !crates_io_publish.is_empty() {
            let mut release_cmd = cargo_cmd();

            release_cmd
                .env("RUSTC_BOOTSTRAP", "1")
                .arg("-Zunstable-options")
                .arg("-Zpackage-workspace")
                .arg("publish")
                .arg("--no-verify");

            if self.dry_run {
                release_cmd.arg("--dry-run");
            }

            for package in &crates_io_publish {
                release_cmd.arg("-p").arg(package.as_ref());
            }

            if let Some(token) = &self.crates_io_token {
                release_cmd.arg("--token").arg(token);
            }

            if !release_cmd.status().context("crates io release")?.success() {
                anyhow::bail!("failed to publish crates");
            }
        }

        for (package, release) in &git_releases {
            let gh_release_view = Command::new("gh")
                .arg("release")
                .arg("view")
                .arg(release.tag_name.trim())
                .arg("--json")
                .arg("url")
                .output()
                .context("gh release view")?;

            if gh_release_view.status.success() {
                tracing::info!("{} is already released", release.tag_name.trim());
                continue;
            }

            let mut gh_release_create = Command::new("gh");

            gh_release_create
                .arg("release")
                .arg("create")
                .arg(release.tag_name.trim())
                .arg("--target")
                .arg(current_commit)
                .arg("--title")
                .arg(release.name.trim())
                .arg("--notes")
                .arg(release.body.trim());

            for artifact in &release.artifacts {
                match artifact {
                    GitReleaseArtifact::File { path, name } => {
                        let artifact = package.manifest_path.parent().unwrap().join(path);
                        let name = name.as_deref().or_else(|| artifact.file_name());
                        gh_release_create.arg(if let Some(name) = name {
                            format!("{artifact}#{name}")
                        } else {
                            artifact.to_string()
                        });
                    }
                }
            }

            if !self.dry_run {
                if !gh_release_create.status().context("gh release create")?.success() {
                    anyhow::bail!("failed to create gh release");
                }
            } else {
                tracing::info!("skipping running: {gh_release_create}")
            }
        }

        tracing::info!("released packages");

        Ok(())
    }
}

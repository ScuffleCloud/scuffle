use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Write;
use std::io::Read;
use std::process::Stdio;

use anyhow::Context;
use cargo_metadata::camino::Utf8Path;
use cargo_metadata::{DependencyKind, semver};

use super::utils::{Fragment, Package, concurrently, git_workdir_clean};
use crate::cmd::release::utils::{LicenseKind, PackageError, PackageErrorMissing, PackageFile, WriteUndo};
use crate::utils::{self, cargo_cmd};

#[derive(Debug, Clone, clap::Parser)]
pub struct Check {
    /// The pull request number
    #[arg(long, short = 'n')]
    pr_number: Option<u64>,
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
    /// Output markdown to stdout (used for CI to generate PR comments / Summaries)
    #[arg(long, group = "fix-conflict")]
    stdout_markdown: bool,
    /// Return a non-zero exit status at the end if a check failed.
    #[arg(long)]
    exit_status: bool,
    // Concurrency to run at
    #[arg(long, default_value_t = num_cpus::get())]
    concurrency: usize,
}

impl Check {
    pub fn run(self) -> anyhow::Result<()> {
        if !self.allow_dirty {
            git_workdir_clean()?;
        }

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

        let fragment = if let Some(pr_number) = self.pr_number {
            Some(Fragment::new(
                pr_number,
                &metadata.workspace_root.join("changes.d").join(format!("pr-{pr_number}.toml")),
            )?)
        } else {
            None
        };

        let mut new_releases_markdown = Vec::new();
        let mut errors_markdown = Vec::new();
        let mut any_issue = false;

        for group in groups.values() {
            if let Some(next_version) = group.iter().filter_map(|p| p.next_version()).max() {
                group.iter().for_each(|p| p.set_next_version(next_version.clone()));
            }

            for package in group {
                if let Some(next_version) = package.next_version() {
                    let old_version = &package.version;
                    any_issue = true;
                    let semver_output = package.semver_output();
                    let extra = if let Some((breaking, _)) = &semver_output {
                        if *breaking {
                            " (⚠️ API breaking changes)"
                        } else {
                            " (✓ API compatible changes)"
                        }
                    } else {
                        ""
                    };
                    tracing::warn!("{} requires a version bump to {next_version}{extra}", package.name);
                    new_releases_markdown.push(
                        fmtools::fmt(|f| {
                            write!(f, "* `{}`: {old_version} -> {next_version}{extra}", package.name)?;
                            if let Some((true, logs)) = &semver_output {
                                let mut f = indent_write::fmt::IndentWriter::new("  ", f);
                                f.write_str("\n\n<details><summary>Semver Checks Output</summary>\n\n")?;
                                write!(f, "```\n{logs}\n```\n\n")?;
                                f.write_str("</details>\n")?;
                            }
                            Ok(())
                        })
                        .to_string(),
                    );
                }

                let mut errors = package.errors();
                if let Some(fragment) = &fragment {
                    if !fragment.has_package(&package.name)
                        && package.next_version().is_some()
                        && package.changelog_path().is_some()
                    {
                        tracing::warn!("changelog entry must be provided");
                        errors.insert(0, PackageError::Missing(PackageErrorMissing::ChangelogEntry));
                    }
                }

                let min_versions_output = package.min_versions_output();

                if !errors.is_empty() || min_versions_output.is_some() {
                    any_issue = true;
                    errors_markdown.push(format!("## {}\n", package.name));
                    for error in errors.iter() {
                        errors_markdown.push(format!("* {error}"))
                    }
                    if let Some(min_versions_output) = min_versions_output {
                        errors_markdown.push(format!("* min package versions issue\n\n<details><summary>Output</summary>\n\n```\n{min_versions_output}\n```\n\n</details>\n"))
                    }
                    errors_markdown.push("".into());
                }
            }
        }

        if self.stdout_markdown {
            let mut first = true;
            if !new_releases_markdown.is_empty() {
                println!("# ⚠️ Version Bumps Needed\n");
                for line in new_releases_markdown {
                    println!("{line}");
                }
                first = false;
            }

            if !errors_markdown.is_empty() {
                if !first {
                    println!()
                }
                println!("# 💥 Errors \n");
                for line in errors_markdown {
                    println!("{line}");
                }
                first = false;
            }

            if first {
                println!("# ✅ Release Checks Passed!\n");
            }
        }

        if self.exit_status && any_issue {
            anyhow::bail!("exit requested at any error");
        }

        tracing::info!("complete");

        Ok(())
    }
}

impl Package {
    #[tracing::instrument(skip_all, fields(package = %self.name))]
    pub fn check(
        &self,
        packages: &BTreeMap<String, Self>,
        workspace_root: &Utf8Path,
        base_branch: Option<&str>,
    ) -> anyhow::Result<()> {
        if !base_branch.is_none_or(|branch| self.has_branch_changes(branch)) {
            tracing::debug!("skipping due to no changes run with --all to check this package");
            return Ok(());
        }

        let start = std::time::Instant::now();
        tracing::debug!("starting validating");

        let license = if self.license.is_none() && self.license_file.is_none() {
            self.report_issue(PackageErrorMissing::License);
            LicenseKind::from_text(LicenseKind::MIT_OR_APACHE2)
        } else if let Some(license) = &self.license {
            LicenseKind::from_text(license)
        } else {
            None
        };

        if let Some(license) = license {
            for kind in license {
                if !self
                    .manifest_path
                    .with_file_name(PackageFile::License(kind).to_string())
                    .exists()
                {
                    self.report_issue(PackageFile::License(kind));
                }
            }
        }

        if self.should_release() && !self.manifest_path.with_file_name(PackageFile::Readme.to_string()).exists() {
            self.report_issue(PackageFile::Readme);
        }

        if self.changelog_path().is_some_and(|path| !path.exists()) {
            self.report_issue(PackageFile::Changelog);
        }

        if self.should_release() && self.description.is_none() {
            self.report_issue(PackageErrorMissing::Description);
        }

        if self.should_release() && self.readme.is_none() {
            self.report_issue(PackageErrorMissing::Readme);
        }

        if self.should_release() && self.repository.is_none() {
            self.report_issue(PackageErrorMissing::Repopository);
        }

        if self.should_release() && self.authors.is_empty() {
            self.report_issue(PackageErrorMissing::Author);
        }

        if self.should_release() && self.documentation.is_none() {
            self.report_issue(PackageErrorMissing::Documentation);
        }

        for dep in &self.dependencies {
            match &dep.kind {
                DependencyKind::Build | DependencyKind::Normal => {
                    if let Some(Some(pkg)) = dep.path.is_some().then(|| packages.get(&dep.name)) {
                        if dep.req.comparators.is_empty() && self.should_publish() {
                            self.report_issue(PackageError::missing_version(dep));
                        } else if pkg.group() == self.group()
                            && dep.req.comparators
                                != [semver::Comparator {
                                    major: self.version.major,
                                    minor: Some(self.version.minor),
                                    patch: Some(self.version.patch),
                                    op: semver::Op::Exact,
                                    pre: self.version.pre.clone(),
                                }]
                        {
                            self.report_issue(PackageError::grouped_version(dep));
                        }
                    } else if self.should_publish() {
                        if dep.registry.is_some()
                            || dep.req.comparators.is_empty()
                            || dep.source.as_ref().is_some_and(|s| !s.starts_with("registry"))
                        {
                            self.report_issue(PackageError::not_publish(dep));
                        }
                    }
                }
                DependencyKind::Development => {
                    if !dep.req.comparators.is_empty() && dep.path.is_some() && packages.contains_key(&dep.name) {
                        self.report_issue(PackageError::has_version(dep));
                    }
                }
                _ => continue,
            }
        }

        // if we aren't in a pr or there is pr changes
        // then
        if let Some(commit) = self.last_git_commit().context("lookup commit")? {
            tracing::debug!("found git changes at {commit}");
            self.report_change();
        }

        if self.should_semver_checks() {
            if let Some(version) = self.last_published_version() {
                static ONCE: std::sync::Once = std::sync::Once::new();
                ONCE.call_once(|| {
                    std::thread::spawn(move || {
                        tracing::info!("running cargo-semver-checks");
                    });
                });

                tracing::debug!(
                    "running semver-checks against baseline-version: {}, current-version: {}",
                    version.vers,
                    self.version
                );

                let semver_checks = cargo_cmd()
                    .arg("semver-checks")
                    .arg("-p")
                    .arg(&self.name)
                    .arg("--baseline-version")
                    .arg(version.vers.to_string())
                    .stderr(Stdio::piped())
                    .stdout(Stdio::piped())
                    .output()
                    .context("semver-checks")?;

                let stdout = String::from_utf8_lossy(&semver_checks.stdout);
                let stdout = stdout.trim().replace(workspace_root.as_str(), ".");
                if !semver_checks.status.success() {
                    let stderr = String::from_utf8_lossy(&semver_checks.stderr);
                    let stderr = stderr.trim().replace(workspace_root.as_str(), ".");
                    if stdout.is_empty() {
                        anyhow::bail!("semver-checks failed\n{stderr}");
                    } else {
                        self.set_semver_output(stderr.contains("requires new major version"), stdout.to_owned());
                    }
                } else {
                    self.set_semver_output(false, stdout.to_owned());
                }
            } else {
                tracing::info!("skipping semver-checks due to no published-version");
            }
        }

        if self.should_min_version_check() {
            let cargo_toml_str = std::fs::read_to_string(&self.manifest_path).context("read Cargo.toml")?;
            let mut cargo_toml_edit = cargo_toml_str.parse::<toml_edit::DocumentMut>().context("parse Cargo.toml")?;

            // Remove dev-dependencies to prevent them from effecting cargo's version resolution.
            cargo_toml_edit.remove("dev-dependencies");
            if let Some(target) = cargo_toml_edit.get_mut("target").and_then(|t| t.as_table_like_mut()) {
                for (_, item) in target.iter_mut() {
                    if let Some(table) = item.as_table_like_mut() {
                        table.remove("dev-dependencies");
                    }
                }
            }

            let mut dep_packages = Vec::new();
            for dep in &self.dependencies {
                let Some(pkg) = packages.get(&dep.name) else {
                    continue;
                };

                if dep.path.is_none() {
                    continue;
                }

                let root = if let Some(target) = &dep.target {
                    &mut cargo_toml_edit["target"][&target.to_string()]
                } else {
                    cargo_toml_edit.as_item_mut()
                };

                let kind = match dep.kind {
                    DependencyKind::Build => "build-dependencies",
                    DependencyKind::Normal => "dependencies",
                    _ => continue,
                };

                let item = root[kind][&dep.name].as_table_like_mut().unwrap();
                let versions = pkg.published_versions();

                tracing::debug!(
                    "min-version-check: finding best version for {} = '{}' outof [{}]",
                    dep.name,
                    dep.req,
                    versions.iter().map(|v| v.vers.to_string()).collect::<Vec<_>>().join(", ")
                );

                let version = versions
                    .iter()
                    .find(|v| dep.req.matches(&v.vers))
                    .map(|v| &v.vers)
                    .unwrap_or(&pkg.version);
                let pinned = semver::VersionReq {
                    comparators: vec![semver::Comparator {
                        op: semver::Op::Exact,
                        major: version.major,
                        minor: Some(version.minor),
                        patch: Some(version.patch),
                        pre: version.pre.clone(),
                    }],
                };

                if version != &pkg.version || pkg.last_published_version().is_some_and(|v| v.vers == pkg.version) {
                    item.remove("path");
                } else {
                    dep_packages.push(&pkg.name);
                }

                item.insert("version", pinned.to_string().into());
            }

            static ONCE: std::sync::Once = std::sync::Once::new();
            ONCE.call_once(|| {
                std::thread::spawn(move || {
                    tracing::info!("running min versions check");
                });
            });

            static SINGLE_THREAD: std::sync::Mutex<()> = std::sync::Mutex::new(());

            let _lock = SINGLE_THREAD.lock().unwrap();
            let _undo = WriteUndo::new(
                &self.manifest_path,
                cargo_toml_edit.to_string().as_bytes(),
                cargo_toml_str.into_bytes(),
            )?;

            let (mut read, write) = std::io::pipe()?;

            let mut cmd = cargo_cmd();
            cmd.env("RUSTC_BOOTSTRAP", "1")
                .stderr(write.try_clone()?)
                .stdout(write)
                .arg("-Zunstable-options")
                .arg("-Zpackage-workspace")
                .arg("publish")
                .arg("--dry-run")
                .arg("--allow-dirty")
                .arg("--all-features")
                .arg("--lockfile-path")
                .arg(workspace_root.join("target").join("release-checks").join("Cargo.lock"))
                .arg("--target-dir")
                .arg(workspace_root.join("target").join("release-checks"))
                .arg("-p")
                .arg(&self.name);

            for package in &dep_packages {
                cmd.arg("-p").arg(package);
            }

            let mut child = cmd.spawn().context("spawn")?;

            drop(cmd);

            let mut output = String::new();
            read.read_to_string(&mut output).context("invalid read")?;

            let result = child.wait().context("wait")?;
            if !result.success() {
                self.set_min_versions_output(output);
            }
        }

        if let Some(next_version) = self.next_version() {
            tracing::debug!(after = ?start.elapsed(), "validation finished, package needs a version bump ({next_version})");
        } else {
            tracing::debug!(after = ?start.elapsed(), "validation finished");
        }

        Ok(())
    }
}

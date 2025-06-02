use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write;

use anyhow::Context;
use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::semver::Version;
use cargo_metadata::{DependencyKind, semver};
use serde::Deserialize as _;
use serde::de::IntoDeserializer;
use serde_derive::{Deserialize, Serialize};
use toml_edit::{DocumentMut, Table};

use super::check::CheckRun;
use super::utils::VersionBump;
use crate::cmd::release::utils::vers_to_comp;
use crate::utils::git_workdir_clean;

#[derive(Debug, Clone, clap::Parser)]
pub struct Update {
    /// Concurrency to run at. By default, this is the total number of cpus on the host.
    #[arg(long, default_value_t = num_cpus::get())]
    concurrency: usize,
    /// Run the command without modifying any files on disk
    #[arg(long)]
    dry_run: bool,
    /// Allow the command to execute even if there are uncomitted changes in the workspace
    #[arg(long)]
    allow_dirty: bool,
    /// Packages to include in the check
    /// by default all packages are included
    #[arg(long = "package", short = 'p')]
    packages: Vec<String>,
    /// Only generate the changelogs, not the version bumps.
    #[arg(long)]
    changelogs_only: bool,
}

impl Update {
    pub fn run(self) -> anyhow::Result<()> {
        if !self.allow_dirty {
            git_workdir_clean()?;
        }

        let metadata = crate::utils::metadata()?;

        let check_run = CheckRun::new(&metadata, &self.packages).context("check run")?;

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

        let dep_graph = check_run
            .all_packages()
            .map(|package| {
                (
                    package.name.as_str(),
                    package
                        .dependencies
                        .iter()
                        .filter(|dep| {
                            dep.path.is_some() && matches!(dep.kind, DependencyKind::Build | DependencyKind::Normal)
                        })
                        .map(|dep| (dep.name.as_str(), dep))
                        .collect::<BTreeMap<_, _>>(),
                )
            })
            .collect::<HashMap<_, _>>();

        let inverted_dep_graph = dep_graph
            .iter()
            .fold(HashMap::<_, Vec<_>>::new(), |mut inverted, (package, deps)| {
                deps.iter().for_each(|(name, dep)| {
                    inverted.entry(*name).or_default().push((*package, *dep));
                });
                inverted
            });

        let flattened_dep_public_graph = dep_graph
            .iter()
            .map(|(package, deps)| {
                let mut seen = HashSet::new();
                let pkg = check_run.get_package(package).unwrap();
                (
                    *package,
                    deps.iter().fold(HashMap::<_, Vec<_>>::new(), |mut deps, (name, dep)| {
                        let mut stack = vec![(pkg, check_run.get_package(name).unwrap(), *dep)];

                        while let Some((pkg, dep_pkg, dep)) = stack.pop() {
                            if pkg.is_dep_public(&dep.name) {
                                deps.entry(dep_pkg.name.as_str()).or_default().push(dep);
                                if seen.insert(&dep_pkg.name) {
                                    stack.extend(
                                        dep_graph
                                            .get(dep_pkg.name.as_str())
                                            .into_iter()
                                            .flatten()
                                            .map(|(name, dep)| (pkg, check_run.get_package(name).unwrap(), *dep)),
                                    );
                                }
                            }
                        }

                        deps
                    }),
                )
            })
            .collect::<HashMap<_, _>>();

        if !self.changelogs_only {
            check_run.process(self.concurrency, &metadata.workspace_root, None)?;

            for fragment in change_fragments.values() {
                for (package, logs) in fragment.items().context("fragment items")? {
                    let Some(pkg) = check_run.get_package(&package) else {
                        tracing::warn!("unknown package: {package}");
                        continue;
                    };

                    pkg.report_change();
                    if logs.iter().any(|l| l.breaking) {
                        pkg.report_breaking_change();
                    }
                }
            }

            let mut found = false;
            for iter in 0..10 {
                let mut has_changes = false;
                for group in check_run.all_groups() {
                    let max_bump_version = group
                        .iter()
                        .map(|p| {
                            match (p.last_published_version(), p.version_bump()) {
                                // There has never been a published version
                                // or there is no bump
                                (None, _) | (_, None) => p.version.clone(),
                                // The last published version is the current version
                                (Some(last_published), Some(bump)) if last_published.vers == p.version => {
                                    bump.next_semver(p.version.clone())
                                }
                                // Last published version is a different version
                                (Some(last_published), Some(bump)) => {
                                    // determine if the last published version is a minor or major version away.
                                    if bump == VersionBump::Major
                                        && !vers_to_comp(last_published.vers.clone()).matches(&p.version)
                                    {
                                        bump.next_semver(last_published.vers)
                                    } else {
                                        p.version.clone()
                                    }
                                }
                            }
                        })
                        .max()
                        .unwrap();

                    group
                        .iter()
                        .filter(|package| package.version != max_bump_version)
                        .for_each(|package| {
                            inverted_dep_graph
                                .get(package.name.as_ref())
                                .into_iter()
                                .flatten()
                                .for_each(|(pkg, dep)| {
                                    if !dep.req.matches(&max_bump_version) || dep.req == package.unreleased_req() {
                                        let pkg = check_run.get_package(pkg).unwrap();
                                        if pkg.is_dep_public(&dep.name) {
                                            pkg.report_breaking_change();
                                        } else {
                                            pkg.report_change();
                                        }
                                    }
                                });
                        });

                    group.iter().for_each(|p| {
                        if p.version != max_bump_version && p.next_version().is_none_or(|v| v != max_bump_version) {
                            tracing::debug!("{} to {} -> {max_bump_version}", p.name, p.version);
                            p.set_next_version(max_bump_version.clone());
                            has_changes = true;
                        }
                    });
                }

                if !has_changes {
                    tracing::debug!("satisfied version constraints after {} iterations", iter + 1);
                    found = true;
                    break;
                }
            }

            if !found {
                anyhow::bail!("could not satisfy version constraints after 10 attempts");
            }

            for package in check_run.groups().flatten() {
                let deps = dep_graph.get(package.name.as_str()).unwrap();
                for dep in deps.values() {
                    let dep_pkg = check_run.get_package(&dep.name).unwrap();

                    let depends_on = dep.req == dep_pkg.unreleased_req()
                        || dep_pkg.last_published_version().is_none_or(|v| !dep.req.matches(&v.vers))
                        // we want to find out if any deps have a major semver change
                        // and a peer dependency is dependent on an older version as a public dep.
                        || flattened_dep_public_graph.get(dep.name.as_str()).unwrap().iter().any(|(inner_dep_name, reqs)| {
                            let inner_dep_pkg = check_run.get_package(inner_dep_name).unwrap();
                            deps.contains_key(inner_dep_name) // if we are also dependant
                                && package.is_dep_public(inner_dep_name) // its also a public dep
                                && check_run.is_accepted_group(inner_dep_pkg.group()) // if the dep is part of the release group
                                && inner_dep_pkg.next_version().is_some_and(|vers| reqs.iter().any(|dep_req| !dep_req.req.matches(&vers)))
                        });

                    if depends_on && !check_run.is_accepted_group(dep_pkg.group()) {
                        anyhow::bail!(
                            "could not update: `{}` because it depends on `{}` which is not part of the packages to be updated.",
                            package.name,
                            dep_pkg.name
                        );
                    }
                }
            }
        }

        let mut pr_body = String::from("## 🤖 New release\n\n");
        let mut release_count = 0;
        let workspace_manifest_path = metadata.workspace_root.join("Cargo.toml");

        let mut workspace_manifest = if !self.changelogs_only {
            let workspace_manifest = std::fs::read_to_string(&workspace_manifest_path).context("workspace manifest read")?;
            Some((
                workspace_manifest
                    .parse::<toml_edit::DocumentMut>()
                    .context("workspace manifest parse")?,
                workspace_manifest,
            ))
        } else {
            None
        };

        let mut workspace_metadata_packages = if let Some((workspace_manifest, _)) = &mut workspace_manifest {
            let mut item = workspace_manifest.as_item_mut().as_table_like_mut().expect("table");
            for key in ["workspace", "metadata", "xtask", "release", "packages"] {
                item = item
                    .entry(key)
                    .or_insert(toml_edit::Item::Table({
                        let mut table = Table::new();
                        table.set_implicit(true);
                        table
                    }))
                    .as_table_like_mut()
                    .expect("table");
            }

            Some(item)
        } else {
            None
        };

        for package in check_run.all_packages() {
            let _span = tracing::info_span!("update", package = %package.name).entered();
            let version = package.next_version().or_else(|| {
                if package.should_publish() && package.last_published_version().is_none() && !self.changelogs_only {
                    Some(package.version.clone())
                } else {
                    None
                }
            });
            if !self.changelogs_only && version.is_none() {
                continue;
            }

            release_count += 1;

            let is_accepted_group = check_run.is_accepted_group(package.group());

            let mut changelogs = if is_accepted_group {
                if let Some(change_log_path_md) = package.changelog_path() {
                    Some((
                        change_log_path_md,
                        generate_change_logs(&package.name, &mut change_fragments).context("generate")?,
                    ))
                } else {
                    None
                }
            } else {
                None
            };

            if !self.changelogs_only {
                let version = version.as_ref().unwrap();
                if is_accepted_group {
                    pr_body.push_str(
                        &fmtools::fmt(|mut f| {
                            let mut f = indent_write::fmt::IndentWriter::new("  ", &mut f);
                            write!(f, "* `{}`: ", package.name)?;
                            let last_published = package.last_published_version();
                            f.write_str(match &last_published {
                                None => " 📦 **New Crate**",
                                Some(v) if vers_to_comp(v.vers.clone()).matches(version) => " ✨ **Minor**",
                                Some(_) => " 🚀 **Major**",
                            })?;

                            let mut f = indent_write::fmt::IndentWriter::new("  ", f);
                            match &last_published {
                                Some(base) => write!(f, "\n* Version: **`{}`** ➡️ **`{version}`**", base.vers)?,
                                None => write!(f, "\n* Version: **`{version}`**")?,
                            }

                            if package.group() != package.name.as_str() {
                                write!(f, " (group: **`{}`**)", package.group())?;
                            }
                            f.write_str("\n")?;
                            Ok(())
                        })
                        .to_string(),
                    );

                    if let Some(workspace_metadata_packages) = &mut workspace_metadata_packages {
                        workspace_metadata_packages.insert(package.name.as_str(), version.to_string().into());
                    }
                }
                let cargo_toml_raw = std::fs::read_to_string(&package.manifest_path).context("read cargo toml")?;
                let mut cargo_toml_edit = cargo_toml_raw.parse::<toml_edit::DocumentMut>().context("parse toml")?;
                if is_accepted_group {
                    cargo_toml_edit["package"]["version"] = version.to_string().into();
                }

                tracing::debug!("checking deps");

                for dep in &package.dependencies {
                    if dep.path.is_none() {
                        continue;
                    }

                    let kind = match dep.kind {
                        DependencyKind::Build => "build-dependencies",
                        DependencyKind::Normal => "dependencies",
                        _ => continue,
                    };

                    let Some(pkg) = check_run.get_package(&dep.name) else {
                        continue;
                    };

                    if !check_run.is_accepted_group(pkg.group()) {
                        continue;
                    }

                    let depends_on = dep.req == pkg.unreleased_req();
                    if !depends_on && pkg.next_version().is_none_or(|vers| dep.req.matches(&vers)) {
                        tracing::debug!("skipping version update on {}", dep.name);
                        continue;
                    }

                    let root = if let Some(target) = &dep.target {
                        &mut cargo_toml_edit["target"][&target.to_string()]
                    } else {
                        cargo_toml_edit.as_item_mut()
                    };

                    let item = root[kind][&dep.name].as_table_like_mut().unwrap();
                    let pkg_version = pkg.next_version().unwrap_or_else(|| pkg.version.clone());

                    let version = if pkg.group() == package.group() {
                        semver::VersionReq {
                            comparators: vec![semver::Comparator {
                                op: semver::Op::Exact,
                                major: pkg_version.major,
                                minor: Some(pkg_version.minor),
                                patch: Some(pkg_version.patch),
                                pre: pkg_version.pre.clone(),
                            }],
                        }
                        .to_string()
                    } else {
                        if !depends_on {
                            if let Some((_, changelogs)) = changelogs.as_mut() {
                                let mut log =
                                    PackageChangeLog::new("chore", format!("bump {} to `{pkg_version}`", dep.name));
                                log.breaking = package.is_dep_public(&dep.name);
                                changelogs.push(log)
                            }
                        }

                        pkg_version.to_string()
                    };

                    item.insert("version", version.into());
                }

                let cargo_toml = cargo_toml_edit.to_string();
                if cargo_toml != cargo_toml_raw {
                    if !self.dry_run {
                        tracing::debug!("updating {}", package.manifest_path);
                        std::fs::write(&package.manifest_path, cargo_toml).context("write cargo toml")?;
                    } else {
                        tracing::warn!("not modifying {} because dry-run", package.manifest_path);
                    }
                }
            }

            if let Some((change_log_path_md, changelogs)) = changelogs {
                update_change_log(
                    &changelogs,
                    &change_log_path_md,
                    &package.name,
                    version.as_ref(),
                    package.last_published_version().map(|v| v.vers).as_ref(),
                    self.dry_run,
                )
                .context("update")?;
                if !self.dry_run {
                    save_change_fragments(&mut change_fragments).context("save")?;
                }
                tracing::info!(package = %package.name, "updated change logs");
            }
        }

        if let Some((workspace_manifest, workspace_manifest_str)) = workspace_manifest {
            let workspace_manifest = workspace_manifest.to_string();
            if workspace_manifest != workspace_manifest_str {
                if self.dry_run {
                    tracing::warn!("skipping write of {workspace_manifest_path}")
                } else {
                    std::fs::write(&workspace_manifest_path, workspace_manifest).context("write workspace metadata")?;
                }
            }
        }

        if release_count != 0 {
            println!("{}", pr_body.trim());
        } else {
            tracing::info!("no packages to release!");
        }

        Ok(())
    }
}

fn update_change_log(
    logs: &[PackageChangeLog],
    change_log_path_md: &Utf8Path,
    name: &str,
    version: Option<&Version>,
    previous_version: Option<&Version>,
    dry_run: bool,
) -> anyhow::Result<()> {
    let mut change_log = std::fs::read_to_string(change_log_path_md).context("failed to read CHANGELOG.md")?;

    // Find the # [Unreleased] section
    // So we can insert the new logs after it
    let (mut breaking_changes, mut other_changes) = logs.iter().partition::<Vec<_>, _>(|log| log.breaking);
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
                write!(f, "- {category}: {tag}", category = log.category, tag = tag.trim(),)?;

                if !log.pr_numbers.is_empty() {
                    f.write_str(" (")?;
                    let mut first = true;
                    for pr_number in &log.pr_numbers {
                        if !first {
                            f.write_str(", ")?;
                        }
                        first = false;
                        write!(f, "[#{pr_number}](https://github.com/scufflecloud/scuffle/pull/{pr_number})")?;
                    }
                    f.write_str(")")?;
                }

                if !log.authors.is_empty() {
                    f.write_str(" (")?;
                    let mut first = true;
                    let mut seen = HashSet::new();
                    for author in &log.authors {
                        let author = author.trim().trim_start_matches('@').trim();
                        if !seen.insert(author.to_lowercase()) {
                            continue;
                        }

                        if !first {
                            f.write_str(", ")?;
                        }
                        first = false;
                        f.write_char('@')?;
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

    replaced.push_str("## [Unreleased]\n");

    if let Some(version) = version {
        replaced.push_str(&format!(
            "\n## [{version}](https://github.com/ScuffleCloud/scuffle/releases/tag/{name}-v{version}) - {date}\n\n",
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

    change_log = change_log.replace("## [Unreleased]", replaced.trim());

    if !dry_run {
        std::fs::write(change_log_path_md, change_log).context("failed to write CHANGELOG.md")?;
    } else {
        tracing::warn!("not modifying {change_log_path_md} because dry-run");
    }

    Ok(())
}

fn generate_change_logs(
    package: &str,
    change_fragments: &mut BTreeMap<u64, Fragment>,
) -> anyhow::Result<Vec<PackageChangeLog>> {
    let mut logs = Vec::new();
    let mut seen_logs = HashMap::new();

    for fragment in change_fragments.values_mut() {
        for log in fragment.remove_package(package).context("parse")? {
            let key = (log.category.clone(), log.description.clone());
            match seen_logs.entry(key) {
                std::collections::hash_map::Entry::Vacant(v) => {
                    v.insert(logs.len());
                    logs.push(log);
                }
                std::collections::hash_map::Entry::Occupied(o) => {
                    let old_log = &mut logs[*o.get()];
                    old_log.pr_numbers.extend(log.pr_numbers);
                    old_log.authors.extend(log.authors);
                    old_log.breaking |= log.breaking;
                }
            }
        }
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

#[derive(Debug, Clone)]
pub struct Fragment {
    path: Utf8PathBuf,
    pr_number: u64,
    toml: toml_edit::DocumentMut,
    changed: bool,
    deleted: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PackageChangeLog {
    #[serde(skip, default)]
    pub pr_numbers: BTreeSet<u64>,
    #[serde(alias = "cat")]
    pub category: String,
    #[serde(alias = "desc")]
    pub description: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[serde(alias = "author")]
    pub authors: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    #[serde(alias = "break", alias = "major")]
    pub breaking: bool,
}

fn is_false(input: &bool) -> bool {
    !*input
}

impl PackageChangeLog {
    pub fn new(category: impl std::fmt::Display, desc: impl std::fmt::Display) -> Self {
        Self {
            pr_numbers: BTreeSet::new(),
            authors: Vec::new(),
            breaking: false,
            category: category.to_string(),
            description: desc.to_string(),
        }
    }
}

impl Fragment {
    pub fn new(pr_number: u64, root: &Utf8Path) -> anyhow::Result<Self> {
        let path = root.join("changes.d").join(format!("pr-{pr_number}.toml"));
        if path.exists() {
            let content = std::fs::read_to_string(&path).context("read")?;
            Ok(Fragment {
                pr_number,
                path: path.to_path_buf(),
                toml: content
                    .parse::<toml_edit::DocumentMut>()
                    .context("change log is not valid toml")?,
                changed: false,
                deleted: false,
            })
        } else {
            Ok(Fragment {
                changed: false,
                deleted: true,
                path: path.to_path_buf(),
                pr_number,
                toml: DocumentMut::new(),
            })
        }
    }

    pub fn changed(&self) -> bool {
        self.changed
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }

    pub fn path(&self) -> &Utf8Path {
        &self.path
    }

    pub fn has_package(&self, package: &str) -> bool {
        self.toml.contains_key(package)
    }

    pub fn items(&self) -> anyhow::Result<BTreeMap<String, Vec<PackageChangeLog>>> {
        self.toml
            .iter()
            .map(|(package, item)| package_to_logs(self.pr_number, item.clone()).map(|logs| (package.to_owned(), logs)))
            .collect()
    }

    pub fn add_log(&mut self, package: &str, log: &PackageChangeLog) {
        if !self.toml.contains_key(package) {
            self.toml.insert(package, toml_edit::Item::ArrayOfTables(Default::default()));
        }

        self.changed = true;

        self.toml[package]
            .as_array_of_tables_mut()
            .unwrap()
            .push(toml_edit::ser::to_document(log).expect("invalid log").as_table().clone())
    }

    pub fn remove_package(&mut self, package: &str) -> anyhow::Result<Vec<PackageChangeLog>> {
        let Some(items) = self.toml.remove(package) else {
            return Ok(Vec::new());
        };

        self.changed = true;

        package_to_logs(self.pr_number, items)
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        if !self.changed {
            return Ok(());
        }

        if self.toml.is_empty() {
            if !self.deleted {
                tracing::debug!(path = %self.path, "removing change fragment cause empty");
                std::fs::remove_file(&self.path).context("remove")?;
                self.deleted = true;
            }
        } else {
            tracing::debug!(path = %self.path, "saving change fragment");
            std::fs::write(&self.path, self.toml.to_string()).context("write")?;
            self.deleted = false;
        }

        self.changed = false;

        Ok(())
    }
}

fn package_to_logs(pr_number: u64, items: toml_edit::Item) -> anyhow::Result<Vec<PackageChangeLog>> {
    let value = items.into_value().expect("items must be a value").into_deserializer();
    let mut logs = Vec::<PackageChangeLog>::deserialize(value).context("deserialize")?;

    logs.iter_mut().for_each(|log| {
        log.category = log.category.to_lowercase();
        log.pr_numbers = BTreeSet::from_iter([pr_number]);
    });

    Ok(logs)
}

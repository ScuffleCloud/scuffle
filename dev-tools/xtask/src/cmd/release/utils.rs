use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Condvar, Mutex};

use anyhow::Context;
use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::{Dependency, DependencyKind, semver};
use cargo_platform::Platform;
use serde::Deserialize as _;
use serde::de::IntoDeserializer;
use serde_derive::{Deserialize, Serialize};
use sha2::Digest;
use toml_edit::DocumentMut;

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
    pub pr_number: u64,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_bump: Option<(String, semver::Version)>,
}

fn is_false(input: &bool) -> bool {
    !*input
}

impl PackageChangeLog {
    pub fn new(category: impl std::fmt::Display, desc: impl std::fmt::Display) -> Self {
        Self {
            pr_number: 0,
            authors: Vec::new(),
            breaking: false,
            category: category.to_string(),
            description: desc.to_string(),
            version_bump: None,
        }
    }

    pub fn with_version_bump(self, package: impl std::fmt::Display, version: semver::Version) -> Self {
        Self {
            version_bump: Some((package.to_string(), version)),
            ..self
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
        log.pr_number = pr_number;
    });

    Ok(logs)
}

pub struct Package {
    pkg: cargo_metadata::Package,
    published_versions: Mutex<Vec<CratesIoVersion>>,
    data: Mutex<PackageData>,
    metadata: XTaskPackageMetadata,
}

impl std::ops::Deref for Package {
    type Target = cargo_metadata::Package;

    fn deref(&self) -> &Self::Target {
        &self.pkg
    }
}

#[derive(serde_derive::Deserialize, Default, Debug)]
#[serde(default, rename_all = "kebab-case")]
struct XTaskPackageMetadata {
    /// The group this crate belongs to
    ///
    /// Groups make sure versions are lock-stepped, and also ensure
    /// that changelogs are shared across all members of the group.
    /// Group's also ensure that all crates in the same group are in
    /// a single git release.
    ///
    /// **Default**: `name of crate`
    group: Option<String>,
    /// Toggle whether a git release / tag should be created when the
    /// crate is released. For controlling publishing to crates.io,
    /// look at the `publish = false` flag built-in to cargo.
    /// Git releases are always disabled if the crate is not the main
    /// member of its group.
    ///
    /// **Default**: `same as publish`
    git_release: Option<bool>,
    /// The tag name to use when doing a git release.
    ///
    /// By default this is `{{ package }}-v{{ version }}`
    ///
    /// Variables:
    /// - **package**: The name of the package.
    /// - **version**: The version of the package.
    git_tag_name: Option<String>,
    /// The body to use in the github release.
    ///
    /// By default this is:
    /// ```jinja
    /// {% if publish %}
    /// [<img alt="crates.io" src="https://img.shields.io/badge/crates.io-v{{ version }}-orange?labelColor=5C5C5C" height="20">](https://crates.io/crates/{{ package }}/{{ version }})
    /// [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-v{{ version }}-blue?labelColor=5C5C5C" height="20">](https://docs.rs/{{ package }}/{{ version }})
    /// {% endif %}
    /// {{ changelog }}
    /// ```
    ///
    /// Variables:
    /// - **package**: The name of the package.
    /// - **version**: The version of the package.
    /// - **changelog**: The changelog entry for this release.
    /// - **publish**: a boolean if the crate is being published to crates.io
    git_release_body: Option<String>,
    /// The release name to use when doing a git release.
    ///
    /// By default this is the same as the git tag name.
    ///
    /// Variables:
    /// - **package**: The name of the package.
    /// - **version**: The version of the package.
    git_release_name: Option<String>,
    /// Toggles whether to run cargo-semver-checks to detect breaking public
    /// api changes when determining the next version of the crate.
    ///
    /// **Default**: `same as publish`
    semver_checks: Option<bool>,
    /// Toggles whether to run a minimum version check to ensure the package
    /// manifest has the correct version ranges.
    ///
    /// **Default**: `same as publish`
    min_versions_checks: Option<bool>,
    /// Controls how versions are bumped / released.
    style: Option<VersionBumpStyle>,
}

impl XTaskPackageMetadata {
    fn from_package(package: &cargo_metadata::Package) -> anyhow::Result<Self> {
        let Some(metadata) = package.metadata.get("xtask").and_then(|v| v.get("release")) else {
            return Ok(Self::default());
        };

        serde_json::from_value(metadata.clone()).context("xtask")
    }
}

#[derive(serde_derive::Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum VersionBumpStyle {
    /// Requires the version of the crate to be bumped in the pr
    /// that makes the change.
    OnChange,
    /// Verisons will be updated manually.
    Manual,
}

impl VersionBumpStyle {
    pub fn is_manual(&self) -> bool {
        matches!(self, Self::Manual)
    }
}

#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum VersionBump {
    #[default]
    None = 0,
    Minor = 1,
    Major = 2,
}

impl VersionBump {
    fn bump(&mut self, new: Self) -> &mut Self {
        *self = new.max(*self);
        self
    }

    fn bump_major(&mut self) -> &mut Self {
        self.bump(Self::Major)
    }

    fn bump_minor(&mut self) -> &mut Self {
        self.bump(Self::Minor)
    }

    fn next_semver(&self, version: semver::Version) -> semver::Version {
        match self {
            Self::None => version,
            // pre-release always bump that
            _ if !version.pre.is_empty() => semver::Version {
                pre: semver::Prerelease::new(&increment_last_identifier(&version.pre))
                    .expect("pre release increment failed, this is a bug"),
                ..version
            },
            // 0.0.x always bump patch
            _ if version.major == 0 && version.minor == 0 => semver::Version {
                patch: version.patch + 1,
                ..version
            },
            // 0.x.y => 0.(x + 1).0
            Self::Major if version.major == 0 => semver::Version {
                minor: version.minor + 1,
                patch: 0,
                ..version
            },
            // x.y.z => (x + 1).0.0
            Self::Major => semver::Version {
                major: version.major + 1,
                minor: 0,
                patch: 0,
                ..version
            },
            // 0.x.y => 0.x.(y + 1)
            Self::Minor if version.major == 0 => semver::Version {
                patch: version.patch + 1,
                ..version
            },
            // x.y.z => x.(y + 1).0
            Self::Minor => semver::Version {
                minor: version.minor + 1,
                patch: 0,
                ..version
            },
        }
    }
}

fn increment_last_identifier(release: &str) -> String {
    match release.rsplit_once('.') {
        Some((left, right)) => {
            if let Ok(right_num) = right.parse::<u32>() {
                format!("{left}.{}", right_num + 1)
            } else {
                format!("{release}.1")
            }
        }
        None => format!("{release}.1"),
    }
}

#[derive(Clone, Copy)]
pub enum PackageErrorMissing {
    Description,
    License,
    Readme,
    Repopository,
    Author,
    Documentation,
    ChangelogEntry,
}

impl std::fmt::Display for PackageErrorMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Description => f.write_str("description in Cargo.toml"),
            Self::License => f.write_str("license in Cargo.toml"),
            Self::Readme => f.write_str("readme file path in Cargo.toml"),
            Self::Repopository => f.write_str("repository link in Cargo.toml"),
            Self::Author => f.write_str("authors in Cargo.toml"),
            Self::Documentation => f.write_str("documentation link in Cargo.toml"),
            Self::ChangelogEntry => f.write_str("changelog entry"),
        }
    }
}

impl From<PackageErrorMissing> for PackageError {
    fn from(value: PackageErrorMissing) -> Self {
        PackageError::Missing(value)
    }
}

#[derive(Clone, Copy)]
pub enum LicenseKind {
    Mit,
    Apache2,
    AGpl3,
}

impl LicenseKind {
    pub const AGPL_3: &str = "AGPL-3.0";
    const APACHE2: &str = "Apache-2.0";
    const MIT: &str = "MIT";
    pub const MIT_OR_APACHE2: &str = "MIT OR Apache-2.0";

    pub fn from_text(text: &str) -> Option<Vec<LicenseKind>> {
        match text {
            Self::MIT_OR_APACHE2 => Some(vec![LicenseKind::Mit, LicenseKind::Apache2]),
            Self::AGPL_3 => Some(vec![LicenseKind::AGpl3]),
            _ => None,
        }
    }
}

impl std::fmt::Display for LicenseKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mit => f.write_str(Self::MIT),
            Self::Apache2 => f.write_str(Self::APACHE2),
            Self::AGpl3 => f.write_str(Self::AGPL_3),
        }
    }
}

#[derive(Clone, Copy)]
pub enum PackageFile {
    License(LicenseKind),
    Readme,
    Changelog,
}

impl std::fmt::Display for PackageFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Changelog => f.write_str("CHANGELOG.md"),
            Self::Readme => f.write_str("README.md"),
            Self::License(name) => write!(f, "LICENSE.{name}"),
        }
    }
}

impl From<PackageFile> for PackageError {
    fn from(value: PackageFile) -> Self {
        PackageError::MissingFile(value)
    }
}

#[derive(Clone)]
pub enum PackageError {
    Missing(PackageErrorMissing),
    MissingFile(PackageFile),
    DependencyMissingVersion {
        name: String,
        target: Option<Platform>,
        kind: DependencyKind,
    },
    DevDependencyHasVersion {
        name: String,
        target: Option<Platform>,
    },
    DependencyGroupedVersion {
        name: String,
        target: Option<Platform>,
        kind: DependencyKind,
    },
    DependencyNotPublishable {
        name: String,
        target: Option<Platform>,
        kind: DependencyKind,
    },
}

impl PackageError {
    pub fn missing_version(dep: &Dependency) -> Self {
        Self::DependencyMissingVersion {
            kind: dep.kind,
            name: dep.name.clone(),
            target: dep.target.clone(),
        }
    }

    pub fn has_version(dep: &Dependency) -> Self {
        Self::DevDependencyHasVersion {
            name: dep.name.clone(),
            target: dep.target.clone(),
        }
    }

    pub fn not_publish(dep: &Dependency) -> Self {
        Self::DependencyNotPublishable {
            kind: dep.kind,
            name: dep.name.clone(),
            target: dep.target.clone(),
        }
    }

    pub fn grouped_version(dep: &Dependency) -> Self {
        Self::DependencyGroupedVersion {
            kind: dep.kind,
            name: dep.name.clone(),
            target: dep.target.clone(),
        }
    }
}

pub fn dep_kind_to_name(kind: &DependencyKind) -> &str {
    match kind {
        DependencyKind::Build => "build-dependencies",
        DependencyKind::Development => "dev-dependencies",
        DependencyKind::Normal => "dependencies",
        kind => panic!("unknown dep kind: {kind:?}"),
    }
}

impl std::fmt::Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing(item) => write!(f, "{item} must be provided"),
            Self::DependencyMissingVersion {
                name,
                target: Some(Platform::Cfg(cfg)),
                kind,
            } => {
                write!(
                    f,
                    "`{name}` must have a version in `[target.'{cfg}'.{kind}]`",
                    kind = dep_kind_to_name(kind)
                )
            }
            Self::DependencyMissingVersion {
                name,
                target: Some(Platform::Name(platform)),
                kind,
            } => {
                write!(
                    f,
                    "`{name}` must have a version in `[target.{platform}.{kind}]`",
                    kind = dep_kind_to_name(kind)
                )
            }
            Self::DependencyMissingVersion {
                name,
                target: None,
                kind,
            } => {
                write!(f, "`{name}` must have a version in `[{kind}]`", kind = dep_kind_to_name(kind))
            }
            Self::DevDependencyHasVersion {
                name,
                target: Some(Platform::Cfg(cfg)),
            } => {
                write!(f, "`{name}` must not have a version in `[target.'{cfg}'.dev-dependencies]`",)
            }
            Self::DevDependencyHasVersion {
                name,
                target: Some(Platform::Name(platform)),
            } => {
                write!(
                    f,
                    "`{name}` must not have a version in `[target.{platform}.dev-dependencies]`",
                )
            }
            Self::DevDependencyHasVersion { name, target: None } => {
                write!(f, "`{name}` must not have a version in `[dev-dependencies]`")
            }
            Self::DependencyNotPublishable {
                name,
                target: Some(Platform::Cfg(cfg)),
                kind,
            } => {
                write!(
                    f,
                    "`{name}` is not publishable in `[target.'{cfg}'.{kind}]`",
                    kind = dep_kind_to_name(kind)
                )
            }
            Self::DependencyNotPublishable {
                name,
                target: Some(Platform::Name(platform)),
                kind,
            } => {
                write!(
                    f,
                    "{name} is not publishable in [target.{platform}.{kind}]",
                    kind = dep_kind_to_name(kind)
                )
            }
            Self::DependencyNotPublishable {
                name,
                target: None,
                kind,
            } => {
                write!(f, "`{name}` is not publishable in `[{kind}]`", kind = dep_kind_to_name(kind))
            }
            Self::DependencyGroupedVersion {
                name,
                target: Some(Platform::Name(platform)),
                kind,
            } => {
                write!(
                    f,
                    "`{name}` must be pinned to the same version as the current crate in `[target.{platform}.{kind}]`",
                    kind = dep_kind_to_name(kind)
                )
            }
            Self::DependencyGroupedVersion {
                name,
                target: Some(Platform::Cfg(cfg)),
                kind,
            } => {
                write!(
                    f,
                    "`{name}` must be pinned to the same version as the current crate in `[target.'{cfg}'.{kind}]`",
                    kind = dep_kind_to_name(kind)
                )
            }
            Self::DependencyGroupedVersion {
                name,
                target: None,
                kind,
            } => {
                write!(
                    f,
                    "`{name}` must be pinned to the same version as the current crate in `[{kind}]`",
                    kind = dep_kind_to_name(kind)
                )
            }
            Self::MissingFile(file) => {
                write!(f, "missing file {file} in crate")
            }
        }
    }
}

#[derive(Default)]
pub struct PackageData {
    version_bump: VersionBump,
    semver_output: Option<(bool, String)>,
    min_versions_output: Option<String>,
    next_version: Option<semver::Version>,
    issues: Vec<PackageError>,
}

#[derive(serde_derive::Deserialize, Clone)]
pub struct CratesIoVersion {
    pub name: String,
    pub vers: semver::Version,
    pub cksum: String,
}

#[tracing::instrument(skip_all, fields(package = %crate_name))]
pub fn crates_io_versions(crate_name: &str) -> anyhow::Result<Vec<CratesIoVersion>> {
    let url = crate_index_url(crate_name);

    tracing::info!(url = %url, "checking on crates.io");
    let command = Command::new("curl")
        .arg("-s")
        .arg("-L")
        .arg("-w")
        .arg("\n%{http_code}\n")
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("curl")?;

    let stdout = String::from_utf8_lossy(&command.stdout);
    let stderr = String::from_utf8_lossy(&command.stderr);
    let lines = stdout.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect::<Vec<_>>();
    let status = lines.last().copied().unwrap_or_default();
    match status {
        "200" => {}
        "404" => return Ok(Vec::new()),
        status => {
            anyhow::bail!("curl failed ({status}): {stderr} {stdout}")
        }
    }

    let mut versions = Vec::new();
    for line in lines.iter().take(lines.len() - 1).copied() {
        versions.push(serde_json::from_str::<CratesIoVersion>(line).context("json")?)
    }

    versions.sort_by(|a, b| a.vers.cmp(&b.vers));

    Ok(versions)
}

fn crate_index_url(crate_name: &str) -> String {
    let name = crate_name.to_lowercase();
    let len = name.len();

    match len {
        0 => panic!("Invalid crate name"),
        1 => format!("https://index.crates.io/1/{name}"),
        2 => format!("https://index.crates.io/2/{name}"),
        3 => format!("https://index.crates.io/3/{}/{}", &name[0..1], name),
        _ => {
            let prefix = &name[0..2];
            let suffix = &name[2..4];
            format!("https://index.crates.io/{prefix}/{suffix}/{name}")
        }
    }
}

#[tracing::instrument(skip_all, fields(name = %version.name, version = %version.vers))]
pub fn download_crate(version: &CratesIoVersion) -> anyhow::Result<PathBuf> {
    let crate_file = format!("{}-{}.crate", version.name, version.vers);
    let home = home::cargo_home().context("home dir")?;
    let registry_cache = home.join("registry").join("cache");
    let mut desired_path = home.join("scuffle-xtask-release").join(&crate_file);
    let is_match = |path: &Path| {
        tracing::debug!("checking {}", path.display());
        if let Ok(read) = std::fs::read(path) {
            let hash = sha2::Sha256::digest(&read);
            let hash = hex::encode(hash);
            hash == version.cksum
        } else {
            false
        }
    };

    if is_match(&desired_path) {
        tracing::debug!("found {}", desired_path.display());
        return Ok(desired_path);
    }

    if registry_cache.exists() {
        let dirs = std::fs::read_dir(registry_cache).context("read_dir")?;
        for dir in dirs {
            let dir = dir?;
            let file_name = dir.file_name();
            let Some(file_name) = file_name.to_str() else {
                continue;
            };

            if file_name.starts_with("index.crates.io-") {
                desired_path = dir.path().join(&crate_file);
                if is_match(&desired_path) {
                    tracing::debug!("found at {}", desired_path.display());
                    return Ok(desired_path);
                }
            }
        }
    }

    let url = format!("https://static.crates.io/crates/{}/{crate_file}", version.name);

    tracing::info!(url = %url, "fetching from crates.io");

    let output = Command::new("curl")
        .arg("-s")
        .arg("-L")
        .arg(url)
        .arg("-o")
        .arg(&desired_path)
        .output()
        .context("download")?;

    if !output.status.success() {
        anyhow::bail!("curl failed")
    }

    Ok(desired_path)
}

impl Package {
    const DEFAULT_GIT_RELEASE_BODY: &str = r#"{% if publish %}
[<img alt="crates.io" src="https://img.shields.io/badge/crates.io-v{{ version }}-orange?labelColor=5C5C5C" height="20">](https://crates.io/crates/{{ package }}/{{ version }})
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-v{{ version }}-blue?labelColor=5C5C5C" height="20">](https://docs.rs/{{ package }}/{{ version }})
{% endif %}
{{ changelog }}"#;
    const DEFAULT_GIT_TAG_NAME: &str = "{{ package }}-v{{ version }}";

    pub fn should_publish(&self) -> bool {
        self.pkg.publish.is_none()
    }

    pub fn group(&self) -> &str {
        self.metadata.group.as_deref().unwrap_or(&self.pkg.name)
    }

    pub fn changelog_path(&self) -> Option<Utf8PathBuf> {
        if self.group() == self.pkg.name && self.should_release() {
            Some(self.pkg.manifest_path.with_file_name("CHANGELOG.md"))
        } else {
            None
        }
    }

    pub fn should_git_release(&self) -> bool {
        self.metadata.git_release.unwrap_or_else(|| self.should_publish()) && self.group() == self.pkg.name
    }

    pub fn should_semver_checks(&self) -> bool {
        self.metadata.semver_checks.unwrap_or(true) && self.should_publish() && self.pkg.targets.iter().any(|t| t.is_lib())
    }

    pub fn should_min_version_check(&self) -> bool {
        self.metadata.min_versions_checks.unwrap_or(true)
            && self.should_publish()
            && self.pkg.targets.iter().any(|t| t.is_lib())
    }

    pub fn should_release(&self) -> bool {
        self.should_git_release() || self.should_publish()
    }

    pub fn last_published_version(&self) -> Option<CratesIoVersion> {
        let published_versions = self.published_versions.lock().unwrap();
        let version = published_versions.binary_search_by(|r| r.vers.cmp(&self.pkg.version));
        match version {
            Ok(idx) => Some(published_versions[idx].clone()),
            Err(idx) => idx.checked_sub(1).and_then(|idx| published_versions.get(idx).cloned()),
        }
    }

    pub fn git_tag_name(&self) -> anyhow::Result<Option<String>> {
        self.git_tag_name_version(&self.pkg.version)
    }

    fn git_tag_name_version(&self, version: &semver::Version) -> anyhow::Result<Option<String>> {
        if !self.should_git_release() {
            return Ok(None);
        }

        let tag_name = self.metadata.git_tag_name.as_deref().unwrap_or(Self::DEFAULT_GIT_TAG_NAME);

        let env = minijinja::Environment::new();
        let ctx = minijinja::context! {
            package => &self.pkg.name,
            version => version,
        };

        env.render_str(tag_name, ctx).map(Some).context("git_tag_name")
    }

    pub fn git_release_name(&self) -> anyhow::Result<Option<String>> {
        if !self.should_git_release() {
            return Ok(None);
        }

        let tag_name = self
            .metadata
            .git_release_name
            .as_deref()
            .or(self.metadata.git_tag_name.as_deref())
            .unwrap_or(Self::DEFAULT_GIT_TAG_NAME);

        let env = minijinja::Environment::new();
        let ctx = minijinja::context! {
            package => &self.pkg.name,
            version => &self.pkg.version,
        };

        env.render_str(tag_name, ctx).map(Some).context("git_release_name")
    }

    pub fn git_release_body(&self) -> anyhow::Result<Option<String>> {
        if !self.should_git_release() {
            return Ok(None);
        }

        let tag_name = self
            .metadata
            .git_release_body
            .as_deref()
            .or(self.metadata.git_tag_name.as_deref())
            .unwrap_or(Self::DEFAULT_GIT_RELEASE_BODY);

        let changelog = if let Some(path) = self.changelog_path() {
            let changelogs = std::fs::read_to_string(path).context("read changelog")?;
            changelogs
                .lines()
                .skip_while(|s| !s.starts_with("## ")) // skip to the first `## [Unreleased]`
                .skip(1) // skips the `## [Unreleased]` line
                .skip_while(|s| !s.starts_with("## ")) // skip to the first `## [{{ version }}]`
                .skip(1) // skips the `## [{{ version }}]` line
                .take_while(|s| !s.starts_with("## ")) // takes all lines until the next `## [{{ version }}]`
                .map(|s| s.trim()) // removes all whitespace
                .filter(|s| s.is_empty()) // skips empty lines
                .collect::<Vec<_>>()
                .join("\n")
        } else {
            String::new()
        };

        let env = minijinja::Environment::new();
        let ctx = minijinja::context! {
            package => &self.pkg.name,
            version => &self.pkg.version,
            publish => self.should_publish(),
            changelog => changelog,
        };

        env.render_str(tag_name, ctx).map(Some).context("git_release_body")
    }

    pub fn has_branch_changes(&self, base: &str) -> bool {
        let output = match Command::new("git")
            .arg("rev-list")
            .arg("-1")
            .arg(format!("{base}..HEAD"))
            .arg("--")
            .arg(self.pkg.manifest_path.parent().unwrap())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .output()
        {
            Ok(output) => output,
            Err(err) => {
                tracing::warn!("git rev-list failed: {err}");
                return true;
            }
        };

        if !output.status.success() {
            tracing::warn!("git rev-list failed: {}", String::from_utf8_lossy(&output.stderr));
            return true;
        }

        let commit = String::from_utf8_lossy(&output.stdout);
        !commit.trim().is_empty()
    }

    pub fn last_git_commit(&self) -> anyhow::Result<Option<String>> {
        let last_commit = if self.should_publish() {
            let Some(last_published) = self.last_published_version() else {
                return Ok(None);
            };

            // It only makes sense to check git diffs if we are currently on the latest published version.
            if last_published.vers != self.pkg.version {
                return Ok(None);
            }

            let crate_path = download_crate(&last_published)?;
            let tar_output = Command::new("tar")
                .arg("-xOzf")
                .arg(crate_path)
                .arg(format!(
                    "{}-{}/.cargo_vcs_info.json",
                    last_published.name, last_published.vers
                ))
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .output()
                .context("tar get cargo vcs info")?;

            if !tar_output.status.success() {
                anyhow::bail!("tar extact of crate failed: {}", String::from_utf8_lossy(&tar_output.stderr))
            }

            #[derive(serde::Deserialize)]
            struct VscInfo {
                git: VscInfoGit,
            }

            #[derive(serde::Deserialize)]
            struct VscInfoGit {
                sha1: String,
            }

            let vsc_info: VscInfo = serde_json::from_slice(&tar_output.stdout).context("invalid vcs info")?;
            vsc_info.git.sha1
        } else if self.should_release() {
            // check if a tag exists.
            let Some(tag_name) = self.git_tag_name()? else {
                return Ok(None);
            };

            let output = Command::new("git")
                .arg("rev-parse")
                .arg(format!("refs/tags/{tag_name}"))
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .output()
                .context("git rev-parse for tag")?;

            // tag doesnt exist
            if !output.status.success() {
                return Ok(None);
            }

            String::from_utf8_lossy(&output.stdout).trim().to_owned()
        } else {
            return Ok(None);
        };

        // git rev-list -1 HEAD~100..HEAD -- README.md
        let output = Command::new("git")
            .arg("rev-list")
            .arg("-1")
            .arg(format!("{last_commit}..HEAD"))
            .arg("--")
            .arg(self.pkg.manifest_path.parent().unwrap())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .output()
            .context("git rev-list lookup diffs")?;

        if !output.status.success() {
            anyhow::bail!("git rev-list failed: {}", String::from_utf8_lossy(&output.stderr))
        }

        let commit = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if commit.is_empty() { Ok(None) } else { Ok(Some(commit)) }
    }

    pub fn next_version(&self) -> Option<semver::Version> {
        let data = self.data.lock().unwrap();
        data.next_version.clone().or(
            if data.version_bump == VersionBump::None || self.version_bump_style().is_manual() {
                None
            } else {
                Some(data.version_bump.next_semver(self.pkg.version.clone()))
            },
        )
    }

    pub fn set_next_version(&self, version: semver::Version) {
        self.data.lock().unwrap().next_version = Some(version);
    }

    pub fn report_change(&self) {
        self.data.lock().unwrap().version_bump.bump_minor();
    }

    pub fn report_breaking_change(&self) {
        self.data.lock().unwrap().version_bump.bump_major();
    }

    pub fn has_errors(&self) -> bool {
        !self.data.lock().unwrap().issues.is_empty()
    }

    pub fn version_bump_style(&self) -> VersionBumpStyle {
        self.metadata.style.unwrap_or(if self.should_publish() {
            VersionBumpStyle::OnChange
        } else {
            VersionBumpStyle::Manual
        })
    }

    pub fn new(pkg: cargo_metadata::Package) -> anyhow::Result<Self> {
        Ok(Self {
            data: Default::default(),
            metadata: XTaskPackageMetadata::from_package(&pkg)?,
            published_versions: Default::default(),
            pkg,
        })
    }

    pub fn published_versions(&self) -> Vec<CratesIoVersion> {
        self.published_versions.lock().unwrap().clone()
    }

    pub fn fetch_published(&self) -> anyhow::Result<()> {
        if self.should_publish() {
            *self.published_versions.lock().unwrap() = crates_io_versions(&self.pkg.name)?;
        }
        Ok(())
    }

    pub fn report_issue(&self, issue: impl Into<PackageError>) {
        let issue = issue.into();
        tracing::warn!("{}", issue.to_string().replace("`", ""));
        self.data.lock().unwrap().issues.push(issue);
    }

    pub fn set_semver_output(&self, breaking: bool, output: String) {
        if breaking {
            self.report_breaking_change();
        }
        self.data.lock().unwrap().semver_output = Some((breaking, output));
    }

    pub fn set_min_versions_output(&self, output: String) {
        self.data.lock().unwrap().min_versions_output = Some(output);
    }

    pub fn semver_output(&self) -> Option<(bool, String)> {
        self.data.lock().unwrap().semver_output.clone()
    }

    pub fn min_versions_output(&self) -> Option<String> {
        self.data.lock().unwrap().min_versions_output.clone()
    }

    pub fn errors(&self) -> Vec<PackageError> {
        self.data.lock().unwrap().issues.clone()
    }
}

pub fn relative_to(path: &Utf8Path, dir: &Utf8Path) -> Utf8PathBuf {
    // If the path is already relative, just return it as is
    if path.is_relative() {
        return path.to_owned();
    }

    // Attempt to strip the prefix
    if let Ok(stripped) = path.strip_prefix(dir) {
        return stripped.to_owned();
    }

    // Fall back to manual computation like pathdiff does
    let mut result = Utf8PathBuf::new();

    let mut dir_iter = dir.components();
    let mut path_iter = path.components();

    // Skip common prefix components
    while let (Some(d), Some(p)) = (dir_iter.clone().next(), path_iter.clone().next()) {
        if d == p {
            dir_iter.next();
            path_iter.next();
        } else {
            break;
        }
    }

    // For remaining components in dir, add ".."
    for _ in dir_iter {
        result.push("..");
    }

    // Add remaining components from path
    for p in path_iter {
        result.push(p);
    }

    result
}

struct Semaphore {
    count: Mutex<usize>,
    cvar: Condvar,
}

impl Semaphore {
    fn new(initial: usize) -> Self {
        Self {
            count: Mutex::new(if initial == 0 { usize::MAX } else { initial }),
            cvar: Condvar::new(),
        }
    }

    fn acquire(&self) {
        let count = self.count.lock().unwrap();
        let mut count = self.cvar.wait_while(count, |count| *count == 0).unwrap();
        *count -= 1;
    }

    fn release(&self) {
        let mut count = self.count.lock().unwrap();
        *count += 1;
        self.cvar.notify_one();
    }
}

pub fn concurrently<F: FnOnce() -> T + Send, T: Send, C: FromIterator<T>>(
    concurrency: usize,
    items: impl IntoIterator<Item = F>,
) -> C {
    let sem = Semaphore::new(concurrency);
    std::thread::scope(|s| {
        let items = items
            .into_iter()
            .map(|i| {
                s.spawn(|| {
                    sem.acquire();
                    let r = i();
                    sem.release();
                    r
                })
            })
            .collect::<Vec<_>>();
        C::from_iter(items.into_iter().map(|item| item.join().unwrap()))
    })
}

pub fn git_workdir_clean() -> anyhow::Result<()> {
    const ERROR_MESSAGE: &str = "git working directory is dirty, please commit your changes or run with --allow-dirty";
    anyhow::ensure!(
        Command::new("git")
            .arg("diff")
            .arg("--exit-code")
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .output()
            .context("git diff")?
            .status
            .success(),
        ERROR_MESSAGE,
    );

    anyhow::ensure!(
        Command::new("git")
            .arg("diff")
            .arg("--staged")
            .arg("--exit-code")
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .output()
            .context("git diff")?
            .status
            .success(),
        ERROR_MESSAGE,
    );

    Ok(())
}

pub struct WriteUndo {
    og: Vec<u8>,
    path: Utf8PathBuf,
}

impl WriteUndo {
    pub fn new(path: &Utf8Path, content: &[u8], og: Vec<u8>) -> anyhow::Result<Self> {
        std::fs::write(path, content).context("write")?;
        Ok(Self {
            og,
            path: path.to_path_buf(),
        })
    }
}

impl Drop for WriteUndo {
    fn drop(&mut self) {
        if let Err(err) = std::fs::write(&self.path, &self.og) {
            tracing::error!(path = %self.path, "failed to undo write: {err}");
        }
    }
}

#[derive(serde_derive::Deserialize, Debug)]
#[serde(default, rename_all = "kebab-case")]
pub struct XTaskWorkspaceMetadata {
    pub pr_title: String,
    pub pr_branch: String,
    pub pr_labels: Vec<String>,
}

impl Default for XTaskWorkspaceMetadata {
    fn default() -> Self {
        Self {
            pr_title: "Cargo release".into(),
            pr_branch: "cargo-release".into(),
            pr_labels: Vec::new(),
        }
    }
}

impl XTaskWorkspaceMetadata {
    pub fn from_metadata(workspace: &cargo_metadata::Metadata) -> anyhow::Result<Self> {
        let Some(metadata) = workspace.workspace_metadata.get("xtask").and_then(|v| v.get("release")) else {
            return Ok(Self::default());
        };

        serde_json::from_value(metadata.clone()).context("xtask")
    }
}

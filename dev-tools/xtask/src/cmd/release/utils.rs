use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};

use anyhow::Context;
use cargo_metadata::camino::Utf8PathBuf;
use cargo_metadata::{Dependency, DependencyKind, semver};
use cargo_platform::Platform;
use sha2::Digest;

use crate::utils::Command;

#[derive(Clone)]
pub struct Package {
    pkg: cargo_metadata::Package,
    version_slated: Option<semver::Version>,
    published_versions: Arc<Mutex<Vec<CratesIoVersion>>>,
    data: Arc<Mutex<PackageData>>,
    metadata: XTaskPackageMetadata,
}

impl std::ops::Deref for Package {
    type Target = cargo_metadata::Package;

    fn deref(&self) -> &Self::Target {
        &self.pkg
    }
}

#[derive(serde_derive::Deserialize, Default, Debug, Clone)]
#[serde(default, rename_all = "kebab-case")]
struct GitReleaseMeta {
    name: Option<String>,
    tag_name: Option<String>,
    enabled: Option<bool>,
    body: Option<String>,
    artifacts: Vec<GitReleaseArtifact>,
}

#[derive(serde_derive::Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum GitReleaseArtifact {
    File { path: String, name: Option<String> },
}

#[derive(serde_derive::Deserialize, Default, Debug, Clone)]
#[serde(default, rename_all = "kebab-case")]
struct XTaskPackageMetadata {
    group: Option<String>,
    git_release: GitReleaseMeta,
    semver_checks: Option<bool>,
    min_versions_checks: Option<bool>,
    private_dependencies: HashSet<String>,
}

impl XTaskPackageMetadata {
    fn from_package(package: &cargo_metadata::Package) -> anyhow::Result<Self> {
        let Some(metadata) = package.metadata.get("xtask").and_then(|v| v.get("release")) else {
            return Ok(Self::default());
        };

        serde_json::from_value(metadata.clone()).context("xtask")
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum VersionBump {
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

    pub fn next_semver(&self, version: semver::Version) -> semver::Version {
        match self {
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
    GitRelease {
        error: String,
    },
    GitReleaseArtifactFileMissing {
        path: String,
    },
    VersionChanged {
        from: semver::Version,
        to: semver::Version,
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

    pub fn version_changed(from: semver::Version, to: semver::Version) -> Self {
        Self::VersionChanged { from, to }
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
            Self::GitRelease { error } => {
                write!(f, "error generating git release: {error}")
            }
            Self::GitReleaseArtifactFileMissing { path } => {
                write!(f, "missing file artifact used by git release: {path}")
            }
            Self::VersionChanged { from, to } => write!(f, "package version has changed `{from}` -> `{to}`"),
        }
    }
}

#[derive(Default)]
pub struct PackageData {
    version_bump: Option<VersionBump>,
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

#[derive(Debug, Clone)]
pub struct GitRelease {
    pub name: String,
    pub tag_name: String,
    pub body: String,
    pub artifacts: Vec<GitReleaseArtifact>,
}

impl Package {
    const DEFAULT_GIT_RELEASE_BODY: &str = include_str!("./git_release_body_tmpl.md");
    const DEFAULT_GIT_TAG_NAME: &str = "{{ package }}-v{{ version }}";

    pub fn new(workspace_meta: &WorkspaceReleaseMetadata, pkg: cargo_metadata::Package) -> anyhow::Result<Self> {
        Ok(Self {
            data: Default::default(),
            version_slated: workspace_meta.packages.get(pkg.name.as_str()).cloned(),
            metadata: XTaskPackageMetadata::from_package(&pkg)?,
            published_versions: Default::default(),
            pkg,
        })
    }

    pub fn should_publish(&self) -> bool {
        self.pkg.publish.is_none()
    }

    pub fn group(&self) -> &str {
        self.metadata.group.as_deref().unwrap_or(&self.pkg.name)
    }

    pub fn is_dep_public(&self, name: &str) -> bool {
        !self.metadata.private_dependencies.contains(name)
    }

    pub fn unreleased_req(&self) -> semver::VersionReq {
        semver::VersionReq {
            comparators: vec![semver::Comparator {
                op: semver::Op::GreaterEq,
                major: self.version.major,
                minor: Some(self.version.minor),
                patch: Some(self.version.patch),
                pre: self.version.pre.clone(),
            }],
        }
    }

    pub fn changelog_path(&self) -> Option<Utf8PathBuf> {
        if self.group() == self.pkg.name.as_ref() && self.should_release() {
            Some(self.pkg.manifest_path.with_file_name("CHANGELOG.md"))
        } else {
            None
        }
    }

    pub fn should_git_release(&self) -> bool {
        self.metadata.git_release.enabled.unwrap_or_else(|| self.should_publish()) && self.group() == self.pkg.name.as_ref()
    }

    pub fn git_release(&self) -> anyhow::Result<Option<GitRelease>> {
        if !self.should_git_release() {
            return Ok(None);
        }

        Ok(Some(GitRelease {
            body: self.git_release_body().context("body")?,
            name: self.git_release_name().context("name")?,
            tag_name: self.git_tag_name().context("tag")?,
            artifacts: self.metadata.git_release.artifacts.clone(),
        }))
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

    pub fn slated_for_release(&self) -> bool {
        self.should_release()
            && self.version_slated.as_ref().is_some_and(|v| v == &self.version)
            && self.last_published_version().is_none_or(|p| p.vers != self.version)
    }

    fn git_tag_name(&self) -> anyhow::Result<String> {
        self.git_tag_name_version(&self.pkg.version)
    }

    fn git_tag_name_version(&self, version: &semver::Version) -> anyhow::Result<String> {
        let tag_name = self
            .metadata
            .git_release
            .tag_name
            .as_deref()
            .unwrap_or(Self::DEFAULT_GIT_TAG_NAME);

        let env = minijinja::Environment::new();
        let ctx = minijinja::context! {
            package => &self.pkg.name,
            version => version,
        };

        env.render_str(tag_name, ctx).context("render")
    }

    fn git_release_name(&self) -> anyhow::Result<String> {
        let tag_name = self
            .metadata
            .git_release
            .name
            .as_deref()
            .or(self.metadata.git_release.tag_name.as_deref())
            .unwrap_or(Self::DEFAULT_GIT_TAG_NAME);

        let env = minijinja::Environment::new();
        let ctx = minijinja::context! {
            package => &self.pkg.name,
            version => &self.pkg.version,
        };

        env.render_str(tag_name, ctx).context("render")
    }

    fn git_release_body(&self) -> anyhow::Result<String> {
        let tag_name = self
            .metadata
            .git_release
            .body
            .as_deref()
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
                .skip_while(|s| s.is_empty())
                .map(|s| s.trim()) // removes all whitespace
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

        env.render_str(tag_name, ctx).context("render")
    }

    pub fn has_branch_changes(&self, base: &str) -> bool {
        let output = match Command::new("git")
            .arg("diff")
            .arg(format!("{base}..HEAD"))
            .arg("--quiet")
            .arg("--")
            .arg(self.pkg.manifest_path.parent().unwrap())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .output()
        {
            Ok(output) => output,
            Err(err) => {
                tracing::warn!("git diff failed: {err}");
                return true;
            }
        };

        if !output.status.success() && !output.stderr.is_empty() {
            tracing::warn!("git diff failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        !output.status.success()
    }

    pub fn has_changed_since_publish(&self) -> anyhow::Result<bool> {
        let last_commit = if self.should_publish() {
            let Some(last_published) = self.last_published_version() else {
                return Ok(false);
            };

            // It only makes sense to check git diffs if we are currently on the latest published version.
            if last_published.vers != self.pkg.version {
                return Ok(false);
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
            let tag_name = self.git_tag_name().context("tag name")?;

            let output = Command::new("git")
                .arg("rev-parse")
                .arg(format!("refs/tags/{tag_name}"))
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .output()
                .context("git rev-parse for tag")?;

            // tag doesnt exist
            if !output.status.success() {
                return Ok(false);
            }

            String::from_utf8_lossy(&output.stdout).trim().to_owned()
        } else {
            return Ok(false);
        };

        // git diff HEAD~100..HEAD --quiet -- README.md
        let output = Command::new("git")
            .arg("diff")
            .arg(format!("{last_commit}..HEAD"))
            .arg("--quiet")
            .arg("--")
            .arg(self.pkg.manifest_path.parent().unwrap())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .output()
            .context("git diff")?;

        if !output.status.success() && !output.stderr.is_empty() {
            anyhow::bail!("failed to get git diff {}", String::from_utf8_lossy(&output.stderr))
        }

        Ok(!output.status.success())
    }

    pub fn next_version(&self) -> Option<semver::Version> {
        self.data.lock().unwrap().next_version.clone()
    }

    pub fn set_next_version(&self, version: semver::Version) {
        if self.version != version {
            self.data.lock().unwrap().next_version = Some(version);
        }
    }

    pub fn report_change(&self) {
        self.data
            .lock()
            .unwrap()
            .version_bump
            .get_or_insert(VersionBump::Minor)
            .bump_minor();
    }

    pub fn report_breaking_change(&self) {
        self.data
            .lock()
            .unwrap()
            .version_bump
            .get_or_insert(VersionBump::Major)
            .bump_major();
    }

    pub fn version_bump(&self) -> Option<VersionBump> {
        self.data.lock().unwrap().version_bump
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

#[derive(serde_derive::Deserialize, serde_derive::Serialize, Default)]
pub struct WorkspaceReleaseMetadata {
    pub packages: BTreeMap<String, semver::Version>,
}

impl WorkspaceReleaseMetadata {
    pub fn from_metadadata(metadata: &cargo_metadata::Metadata) -> anyhow::Result<Self> {
        let Some(value) = metadata.workspace_metadata.get("xtask").and_then(|x| x.get("release")) else {
            return Ok(Default::default());
        };

        serde_json::from_value(value.clone()).context("deserialize")
    }
}

pub fn vers_to_comp(vers: semver::Version) -> semver::Comparator {
    semver::Comparator {
        op: semver::Op::Caret,
        major: vers.major,
        minor: Some(vers.minor),
        patch: Some(vers.patch),
        pre: vers.pre,
    }
}

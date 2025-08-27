//! Library for generating rust_project.json files from a `Vec<CrateSpec>`
//! See official documentation of file format at <https://rust-analyzer.github.io/manual.html>

use core::fmt;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::str::FromStr;

use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};

use crate::query::{CrateSpec, CrateType};
use crate::{ToolchainInfo, buildfile_to_targets, source_file_to_buildfile};

/// The argument that `rust-analyzer` can pass to the workspace discovery command.
#[derive(Clone, Debug, serde_derive::Deserialize, serde_derive::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RustAnalyzerArg {
    Path(Utf8PathBuf),
    Buildfile(Utf8PathBuf),
}

impl RustAnalyzerArg {
    /// Consumes itself to return a build file and the targets to build.
    pub fn into_target_details(self, workspace: &Utf8Path) -> anyhow::Result<(Utf8PathBuf, String)> {
        match self {
            Self::Path(file) => {
                let buildfile = source_file_to_buildfile(&file)?;
                buildfile_to_targets(workspace, &buildfile).map(|t| (buildfile, t))
            }
            Self::Buildfile(buildfile) => buildfile_to_targets(workspace, &buildfile).map(|t| (buildfile, t)),
        }
    }
}

impl FromStr for RustAnalyzerArg {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s).context("rust analyzer argument error")
    }
}

/// The format that `rust_analyzer` expects as a response when automatically invoked.
/// See [rust-analyzer documentation][rd] for a thorough description of this interface.
///
/// [rd]: <https://rust-analyzer.github.io/manual.html#rust-analyzer.workspace.discoverConfig>
#[derive(Debug, serde_derive::Serialize)]
#[serde(tag = "kind")]
#[serde(rename_all = "snake_case")]
pub enum DiscoverProject<'a> {
    Finished {
        buildfile: Utf8PathBuf,
        project: RustProject,
    },
    Error {
        error: String,
        source: Option<String>,
    },
    Progress {
        message: &'a fmt::Arguments<'a>,
    },
}

/// A `rust-project.json` workspace representation. See
/// [rust-analyzer documentation][rd] for a thorough description of this interface.
///
/// [rd]: <https://rust-analyzer.github.io/manual.html#non-cargo-based-projects>
#[derive(Debug, serde_derive::Serialize)]
pub struct RustProject {
    /// The path to a Rust sysroot.
    sysroot: Utf8PathBuf,

    /// Path to the directory with *source code* of
    /// sysroot crates.
    sysroot_src: Utf8PathBuf,

    /// The set of crates comprising the current
    /// project. Must include all transitive
    /// dependencies as well as sysroot crate (libstd,
    /// libcore and such).
    crates: Vec<Crate>,

    /// The set of runnables, such as tests or benchmarks,
    /// that can be found in the crate.
    runnables: Vec<Runnable>,
}

/// A `rust-project.json` crate representation. See
/// [rust-analyzer documentation][rd] for a thorough description of this interface.
///
/// [rd]: <https://rust-analyzer.github.io/manual.html#non-cargo-based-projects>
#[derive(Debug, serde_derive::Serialize)]
pub struct Crate {
    /// A name used in the package's project declaration
    #[serde(skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,

    /// Path to the root module of the crate.
    root_module: String,

    /// Edition of the crate.
    edition: String,

    /// Dependencies
    deps: Vec<Dependency>,

    /// Should this crate be treated as a member of current "workspace".
    #[serde(skip_serializing_if = "Option::is_none")]
    is_workspace_member: Option<bool>,

    /// Optionally specify the (super)set of `.rs` files comprising this crate.
    #[serde(skip_serializing_if = "Source::is_empty")]
    source: Source,

    /// The set of cfgs activated for a given crate, like
    /// `["unix", "feature=\"foo\"", "feature=\"bar\""]`.
    cfg: BTreeSet<String>,

    /// Target triple for this Crate.
    #[serde(skip_serializing_if = "Option::is_none")]
    target: Option<String>,

    /// Environment variables, used for the `env!` macro
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<BTreeMap<String, String>>,

    /// Whether the crate is a proc-macro crate.
    is_proc_macro: bool,

    /// For proc-macro crates, path to compiled proc-macro (.so file).
    #[serde(skip_serializing_if = "Option::is_none")]
    proc_macro_dylib_path: Option<String>,

    /// Build information for the crate
    #[serde(skip_serializing_if = "Option::is_none")]
    build: Option<Build>,
}

#[derive(Debug, Default, serde_derive::Serialize)]
pub struct Source {
    include_dirs: Vec<String>,
    exclude_dirs: Vec<String>,
}

impl Source {
    fn is_empty(&self) -> bool {
        self.include_dirs.is_empty() && self.exclude_dirs.is_empty()
    }
}

#[derive(Debug, serde_derive::Serialize)]
pub struct Dependency {
    /// Index of a crate in the `crates` array.
    #[serde(rename = "crate")]
    crate_index: usize,

    /// The display name of the crate.
    name: String,
}

#[derive(Debug, serde_derive::Serialize)]
pub struct Build {
    /// The name associated with this crate.
    ///
    /// This is determined by the build system that produced
    /// the `rust-project.json` in question. For instance, if bazel were used,
    /// the label might be something like `//ide/rust/rust-analyzer:rust-analyzer`.
    ///
    /// Do not attempt to parse the contents of this string; it is a build system-specific
    /// identifier similar to [`Crate::display_name`].
    label: String,
    /// Path corresponding to the build system-specific file defining the crate.
    build_file: Utf8PathBuf,
    /// The kind of target.
    ///
    /// Examples (non-exhaustively) include [`TargetKind::Bin`], [`TargetKind::Lib`],
    /// and [`TargetKind::Test`]. This information is used to determine what sort
    /// of runnable codelens to provide, if any.
    target_kind: TargetKind,
    runnables: Vec<Runnable>,
}

#[derive(Clone, Copy, Debug, PartialEq, serde_derive::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TargetKind {
    Bin,
    /// Any kind of Cargo lib crate-type (dylib, rlib, proc-macro, ...).
    Lib,
    Test,
}

/// A template-like structure for describing runnables.
///
/// These are used for running and debugging binaries and tests without encoding
/// build system-specific knowledge into rust-analyzer.
///
/// # Example
///
/// Below is an example of a test runnable. `{label}` and `{test_id}`
/// are explained in [`Runnable::args`]'s documentation.
///
/// ```json
/// {
///     "program": "bazel",
///     "args": [
///         "test",
///          "{label}",
///          "--test_arg",
///          "{test_id}",
///     ],
///     "cwd": "/home/user/repo-root/",
///     "kind": "testOne"
/// }
/// ```
#[derive(Debug, serde_derive::Serialize)]
pub struct Runnable {
    /// The program invoked by the runnable.
    ///
    /// For example, this might be `cargo`, `bazel`, etc.
    program: String,
    /// The arguments passed to [`Runnable::program`].
    ///
    /// The args can contain two template strings: `{label}` and `{test_id}`.
    /// rust-analyzer will find and replace `{label}` with [`Build::label`] and
    /// `{test_id}` with the test name.
    args: Vec<String>,
    /// The current working directory of the runnable.
    cwd: Utf8PathBuf,
    kind: RunnableKind,
}

/// The kind of runnable.
#[derive(Debug, Clone, Copy, serde_derive::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RunnableKind {
    /// Output rustc diagnostics
    Check,

    /// Can run a binary.
    Run,

    /// Run a single test.
    TestOne,

    /// Runs a module of tests
    TestMod,

    /// Runs a doc test
    DocTest,
}

pub fn assemble_rust_project(
    bazel: &Utf8Path,
    workspace: &Utf8Path,
    output_base: &Utf8Path,
    toolchain_info: ToolchainInfo,
    crate_specs: impl IntoIterator<Item = CrateSpec>,
) -> anyhow::Result<RustProject> {
    let mut project = RustProject {
        sysroot: toolchain_info.sysroot,
        sysroot_src: toolchain_info.sysroot_src,
        crates: Vec::new(),
        runnables: vec![],
    };

    let mut all_crates: Vec<_> = crate_specs.into_iter().collect();

    all_crates.sort_by_key(|a| !a.is_test);

    let merged_crates_index: HashMap<_, _> = all_crates.iter().enumerate().map(|(idx, c)| (&c.crate_id, idx)).collect();

    for c in all_crates.iter() {
        log::trace!("Merging crate {}", &c.crate_id);

        let target_kind = match c.crate_type {
            CrateType::Bin if c.is_test => TargetKind::Test,
            CrateType::Bin => TargetKind::Bin,
            CrateType::Rlib
            | CrateType::Lib
            | CrateType::Dylib
            | CrateType::Cdylib
            | CrateType::Staticlib
            | CrateType::ProcMacro => TargetKind::Lib,
        };

        let mut runnables = Vec::new();

        if let Some(info) = &c.info {
            if let Some(crate_label) = &info.crate_label
                && matches!(target_kind, TargetKind::Bin)
            {
                runnables.push(Runnable {
                    program: bazel.to_string(),
                    args: vec!["run".to_string(), format!("//{crate_label}")],
                    cwd: workspace.to_owned(),
                    kind: RunnableKind::Run,
                });
            }

            if let Some(test_label) = &info.test_label {
                runnables.extend([
                    Runnable {
                        program: bazel.to_string(),
                        args: vec![
                            format!("--output_base={output_base}"),
                            "test".to_owned(),
                            format!("//{test_label}"),
                            "--test_output".to_owned(),
                            "streamed".to_owned(),
                            "--test_arg".to_owned(),
                            "--exact".to_owned(),
                            "--test_arg".to_owned(),
                            "{test_id}".to_owned(),
                        ],
                        cwd: workspace.to_owned(),
                        kind: RunnableKind::TestOne,
                    },
                    Runnable {
                        program: bazel.to_string(),
                        args: vec![
                            format!("--output_base={output_base}"),
                            "test".to_owned(),
                            format!("//{test_label}"),
                            "--test_output".to_owned(),
                            "streamed".to_owned(),
                            "--test_arg".to_owned(),
                            "{path}".to_owned(),
                        ],
                        cwd: workspace.to_owned(),
                        kind: RunnableKind::TestMod,
                    },
                ]);
            }

            if let Some(doc_test_label) = &info.doc_test_label {
                runnables.push(Runnable {
                    program: bazel.to_string(),
                    args: vec![
                        format!("--output_base={output_base}"),
                        "test".to_owned(),
                        format!("//{doc_test_label}"),
                        "--test_output".to_owned(),
                        "streamed".to_owned(),
                        "--test_arg".to_owned(),
                        "{test_id}".to_owned(),
                    ],
                    cwd: workspace.to_owned(),
                    kind: RunnableKind::DocTest,
                });
            }

            if let Some(clippy_label) = &info.clippy_label {
                runnables.push(Runnable {
                    program: bazel.to_string(),
                    args: vec![
                        format!("--output_base={output_base}"),
                        "run".to_owned(),
                        "--config=wrapper".to_owned(),
                        "//misc/utils/rust/analyzer/check".to_owned(),
                        "--".to_owned(),
                        "--config=wrapper".to_owned(),
                        format!("//{clippy_label}"),
                    ],
                    cwd: workspace.to_owned(),
                    kind: RunnableKind::Check,
                });
            }
        }

        project.crates.push(Crate {
            display_name: Some(c.display_name.clone()),
            root_module: c.root_module.clone(),
            edition: c.edition.clone(),
            deps: c
                .deps
                .iter()
                .map(|dep| {
                    let crate_index = *merged_crates_index
                        .get(dep)
                        .expect("failed to find dependency on second lookup");
                    let dep_crate = &all_crates[crate_index];
                    let name = if let Some(alias) = c.aliases.get(dep) {
                        alias.clone()
                    } else {
                        dep_crate.display_name.clone()
                    };
                    Dependency { crate_index, name }
                })
                .collect(),
            is_workspace_member: Some(c.is_workspace_member),
            source: match &c.source {
                Some(s) => Source {
                    exclude_dirs: s.exclude_dirs.clone(),
                    include_dirs: s.include_dirs.clone(),
                },
                None => Source::default(),
            },
            cfg: c.cfg.clone(),
            target: Some(c.target.clone()),
            env: Some(c.env.clone()),
            is_proc_macro: c.proc_macro_dylib_path.is_some(),
            proc_macro_dylib_path: c.proc_macro_dylib_path.clone(),
            build: c.build.as_ref().map(|b| Build {
                label: b.label.clone(),
                build_file: b.build_file.clone().into(),
                target_kind,
                runnables,
            }),
        });
    }

    Ok(project)
}

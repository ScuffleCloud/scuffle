//! A crate which allows you to compile Rust code at runtime (hence the name
//! `postcompile`).
//!
//! What that means is that you can provide the input to `rustc` and then get
//! back the expanded output, compiler errors, warnings, etc.
//!
//! This is particularly useful when making snapshot tests of proc-macros, look
//! below for an example with the `insta` crate.
#![cfg_attr(feature = "docs", doc = "\n\nSee the [changelog][changelog] for a full release history.")]
#![cfg_attr(feature = "docs", doc = "## Feature flags")]
#![cfg_attr(feature = "docs", doc = document_features::document_features!())]
//! ## Usage
//!
//! ```rust,standalone_crate,test_harness
//! # macro_rules! assert_snapshot {
//! #     ($expr:expr) => { $expr };
//! # }
//! #[test]
//! fn some_cool_test() {
//!     assert_snapshot!(postcompile::compile!({
//!         #![allow(unused)]
//!
//!         #[derive(Debug, Clone)]
//!         struct Test {
//!             a: u32,
//!             b: i32,
//!         }
//!
//!         const TEST: Test = Test { a: 1, b: 3 };
//!     }));
//! }
//!
//! #[test]
//! fn some_cool_test_extern() {
//!     assert_snapshot!(postcompile::compile_str!(include_str!("some_file.rs")));
//! }
//!
//! #[test]
//! fn test_inside_test() {
//!     assert_snapshot!(postcompile::compile!(
//!         postcompile::config! {
//!             test: true,
//!         },
//!         {
//!             fn add(a: i32, b: i32) -> i32 {
//!                 a + b
//!             }
//!
//!             #[test]
//!             fn test_add() {
//!                 assert_eq!(add(1, 2), 3);
//!             }
//!         },
//!     ));
//! }
//!
//! #[test]
//! fn test_inside_test_with_tokio() {
//!     assert_snapshot!(postcompile::compile!(
//!         postcompile::config! {
//!             test: true,
//!             dependencies: vec![
//!                 postcompile::Dependency::version("tokio", "1").feature("full")
//!             ]
//!         },
//!         {
//!             async fn async_add(a: i32, b: i32) -> i32 {
//!                 a + b
//!             }
//!
//!             #[tokio::test]
//!             async fn test_add() {
//!                 assert_eq!(async_add(1, 2).await, 3);
//!             }
//!         },
//!     ));
//! }
//! ```
//!
//! ## Features
//!
//! - Cached builds: This crate reuses the cargo build cache of the original
//!   crate so that only the contents of the macro are compiled & not any
//!   additional dependencies.
//! - Coverage: This crate works with [`cargo-llvm-cov`](https://crates.io/crates/cargo-llvm-cov)
//!   out of the box, which allows you to instrument the proc-macro expansion.
//! - Testing: You can define tests with the `#[test]` macro and the tests will run on the generated code.
//!
//! ## Alternatives
//!
//! - [`compiletest_rs`](https://crates.io/crates/compiletest_rs): This crate is
//!   used by the Rust compiler team to test the compiler itself. Not really
//!   useful for proc-macros.
//! - [`trybuild`](https://crates.io/crates/trybuild): This crate is an
//!   all-in-one solution for testing proc-macros, with built in snapshot
//!   testing.
//! - [`ui_test`](https://crates.io/crates/ui_test): Similar to `trybuild` with
//!   a slightly different API & used by the Rust compiler team to test the
//!   compiler itself.
//!
//! ### Differences
//!
//! The other libraries are focused on testing & have built in test harnesses.
//! This crate takes a step back and allows you to compile without a testing
//! harness. This has the advantage of being more flexible, and allows you to
//! use whatever testing framework you want.
//!
//! In the examples above I showcase how to use this crate with the `insta`
//! crate for snapshot testing.
//!
//! ## Limitations
//!
//! Please note that this crate does not work inside a running compiler process
//! (inside a proc-macro) without hacky workarounds and complete build-cache
//! invalidation.
//!
//! This is because `cargo` holds a lock on the build directory and that if we
//! were to compile inside a proc-macro we would recursively invoke the
//! compiler.
//!
//! ## License
//!
//! This project is licensed under the MIT or Apache-2.0 license.
//! You can choose between one of them if you use this work.
//!
//! `SPDX-License-Identifier: MIT OR Apache-2.0`
#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(missing_docs)]
#![deny(unsafe_code)]
#![deny(unreachable_pub)]

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::Path;
use std::process::Command;

use cargo_manifest::DependencyDetail;

#[derive(serde_derive::Deserialize)]
struct DepsManifest {
    direct: BTreeMap<String, String>,
    search: BTreeSet<String>,
    extra_rustc_args: Vec<String>,
}

/// The return status of the compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitStatus {
    /// If the compiler returned a 0 exit code.
    Success,
    /// If the compiler returned a non-0 exit code.
    Failure(i32),
}

impl std::fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExitStatus::Success => write!(f, "0"),
            ExitStatus::Failure(code) => write!(f, "{code}"),
        }
    }
}

/// The output of the compilation.
#[derive(Debug)]
pub struct CompileOutput {
    /// The status of the compilation.
    pub status: ExitStatus,
    /// The stdout of the compilation.
    /// This will contain the expanded code.
    pub expanded: String,
    /// The stderr of the compilation.
    /// This will contain any errors or warnings from the compiler.
    pub expand_stderr: String,
    /// The stderr of the compilation.
    /// This will contain any errors or warnings from the compiler.
    pub test_stderr: String,
    /// The stdout of the test results.
    pub test_stdout: String,
}

impl std::fmt::Display for CompileOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "exit status: {}", self.status)?;
        if !self.expand_stderr.is_empty() {
            write!(f, "--- expand_stderr\n{}\n", self.expand_stderr)?;
        }
        if !self.test_stderr.is_empty() {
            write!(f, "--- test_stderr\n{}\n", self.test_stderr)?;
        }
        if !self.test_stdout.is_empty() {
            write!(f, "--- test_stdout\n{}\n", self.test_stdout)?;
        }
        if !self.expanded.is_empty() {
            write!(f, "--- expanded\n{}\n", self.expanded)?;
        }
        Ok(())
    }
}

fn cargo(config: &Config, manifest_path: &Path, subcommand: &str) -> Command {
    let mut program = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
    program.arg(subcommand);
    program.current_dir(manifest_path.parent().unwrap());

    program.env_clear();
    program.envs(std::env::vars().filter(|(k, _)| !k.starts_with("CARGO_") && k != "OUT_DIR"));
    program.env("CARGO_TERM_COLOR", "never");
    program.stderr(std::process::Stdio::piped());
    program.stdout(std::process::Stdio::piped());

    let target_dir = if config.target_dir.as_ref().unwrap().ends_with(target_triple::TARGET) {
        config.target_dir.as_ref().unwrap().parent().unwrap()
    } else {
        config.target_dir.as_ref().unwrap()
    };

    program.arg("--quiet");
    program.arg("--manifest-path").arg(manifest_path);
    program.arg("--target-dir").arg(target_dir);

    if !cfg!(trybuild_no_target)
        && !cfg!(postcompile_no_target)
        && config.target_dir.as_ref().unwrap().ends_with(target_triple::TARGET)
    {
        program.arg("--target").arg(target_triple::TARGET);
    }

    program
}

fn rustc() -> Command {
    let mut program = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into()));
    program.stderr(std::process::Stdio::piped());
    program.stdout(std::process::Stdio::piped());
    program
}

fn write_tmp_file(tokens: &str, tmp_file: &Path) {
    std::fs::create_dir_all(tmp_file.parent().unwrap()).unwrap();

    let tokens = if let Ok(file) = syn::parse_file(tokens) {
        prettyplease::unparse(&file)
    } else {
        tokens.to_owned()
    };

    std::fs::write(tmp_file, tokens).unwrap();
}

fn generate_cargo_toml(config: &Config, crate_name: &str) -> std::io::Result<(String, String)> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(config.manifest.as_deref().unwrap())
        .exec()
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

    let workspace_manifest = cargo_manifest::Manifest::from_path(metadata.workspace_root.join("Cargo.toml"))
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

    let manifest = cargo_manifest::Manifest::<cargo_manifest::Value, cargo_manifest::Value> {
        package: Some(cargo_manifest::Package {
            publish: Some(cargo_manifest::MaybeInherited::Local(cargo_manifest::Publish::Flag(false))),
            edition: match config.edition.as_str() {
                "2024" => Some(cargo_manifest::MaybeInherited::Local(cargo_manifest::Edition::E2024)),
                "2021" => Some(cargo_manifest::MaybeInherited::Local(cargo_manifest::Edition::E2021)),
                "2018" => Some(cargo_manifest::MaybeInherited::Local(cargo_manifest::Edition::E2018)),
                "2015" => Some(cargo_manifest::MaybeInherited::Local(cargo_manifest::Edition::E2015)),
                _ => match metadata
                    .packages
                    .iter()
                    .find(|p| p.name.as_ref() == config.package_name)
                    .map(|p| p.edition)
                {
                    Some(cargo_metadata::Edition::E2015) => {
                        Some(cargo_manifest::MaybeInherited::Local(cargo_manifest::Edition::E2015))
                    }
                    Some(cargo_metadata::Edition::E2018) => {
                        Some(cargo_manifest::MaybeInherited::Local(cargo_manifest::Edition::E2018))
                    }
                    Some(cargo_metadata::Edition::E2021) => {
                        Some(cargo_manifest::MaybeInherited::Local(cargo_manifest::Edition::E2021))
                    }
                    Some(cargo_metadata::Edition::E2024) => {
                        Some(cargo_manifest::MaybeInherited::Local(cargo_manifest::Edition::E2024))
                    }
                    _ => None,
                },
            },
            ..cargo_manifest::Package::<cargo_manifest::Value>::new(crate_name.to_owned(), "0.1.0".into())
        }),
        workspace: Some(cargo_manifest::Workspace {
            default_members: None,
            dependencies: None,
            exclude: None,
            members: Vec::new(),
            metadata: None,
            package: None,
            resolver: None,
        }),
        dependencies: Some({
            let mut deps = BTreeMap::new();

            for dep in &config.dependencies {
                let mut detail = if dep.workspace {
                    let Some(dep) = workspace_manifest
                        .workspace
                        .as_ref()
                        .and_then(|workspace| workspace.dependencies.as_ref())
                        .or(workspace_manifest.dependencies.as_ref())
                        .and_then(|deps| deps.get(&dep.name))
                    else {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("workspace has no dep: {}", dep.name),
                        ));
                    };

                    let mut dep = match dep {
                        cargo_manifest::Dependency::Detailed(d) => d.clone(),
                        cargo_manifest::Dependency::Simple(version) => DependencyDetail {
                            version: Some(version.clone()),
                            ..Default::default()
                        },
                        cargo_manifest::Dependency::Inherited(_) => panic!("workspace deps cannot be inherited"),
                    };

                    if let Some(path) = dep.path.as_mut()
                        && std::path::Path::new(path.as_str()).is_relative()
                    {
                        *path = metadata.workspace_root.join(path.as_str()).to_string()
                    }

                    dep
                } else {
                    Default::default()
                };

                if !dep.default_features {
                    detail.features = None;
                }

                detail.default_features = Some(dep.default_features);
                if let Some(mut path) = dep.path.clone() {
                    if std::path::Path::new(path.as_str()).is_relative() {
                        path = config
                            .manifest
                            .as_ref()
                            .unwrap()
                            .parent()
                            .unwrap()
                            .join(path)
                            .to_string_lossy()
                            .to_string();
                    }
                    detail.path = Some(path);
                }
                if let Some(version) = dep.version.clone() {
                    detail.version = Some(version);
                }

                detail.features.get_or_insert_default().extend(dep.features.iter().cloned());

                deps.insert(dep.name.clone(), cargo_manifest::Dependency::Detailed(detail));
            }

            deps
        }),
        patch: workspace_manifest.patch.clone().map(|mut patch| {
            patch.values_mut().for_each(|deps| {
                deps.values_mut().for_each(|dep| {
                    if let cargo_manifest::Dependency::Detailed(dep) = dep
                        && let Some(path) = &mut dep.path
                        && std::path::Path::new(path.as_str()).is_relative()
                    {
                        *path = metadata.workspace_root.join(path.as_str()).to_string()
                    }
                });
            });

            patch
        }),
        ..Default::default()
    };

    Ok((
        toml::to_string(&manifest).map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?,
        std::fs::read_to_string(metadata.workspace_root.join("Cargo.lock"))?,
    ))
}

static TEST_TIME_RE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"\d+\.\d+s").expect("failed to compile regex"));

/// Compiles the given tokens and returns the output.
pub fn compile_custom(tokens: impl std::fmt::Display, config: &Config) -> std::io::Result<CompileOutput> {
    let tokens = tokens.to_string();
    if let Ok(deps_manifest) = std::env::var("POSTCOMPILE_DEPS_MANIFEST") {
        return manifest_mode(deps_manifest, config, tokens);
    }

    let crate_name = config.function_name.replace("::", "__");
    let tmp_crate_path = Path::new(config.tmp_dir.as_deref().unwrap()).join(&crate_name);
    std::fs::create_dir_all(&tmp_crate_path)?;

    let manifest_path = tmp_crate_path.join("Cargo.toml");
    let (cargo_toml, cargo_lock) = generate_cargo_toml(config, &crate_name)?;

    std::fs::write(&manifest_path, cargo_toml)?;
    std::fs::write(tmp_crate_path.join("Cargo.lock"), cargo_lock)?;

    let main_path = tmp_crate_path.join("src").join("main.rs");

    write_tmp_file(&tokens, &main_path);

    let mut program = cargo(config, &manifest_path, "rustc");

    // The first invoke is used to get the macro expanded code.
    // We set this env variable so that this compiler can accept nightly options.)
    program.env("RUSTC_BOOTSTRAP", "1");
    program.arg("--").arg("-Zunpretty=expanded");

    let output = program.output().unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let syn_file = syn::parse_file(&stdout);
    let stdout = syn_file.as_ref().map(prettyplease::unparse).unwrap_or(stdout);

    let cleanup_output = |out: &[u8]| {
        let out = String::from_utf8_lossy(out);
        let tmp_dir = config.tmp_dir.as_ref().unwrap().display().to_string();
        let main_relative = main_path.strip_prefix(&tmp_crate_path).unwrap().display().to_string();
        let main_path = main_path.display().to_string();
        TEST_TIME_RE
            .replace_all(out.as_ref(), "[ELAPSED]s")
            .trim()
            .replace(&main_relative, "[POST_COMPILE]")
            .replace(&main_path, "[POST_COMPILE]")
            .replace(&tmp_dir, "[BUILD_DIR]")
    };

    let mut result = CompileOutput {
        status: if output.status.success() {
            ExitStatus::Success
        } else {
            ExitStatus::Failure(output.status.code().unwrap_or(-1))
        },
        expand_stderr: cleanup_output(&output.stderr),
        expanded: stdout,
        test_stderr: String::new(),
        test_stdout: String::new(),
    };

    if result.status == ExitStatus::Success {
        let mut program = cargo(config, &manifest_path, "test");

        if !config.test {
            program.arg("--no-run");
        }

        let comp_output = program.output().unwrap();
        result.status = if comp_output.status.success() {
            ExitStatus::Success
        } else {
            ExitStatus::Failure(comp_output.status.code().unwrap_or(-1))
        };

        result.test_stderr = cleanup_output(&comp_output.stderr);
        result.test_stdout = cleanup_output(&comp_output.stdout);
    };

    Ok(result)
}

fn manifest_mode(deps_manifest_path: String, config: &Config, tokens: String) -> std::io::Result<CompileOutput> {
    let deps_manifest = match std::fs::read_to_string(&deps_manifest_path) {
        Ok(o) => o,
        Err(err) => panic!("error opening file: {deps_manifest_path} {err}"),
    };
    let manifest: DepsManifest = serde_json::from_str(&deps_manifest)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))
        .unwrap();

    let current_dir = std::env::current_dir().unwrap();

    let args: Vec<_> = manifest
        .direct
        .iter()
        .map(|(name, file)| format!("--extern={name}={file}", file = current_dir.join(file).display()))
        .chain(
            manifest
                .search
                .iter()
                .map(|search| format!("-Ldependency={search}", search = current_dir.join(search).display())),
        )
        .chain(manifest.extra_rustc_args.iter().cloned())
        .chain([
            "--crate-type=lib".into(),
            format!(
                "--edition={}",
                if config.edition.is_empty() {
                    "2024"
                } else {
                    config.edition.as_str()
                }
            ),
        ])
        .collect();

    let tmp_dir = std::env::var("TEST_TMPDIR").expect("TEST_TMPDIR must be set when using manifest mode.");
    let name = config.function_name.replace("::", "__");
    let tmp_rs_path = Path::new(&tmp_dir).join(format!("{name}.rs"));
    write_tmp_file(&tokens, &tmp_rs_path);

    let output = rustc()
        .env("RUSTC_BOOTSTRAP", "1")
        .arg("-Zunpretty=expanded")
        .args(args.iter())
        .arg(&tmp_rs_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let syn_file = syn::parse_file(&stdout);
    let stdout = syn_file.as_ref().map(prettyplease::unparse).unwrap_or(stdout);

    let cleanup_output = |out: &[u8]| {
        let out = String::from_utf8_lossy(out);
        let main_relative = tmp_rs_path.strip_prefix(&tmp_dir).unwrap().display().to_string();
        let main_path = tmp_rs_path.display().to_string();
        TEST_TIME_RE
            .replace_all(out.as_ref(), "[ELAPSED]s")
            .trim()
            .replace(&main_relative, "[POST_COMPILE]")
            .replace(&main_path, "[POST_COMPILE]")
            .replace(&tmp_dir, "[BUILD_DIR]")
    };

    let mut result = CompileOutput {
        status: if output.status.success() {
            ExitStatus::Success
        } else {
            ExitStatus::Failure(output.status.code().unwrap_or(-1))
        },
        expand_stderr: cleanup_output(&output.stderr),
        expanded: stdout,
        test_stderr: String::new(),
        test_stdout: String::new(),
    };

    if result.status == ExitStatus::Success {
        let mut program = rustc();

        program
            .arg("--test")
            .args(args.iter())
            .arg("-o")
            .arg(tmp_rs_path.with_extension("bin"))
            .arg(&tmp_rs_path);

        let mut comp_output = program.output().unwrap();
        if comp_output.status.success() && config.test {
            comp_output = Command::new(tmp_rs_path.with_extension("bin"))
                .arg("--quiet")
                .output()
                .unwrap();
        }

        result.status = if comp_output.status.success() {
            ExitStatus::Success
        } else {
            ExitStatus::Failure(comp_output.status.code().unwrap_or(-1))
        };

        result.test_stderr = cleanup_output(&comp_output.stderr);
        result.test_stdout = cleanup_output(&comp_output.stdout);
    };

    Ok(result)
}

/// The configuration for the compilation.
#[derive(Clone, Debug, Default)]
pub struct Config {
    /// The path to the cargo manifest file of the library being tested.
    /// This is so that we can include the `dependencies` & `dev-dependencies`
    /// making them available in the code provided.
    pub manifest: Option<Cow<'static, Path>>,
    /// The path to the target directory, used to cache builds & find
    /// dependencies.
    pub target_dir: Option<Cow<'static, Path>>,
    /// A temporary directory to write the expanded code to.
    pub tmp_dir: Option<Cow<'static, Path>>,
    /// The name of the function to compile.
    pub function_name: Cow<'static, str>,
    /// The path to the file being compiled.
    pub file_path: Cow<'static, Path>,
    /// The name of the package being compiled.
    pub package_name: Cow<'static, str>,
    /// The dependencies to add to the temporary crate.
    pub dependencies: Vec<Dependency>,
    /// Run any unit tests in the package.
    pub test: bool,
    /// The rust edition to use.
    pub edition: String,
}

/// A dependency to apply to the code
#[derive(Debug, Clone)]
pub struct Dependency {
    name: String,
    path: Option<String>,
    version: Option<String>,
    workspace: bool,
    features: Vec<String>,
    default_features: bool,
}

impl Dependency {
    fn new(name: String) -> Self {
        Self {
            name,
            workspace: false,
            default_features: true,
            features: Vec::new(),
            path: None,
            version: None,
        }
    }

    /// Create a dependency using the workspace dependency
    pub fn workspace(name: impl std::fmt::Display) -> Self {
        Self {
            workspace: true,
            ..Self::new(name.to_string())
        }
    }

    /// Create a dependency using a path to the crate root, relative to the root of the current package.
    pub fn path(name: impl std::fmt::Display, path: impl std::fmt::Display) -> Self {
        Self {
            path: Some(path.to_string()),
            ..Self::new(name.to_string())
        }
    }

    /// Create a dependency using a name and version from crates.io
    pub fn version(name: impl std::fmt::Display, version: impl std::fmt::Display) -> Self {
        Self {
            version: Some(version.to_string()),
            ..Self::new(name.to_string())
        }
    }

    /// Add a feature to the dependency
    pub fn feature(mut self, feature: impl std::fmt::Display) -> Self {
        self.features.push(feature.to_string());
        self
    }

    /// Toggle the default features flag
    pub fn default_features(self, default_features: bool) -> Self {
        Self {
            default_features,
            ..self
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! _function_name {
    () => {{
        fn f() {}
        fn type_name_of_val<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let mut name = type_name_of_val(f).strip_suffix("::f").unwrap_or("");
        while let Some(rest) = name.strip_suffix("::{{closure}}") {
            name = rest;
        }
        name
    }};
}

#[doc(hidden)]
pub fn build_dir() -> Option<&'static Path> {
    Some(Path::new(option_env!("OUT_DIR")?))
}

#[doc(hidden)]
pub fn target_dir() -> Option<&'static Path> {
    build_dir()?.parent()?.parent()?.parent()?.parent()
}

/// Define a config to use when compiling crates.
/// This macro is allows you to provide values for the config items.
/// ```rust
/// let config = postcompile::config! {
///     edition: "2021".into(),
///     dependencies: Vec::new()
/// };
/// ```
///
/// By default the current crate is included as the only dependency. You can undo this by
/// setting the Dependencies field to an empty vector.
///
/// By default the edition is set to whatever the current edition is set to.
#[macro_export]
macro_rules! config {
    (
        $($item:ident: $value:expr),*$(,)?
    ) => {{
        #[allow(unused_mut)]
        let mut config = $crate::Config {
            manifest: option_env!("CARGO_MANIFEST_PATH").map(|env| ::std::borrow::Cow::Borrowed(::std::path::Path::new(env))),
            tmp_dir: $crate::build_dir().map(::std::borrow::Cow::Borrowed),
            target_dir: $crate::target_dir().map(::std::borrow::Cow::Borrowed),
            function_name: ::std::borrow::Cow::Borrowed($crate::_function_name!()),
            file_path: ::std::borrow::Cow::Borrowed(::std::path::Path::new(file!())),
            package_name: ::std::borrow::Cow::Borrowed(env!("CARGO_PKG_NAME")),
            dependencies: vec![
                $crate::Dependency::path(env!("CARGO_PKG_NAME"), ".")
            ],
            ..::core::default::Default::default()
        };

        $(
            config.$item = $value;
        )*

        config
    }};
}

/// Compiles the given tokens and returns the output.
///
/// This macro will panic if we fail to invoke the compiler.
///
/// ```rust
/// // Dummy macro to assert the snapshot.
/// # macro_rules! assert_snapshot {
/// #     ($expr:expr) => { $expr };
/// # }
/// let output = postcompile::compile!({
///     const TEST: u32 = 1;
/// });
///
/// assert_eq!(output.status, postcompile::ExitStatus::Success);
/// // We dont have an assert_snapshot! macro in this crate, but you get the idea.
/// assert_snapshot!(output);
/// ```
///
/// You can provide a custom config using the [`config!`] macro. If not provided the default config is used.
///
/// In this example we enable the `test` flag which will run the tests inside the provided source code.
///
/// ```rust
/// // Dummy macro to assert the snapshot.
/// # macro_rules! assert_snapshot {
/// #     ($expr:expr) => { $expr };
/// # }
/// let output = postcompile::compile!(
///     postcompile::config! {
///         test: true
///     },
///     {
///         const TEST: u32 = 1;
///
///         #[test]
///         fn test() {
///             assert_eq!(TEST, 1);
///         }
///     }
/// );
///
/// assert_eq!(output.status, postcompile::ExitStatus::Success);
/// // We dont have an assert_snapshot! macro in this crate, but you get the idea.
/// assert_snapshot!(output);
/// ```
#[macro_export]
macro_rules! compile {
    (
        $config:expr,
        { $($tokens:tt)* }$(,)?
    ) => {
        $crate::compile_str!($config, stringify!($($tokens)*))
    };
    (
        { $($tokens:tt)* }$(,)?
    ) => {
        $crate::compile_str!(stringify!($($tokens)*))
    };
}

/// Compiles the given string of tokens and returns the output.
///
/// This macro will panic if we fail to invoke the compiler.
///
/// Same as the [`compile!`] macro, but for strings. This allows you to do:
///
/// ```rust,standalone_crate
/// let output = postcompile::compile_str!(include_str!("some_file.rs"));
///
/// // ... do something with the output
/// ```
#[macro_export]
macro_rules! compile_str {
    ($config:expr, $expr:expr $(,)?) => {
        $crate::try_compile_str!($config, $expr).expect("failed to compile")
    };
    ($expr:expr $(,)?) => {
        $crate::try_compile_str!($crate::config!(), $expr).expect("failed to compile")
    };
}

/// Compiles the given string of tokens and returns the output.
///
/// This macro will return an error if we fail to invoke the compiler. Unlike
/// the [`compile!`] macro, this will not panic.
///
/// ```rust
/// let output = postcompile::try_compile!({
///     const TEST: u32 = 1;
/// });
///
/// assert!(output.is_ok());
/// assert_eq!(output.unwrap().status, postcompile::ExitStatus::Success);
/// ```
#[macro_export]
macro_rules! try_compile {
    ($config:expr, { $($tokens:tt)* }$(,)?) => {
        $crate::try_compile_str!($crate::config!(), stringify!($($tokens)*))
    };
    ({ $($tokens:tt)* }$(,)?) => {
        $crate::try_compile_str!($crate::config!(), stringify!($($tokens)*))
    };
}

/// Compiles the given string of tokens and returns the output.
///
/// This macro will return an error if we fail to invoke the compiler.
///
/// Same as the [`try_compile!`] macro, but for strings similar usage to
/// [`compile_str!`].
#[macro_export]
macro_rules! try_compile_str {
    ($config:expr, $expr:expr $(,)?) => {
        $crate::compile_custom($expr, &$config)
    };
    ($expr:expr $(,)?) => {
        $crate::compile_custom($expr, &$crate::config!())
    };
}

/// Changelogs generated by [scuffle_changelog]
#[cfg(feature = "docs")]
#[scuffle_changelog::changelog]
pub mod changelog {}

#[cfg(test)]
#[cfg_attr(all(test, coverage_nightly), coverage(off))]
mod tests {
    use insta::assert_snapshot;

    use crate::Dependency;

    #[test]
    fn compile_success() {
        let out = compile!({
            #[allow(unused)]
            fn main() {
                let a = 1;
                let b = 2;
                let c = a + b;
            }
        });

        assert_snapshot!(out);
    }

    #[test]
    fn compile_failure() {
        let out = compile!({ invalid_rust_code });

        assert_snapshot!(out);
    }

    #[cfg(not(valgrind))]
    #[test]
    fn compile_tests() {
        let out = compile!(
            config! {
                test: true,
                dependencies: vec![
                    Dependency::version("tokio", "1").feature("full"),
                ]
            },
            {
                #[allow(unused)]
                fn fib(n: i32) -> i32 {
                    match n {
                        i32::MIN..=0 => 0,
                        1 => 1,
                        n => fib(n - 1) + fib(n - 2),
                    }
                }

                #[tokio::test]
                async fn test_fib() {
                    assert_eq!(fib(0), 0);
                    assert_eq!(fib(1), 1);
                    assert_eq!(fib(2), 1);
                    assert_eq!(fib(3), 2);
                    assert_eq!(fib(10), 55);
                }
            }
        );

        assert_snapshot!(out)
    }
}

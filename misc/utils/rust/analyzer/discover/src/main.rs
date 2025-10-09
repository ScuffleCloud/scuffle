//! Binary used for automatic Rust workspace discovery by `rust-analyzer`.
//! See [rust-analyzer documentation][rd] for a thorough description of this interface.
//!
//! [rd]: <https://rust-analyzer.github.io/manual.html#rust-analyzer.workspace.discoverConfig>

mod query;
mod rust_project;

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

use anyhow::{Context, bail};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use env_logger::fmt::Formatter;
use env_logger::{Target, WriteStyle};
use log::{LevelFilter, Record};
use rust_project::RustProject;
pub use rust_project::{DiscoverProject, RustAnalyzerArg};
use serde::de::DeserializeOwned;

pub const WORKSPACE_ROOT_FILE_NAMES: &[&str] = &["MODULE.bazel", "REPO.bazel", "WORKSPACE.bazel", "WORKSPACE"];

pub const BUILD_FILE_NAMES: &[&str] = &["BUILD.bazel", "BUILD"];

/// Looks within the current directory for a file that marks a bazel workspace.
///
/// # Errors
///
/// Returns an error if no file from [`WORKSPACE_ROOT_FILE_NAMES`] is found.
fn find_workspace_root_file(workspace: &Utf8Path) -> anyhow::Result<Utf8PathBuf> {
    BUILD_FILE_NAMES
        .iter()
        .chain(WORKSPACE_ROOT_FILE_NAMES)
        .map(|file| workspace.join(file))
        .find(|p| p.exists())
        .with_context(|| format!("no root file found for bazel workspace {workspace}"))
}

fn project_discovery() -> anyhow::Result<DiscoverProject<'static>> {
    let Config {
        workspace,
        execution_root,
        output_base,
        bazel,
        bazel_startup_options,
        bazel_args,
        rust_analyzer_argument,
    } = Config::parse()?;

    log::info!("got rust-analyzer argument: {rust_analyzer_argument:?}");

    let ra_arg = match rust_analyzer_argument {
        Some(ra_arg) => ra_arg,
        None => RustAnalyzerArg::Buildfile(find_workspace_root_file(&workspace)?),
    };

    log::info!("resolved rust-analyzer argument: {ra_arg:?}");

    let (buildfile, targets) = ra_arg.into_target_details(&workspace)?;

    log::debug!("got buildfile: {buildfile}");
    log::debug!("got targets: {targets}");

    // Use the generated files to print the rust-project.json.
    let project = generate_rust_project(
        &bazel,
        &output_base,
        &workspace,
        &execution_root,
        &bazel_startup_options,
        &bazel_args,
        &[targets],
    )?;

    std::fs::write(
        workspace.join(".rust-project.bazel.json"),
        serde_json::to_string_pretty(&project).unwrap(),
    )
    .context("failed to write output")?;

    Ok(DiscoverProject::Finished { buildfile, project })
}

#[allow(clippy::writeln_empty_string)]
fn write_discovery<W>(mut writer: W, discovery: DiscoverProject) -> std::io::Result<()>
where
    W: Write,
{
    serde_json::to_writer(&mut writer, &discovery)?;
    // `rust-analyzer` reads messages line by line, so we must add a newline after each
    writeln!(writer, "")
}

fn main() -> anyhow::Result<()> {
    let log_format_fn = |fmt: &mut Formatter, rec: &Record| {
        let message = rec.args();
        let discovery = DiscoverProject::Progress { message };
        write_discovery(fmt, discovery)
    };

    // Treat logs as progress messages.
    env_logger::Builder::from_default_env()
        // Never write color/styling info
        .write_style(WriteStyle::Never)
        // Format logs as progress messages
        .format(log_format_fn)
        // `rust-analyzer` reads the stdout
        .filter_level(LevelFilter::Trace)
        .target(Target::Stdout)
        .init();

    let discovery = match project_discovery() {
        Ok(discovery) => discovery,
        Err(error) => DiscoverProject::Error {
            error: error.to_string(),
            source: error.source().as_ref().map(ToString::to_string),
        },
    };

    write_discovery(io::stdout(), discovery).context("failed to write discovery")?;
    Ok(())
}

#[derive(Debug)]
pub struct Config {
    /// The path to the Bazel workspace directory. If not specified, uses the result of `bazel info workspace`.
    workspace: Utf8PathBuf,

    /// The path to the Bazel execution root. If not specified, uses the result of `bazel info execution_root`.
    execution_root: Utf8PathBuf,

    /// The path to the Bazel output user root. If not specified, uses the result of `bazel info output_base`.
    output_base: Utf8PathBuf,

    /// The path to a Bazel binary.
    bazel: Utf8PathBuf,

    /// Startup options to pass to `bazel` invocations.
    /// See the [Command-Line Reference](<https://bazel.build/reference/command-line-reference>)
    /// for more details.
    bazel_startup_options: Vec<String>,

    /// Arguments to pass to `bazel` invocations.
    /// See the [Command-Line Reference](<https://bazel.build/reference/command-line-reference>)
    /// for more details.
    bazel_args: Vec<String>,

    /// The argument that `rust-analyzer` can pass to the binary.
    rust_analyzer_argument: Option<RustAnalyzerArg>,
}

impl Config {
    // Parse the configuration flags and supplement with bazel info as needed.
    pub fn parse() -> anyhow::Result<Self> {
        let ConfigParser {
            workspace,
            bazel,
            bazel_startup_options,
            bazel_args,
            rust_analyzer_argument,
        } = ConfigParser::parse();

        // We need some info from `bazel info`. Fetch it now.
        let mut info_map = bazel_info(&bazel, workspace.as_deref(), None, &bazel_startup_options)?;

        let config = Config {
            workspace: info_map
                .remove("workspace")
                .expect("'workspace' must exist in bazel info")
                .into(),
            execution_root: info_map
                .remove("execution_root")
                .expect("'execution_root' must exist in bazel info")
                .into(),
            output_base: info_map
                .remove("output_base")
                .expect("'output_base' must exist in bazel info")
                .into(),
            bazel,
            bazel_startup_options,
            bazel_args,
            rust_analyzer_argument,
        };

        Ok(config)
    }
}

#[derive(Debug, Parser)]
struct ConfigParser {
    /// The path to the Bazel workspace directory. If not specified, uses the result of `bazel info workspace`.
    #[clap(long, env = "BUILD_WORKSPACE_DIRECTORY")]
    workspace: Option<Utf8PathBuf>,

    /// The path to a Bazel binary.
    #[clap(long, default_value = "bazel", env = "BAZEL")]
    bazel: Utf8PathBuf,

    /// Startup options to pass to `bazel` invocations.
    /// See the [Command-Line Reference](<https://bazel.build/reference/command-line-reference>)
    /// for more details.
    #[clap(long = "bazel_startup_option")]
    bazel_startup_options: Vec<String>,

    /// Arguments to pass to `bazel` invocations.
    /// See the [Command-Line Reference](<https://bazel.build/reference/command-line-reference>)
    /// for more details.
    #[clap(long = "bazel_arg")]
    bazel_args: Vec<String>,

    /// The argument that `rust-analyzer` can pass to the binary.
    rust_analyzer_argument: Option<RustAnalyzerArg>,
}

#[allow(clippy::too_many_arguments)]
pub fn generate_rust_project(
    bazel: &Utf8Path,
    output_base: &Utf8Path,
    workspace: &Utf8Path,
    execution_root: &Utf8Path,
    bazel_startup_options: &[String],
    bazel_args: &[String],
    targets: &[String],
) -> anyhow::Result<RustProject> {
    let crate_specs = query::get_crate_specs(
        bazel,
        output_base,
        workspace,
        execution_root,
        bazel_startup_options,
        bazel_args,
        targets,
    )?;

    let path = std::env::var("RUST_ANALYZER_TOOLCHAIN_PATH").context("MISSING RUST_ANALYZER_TOOLCHAIN_PATH")?;

    #[cfg(bazel_runfiles)]
    let path: Utf8PathBuf = runfiles::rlocation!(runfiles::Runfiles::create()?, path)
        .context("toolchain runfile not found")?
        .try_into()?;

    #[cfg(not(bazel_runfiles))]
    let path = Utf8PathBuf::from(path);

    let toolchain_info = deserialize_file_content(&path, output_base, workspace, execution_root)?;

    rust_project::assemble_rust_project(bazel, workspace, output_base, toolchain_info, crate_specs)
}

/// Executes `bazel info` to get a map of context information.
pub fn bazel_info(
    bazel: &Utf8Path,
    workspace: Option<&Utf8Path>,
    output_base: Option<&Utf8Path>,
    bazel_startup_options: &[String],
) -> anyhow::Result<BTreeMap<String, String>> {
    let output = bazel_command(bazel, workspace, output_base)
        .args(bazel_startup_options)
        .arg("info")
        .output()?;

    if !output.status.success() {
        let status = output.status;
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("bazel info failed: ({status:?})\n{stderr}");
    }

    // Extract and parse the output.
    let info_map = String::from_utf8(output.stdout)?
        .trim()
        .split('\n')
        .filter_map(|line| line.split_once(':'))
        .map(|(k, v)| (k.to_owned(), v.trim().to_owned()))
        .collect();

    Ok(info_map)
}

fn bazel_command(bazel: &Utf8Path, workspace: Option<&Utf8Path>, output_base: Option<&Utf8Path>) -> Command {
    let mut cmd = Command::new(bazel);

    cmd
        // Switch to the workspace directory if one was provided.
        .current_dir(workspace.unwrap_or(Utf8Path::new(".")))
        .env_remove("BAZELISK_SKIP_WRAPPER")
        .env_remove("BUILD_WORKING_DIRECTORY")
        .env_remove("BUILD_WORKSPACE_DIRECTORY")
        // Set the output_base if one was provided.
        .args(output_base.map(|s| format!("--output_base={s}")));

    cmd
}

fn deserialize_file_content<T>(
    path: &Utf8Path,
    output_base: &Utf8Path,
    workspace: &Utf8Path,
    execution_root: &Utf8Path,
) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read file: {path}"))?
        .replace("__WORKSPACE__", workspace.as_str())
        .replace("${pwd}", execution_root.as_str())
        .replace("__EXEC_ROOT__", execution_root.as_str())
        .replace("__OUTPUT_BASE__", output_base.as_str());

    log::trace!("{path}\n{content}");

    serde_json::from_str(&content).with_context(|| format!("failed to deserialize file: {path}"))
}

/// `rust-analyzer` associates workspaces with buildfiles. Therefore, when it passes in a
/// source file path, we use this function to identify the buildfile the file belongs to.
fn source_file_to_buildfile(file: &Utf8Path) -> anyhow::Result<Utf8PathBuf> {
    // Skip the first element as it's always the full file path.
    file.ancestors()
        .skip(1)
        .flat_map(|dir| BUILD_FILE_NAMES.iter().map(move |build| dir.join(build)))
        .find(|p| p.exists())
        .with_context(|| format!("no buildfile found for {file}"))
}

fn buildfile_to_targets(workspace: &Utf8Path, buildfile: &Utf8Path) -> anyhow::Result<String> {
    log::info!("getting targets for buildfile: {buildfile}");

    let parent_dir = buildfile
        .strip_prefix(workspace)
        .with_context(|| format!("{buildfile} not part of workspace"))?
        .parent();

    let targets = match parent_dir {
        Some(p) if !p.as_str().is_empty() => format!("//{p}:all"),
        _ => "//...".to_string(),
    };

    Ok(targets)
}

#[derive(Debug, serde_derive::Deserialize)]
struct ToolchainInfo {
    sysroot: Utf8PathBuf,
    sysroot_src: Utf8PathBuf,
}

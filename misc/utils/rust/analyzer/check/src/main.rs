use std::collections::BTreeMap;
use std::process::Command;

use anyhow::{Context, bail};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;

/// Executes `bazel info` to get a map of context information.
fn bazel_info(
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

fn main() -> anyhow::Result<()> {
    let config = Config::parse()?;

    let command = bazel_command(&config.bazel, Some(&config.workspace), Some(&config.output_base))
        .arg("query")
        .arg(format!(r#"kind("rust_clippy rule", set({}))"#, config.targets.join(" ")))
        .output()
        .context("bazel query")?;

    if !command.status.success() {
        anyhow::bail!("failed to query targets: {}", String::from_utf8_lossy(&command.stderr))
    }

    let targets = String::from_utf8_lossy(&command.stdout);
    let items: Vec<_> = targets.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

    let command = bazel_command(&config.bazel, Some(&config.workspace), Some(&config.output_base))
        .arg("cquery")
        .args(&config.bazel_args)
        .arg(format!("set({})", items.join(" ")))
        .arg("--output=starlark")
        .arg("--keep_going")
        .arg("--starlark:expr=[file.path for file in target.files.to_list()]")
        .arg("--build")
        .arg("--output_groups=rust_clippy")
        .output()
        .context("bazel cquery")?;

    let targets = String::from_utf8_lossy(&command.stdout);

    let mut clippy_files = Vec::new();
    for line in targets.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
        clippy_files.extend(serde_json::from_str::<Vec<String>>(line).context("parse line")?);
    }

    for file in clippy_files {
        let path = config.execution_root.join(&file);
        if !path.exists() {
            continue;
        }

        let content = std::fs::read_to_string(path).context("read")?;
        for line in content.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
            println!("{line}");
        }
    }

    Ok(())
}

#[derive(Debug)]
struct Config {
    /// The path to the Bazel workspace directory. If not specified, uses the result of `bazel info workspace`.
    workspace: Utf8PathBuf,

    /// The path to the Bazel execution root. If not specified, uses the result of `bazel info execution_root`.
    execution_root: Utf8PathBuf,

    /// The path to the Bazel output user root. If not specified, uses the result of `bazel info output_base`.
    output_base: Utf8PathBuf,

    /// The path to a Bazel binary.
    bazel: Utf8PathBuf,

    /// Arguments to pass to `bazel` invocations.
    /// See the [Command-Line Reference](<https://bazel.build/reference/command-line-reference>)
    /// for more details.
    bazel_args: Vec<String>,

    /// Space separated list of target patterns that comes after all other args.
    targets: Vec<String>,
}

impl Config {
    // Parse the configuration flags and supplement with bazel info as needed.
    fn parse() -> anyhow::Result<Self> {
        let ConfigParser {
            workspace,
            execution_root,
            output_base,
            bazel,
            config,
            targets,
        } = ConfigParser::parse();

        let bazel_args = config.into_iter().map(|s| format!("--config={s}")).collect();

        match (workspace, execution_root, output_base) {
            (Some(workspace), Some(execution_root), Some(output_base)) => Ok(Config {
                workspace,
                execution_root,
                output_base,
                bazel,
                bazel_args,
                targets,
            }),
            (workspace, _, output_base) => {
                let mut info_map = bazel_info(&bazel, workspace.as_deref(), output_base.as_deref(), &[])?;

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
                    bazel_args,
                    targets,
                };

                Ok(config)
            }
        }
    }
}

#[derive(Debug, Parser)]
struct ConfigParser {
    /// The path to the Bazel workspace directory. If not specified, uses the result of `bazel info workspace`.
    #[clap(long, env = "BUILD_WORKSPACE_DIRECTORY")]
    workspace: Option<Utf8PathBuf>,

    /// The path to the Bazel execution root. If not specified, uses the result of `bazel info execution_root`.
    #[clap(long)]
    execution_root: Option<Utf8PathBuf>,

    /// The path to the Bazel output user root. If not specified, uses the result of `bazel info output_base`.
    #[clap(long, env = "OUTPUT_BASE")]
    output_base: Option<Utf8PathBuf>,

    /// The path to a Bazel binary.
    #[clap(long, default_value = "bazel", env = "BAZEL")]
    bazel: Utf8PathBuf,

    /// A config to pass to Bazel invocations with `--config=<config>`.
    #[clap(long)]
    config: Option<String>,

    /// Space separated list of target patterns that comes after all other args.
    #[clap(default_value = "@//...")]
    targets: Vec<String>,
}

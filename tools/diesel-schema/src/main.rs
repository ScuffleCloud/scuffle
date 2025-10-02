use std::collections::BTreeMap;
use std::io::Read;
use std::process::{Command, Stdio};

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

// TODO(david): This shells out to an expected rule in the workspace root //:rust_analyzer that the user must define.
// It would be more convenient if it could automatically discover all the rust code in the workspace if this target
// does not exist.
fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = Config::parse()?;

    log::info!("running build query");

    let mut command = bazel_command(&config.bazel, Some(&config.workspace), Some(&config.output_base))
        .arg("query")
        .arg(format!(r#"kind("diesel_migration rule", set({}))"#, config.targets.join(" ")))
        .stderr(Stdio::inherit())
        .stdout(Stdio::piped())
        .spawn()
        .context("bazel query")?;

    let mut stdout = command.stdout.take().unwrap();
    let mut targets = String::new();
    stdout.read_to_string(&mut targets).context("stdout read")?;
    if !command.wait().context("query wait")?.success() {
        bail!("failed to run bazel query")
    }

    let items: Vec<_> = targets.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

    let mut command = bazel_command(&config.bazel, Some(&config.workspace), Some(&config.output_base))
        .arg("cquery")
        .args(&config.bazel_args)
        .arg(format!("set({})", items.join(" ")))
        .arg("--output=starlark")
        .arg("--starlark:expr=[file.path for file in target.files.to_list()]")
        .arg("--build")
        .arg("--output_groups=diesel_schema")
        .stderr(Stdio::inherit())
        .stdout(Stdio::piped())
        .spawn()
        .context("bazel cquery")?;

    let mut stdout = command.stdout.take().unwrap();

    let mut targets = String::new();
    stdout.read_to_string(&mut targets).context("stdout read")?;

    if !command.wait().context("cquery wait")?.success() {
        bail!("failed to run bazel cquery")
    }

    let mut diesel_schema_files = Vec::new();

    for line in targets.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
        diesel_schema_files.extend(serde_json::from_str::<Vec<String>>(line).context("parse line")?);
    }

    for file in diesel_schema_files {
        let path = config.execution_root.join(&file);
        if !path.exists() {
            log::warn!("missing {file}");
            continue;
        }

        let file_content = std::fs::read_to_string(&path).context("read")?;
        let files: BTreeMap<Utf8PathBuf, String> = serde_json::from_str(&file_content).context("parse line")?;
        for (file, content) in files {
            let path = config.execution_root.join(&file);
            let current_content = std::fs::read_to_string(&path).context("read")?;
            if content != current_content {
                log::info!("Updating {}", file);
                std::fs::write(&path, content).context("write output")?;
            } else {
                log::info!("{} already up-to-date", path);
            }
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

        let bazel_args = vec![format!("--config={config}")];

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
    #[clap(long, default_value = "bazel")]
    bazel: Utf8PathBuf,

    /// A config to pass to Bazel invocations with `--config=<config>`.
    #[clap(long, default_value = "wrapper")]
    config: String,

    /// Space separated list of target patterns that comes after all other args.
    #[clap(default_value = "@//...")]
    targets: Vec<String>,
}

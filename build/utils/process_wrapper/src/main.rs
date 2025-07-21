use std::collections::BTreeMap;
use std::sync::LazyLock;

use camino::Utf8PathBuf;
use clap::Parser;
use regex::Captures;

pub type ArgsFile = Vec<String>;
pub type EnvFile = BTreeMap<String, String>;

static REGEX: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"bazel-out\/[\w-]+\/bin\/(external\/)?").expect("INVALID REGEX"));

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, env = "PROCESS_WRAPPER_ARGS_FILE")]
    args_file: Utf8PathBuf,

    #[arg(long, env = "PROCESS_WRAPPER_TARGET")]
    target: Utf8PathBuf,

    #[arg(long, env = "PROCESS_WRAPPER_FIX_BUILD_PATHS")]
    fix_build_paths: bool,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    extra: Vec<String>,
}

fn fix_path_regex(arg: String) -> String {
    REGEX
        .replace_all(&arg, |captures: &Captures| {
            let is_external = captures.get(1).is_some();
            if is_external { "../" } else { "" }
        })
        .into_owned()
}

fn main() {
    let args = Args::parse();

    let mut command = std::process::Command::new(args.target);

    let fix_path = if args.fix_build_paths {
        fix_path_regex
    } else {
        |path: String| path
    };

    command.env_clear();

    command.envs(
        std::env::vars()
            .filter(|(key, _)| !key.starts_with("PROCESS_WRAPPER_"))
            .map(|(key, value)| (key, fix_path(value))),
    );

    let args_file_str = std::fs::read_to_string(args.args_file).expect("PROCESS_WRAPPER_ARGS_FILE malformed");
    let arg_file: ArgsFile = serde_json::from_str(&args_file_str).expect("PROCESS_WRAPPER_ARGS_FILE malformed");
    command.args(arg_file.into_iter().map(fix_path));
    command.args(args.extra.into_iter().map(fix_path));

    let status = command.status().expect("PROCESS_WRAPPER_TARGET not found");
    std::process::exit(status.code().unwrap_or(1))
}

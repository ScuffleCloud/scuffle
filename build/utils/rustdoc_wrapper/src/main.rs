use std::process::Stdio;

use camino::Utf8PathBuf;
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    json_out: Option<Utf8PathBuf>,

    #[arg(long)]
    test_out: Option<Utf8PathBuf>,

    #[arg(long)]
    rustdoc: Utf8PathBuf,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    rustdoc_args: Vec<String>,
}

fn json_mode(args: Args) {
    let mut cmd = std::process::Command::new(args.rustdoc);
    cmd.args(args.rustdoc_args);
    let json_out_dir = camino_tempfile::tempdir().expect("unable to create tmp dir");
    cmd.arg("--out-dir").arg(json_out_dir.path());

    let status = cmd.status().expect("failed to run");
    if !status.success() {
        std::process::exit(status.code().unwrap_or(127))
    }

    let json_out = args.json_out.unwrap();
    let dir = std::fs::read_dir(json_out_dir.path()).expect("read dir failed");
    let items: Result<Vec<_>, _> = dir.collect();
    let items = items.expect("failed to read dir");
    if items.len() != 1 {
        panic!("expected exactly 1 file in output dir found: {}", items.len());
    }

    let filetype = items[0].file_type().expect("failed to read filetype");
    if !filetype.is_file() {
        panic!("json output is not file");
    }

    std::fs::copy(items[0].path(), json_out).expect("failed to copy output");
}

fn html_mode(args: Args) {
    let mut cmd = std::process::Command::new(args.rustdoc);
    cmd.args(args.rustdoc_args);
    let status = cmd.status().expect("failed to run");
    if !status.success() {
        std::process::exit(status.code().unwrap_or(127))
    }
}

fn test_mode(args: Args) {
    let mut cmd = std::process::Command::new(args.rustdoc);
    cmd.args(args.rustdoc_args);
    cmd.arg("--output-format=doctest");
    cmd.stderr(Stdio::inherit());
    cmd.stdout(Stdio::piped());

    let output = cmd.output().expect("failed to run");
    if !output.status.success() {
        std::process::exit(output.status.code().unwrap_or(127))
    }

    std::fs::write(args.test_out.unwrap(), output.stdout).expect("failed to write doctest output")
}

fn main() {
    let args = Args::parse();

    match (args.json_out.is_some(), args.test_out.is_some()) {
        (true, false) => json_mode(args),
        (false, true) => test_mode(args),
        (false, false) => html_mode(args),
        (true, true) => panic!("cannot specify both --test-out and --json-out"),
    }
}

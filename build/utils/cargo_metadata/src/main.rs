use std::process::Stdio;

use camino::Utf8PathBuf;
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, env = "CARGO")]
    cargo: Utf8PathBuf,

    #[arg(long, env = "CARGO_MANIFEST")]
    manifest: Option<Utf8PathBuf>,

    #[arg(long, env = "METADATA_OUT")]
    metadata_out: Utf8PathBuf,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    extra: Vec<String>,
}

fn main() {
    let args = Args::parse();

    let mut cmd = std::process::Command::new(args.cargo);

    cmd.arg("metadata").arg("--format-version=1");

    if let Some(manifest) = args.manifest {
        cmd.arg("--manifest-path").arg(manifest);
    }

    let output = cmd
        .args(args.extra)
        .stderr(Stdio::inherit())
        .stdout(Stdio::piped())
        .output()
        .expect("cargo not found");

    if !output.status.success() {
        panic!("failed to run cargo metadata");
    }

    let pwd = camino::Utf8PathBuf::from_path_buf(std::env::current_dir().unwrap()).unwrap();

    let output = String::from_utf8_lossy(&output.stdout).replace(pwd.as_str(), "./");

    std::fs::write(args.metadata_out, output).expect("failed to write output");
}

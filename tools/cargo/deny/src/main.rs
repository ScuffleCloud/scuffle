use std::path::PathBuf;

use clap::Parser;

extern crate runfiles;

#[derive(Debug, clap::Parser)]
struct Args {
    #[arg(long, env = "CARGO_BIN")]
    cargo_bin: PathBuf,
    #[arg(long, env = "CARGO_DENY_BIN")]
    deny_bin: PathBuf,
    #[arg(long, env = "CARGO_DENY_CONFIG")]
    deny_config: PathBuf,
}

fn main() {
    let args = Args::parse();

    let runfiles = runfiles::Runfiles::create().unwrap();
    let cargo_bin = runfiles::rlocation!(runfiles, args.cargo_bin).unwrap();
    let deny_bin = runfiles::rlocation!(runfiles, args.deny_bin).unwrap();
    let deny_config = runfiles::rlocation!(runfiles, args.deny_config).unwrap();

    let status = std::process::Command::new(deny_bin)
        .arg("check")
        .arg("--config")
        .arg(deny_config)
        .env("CARGO", cargo_bin)
        .status()
        .expect("missing deny");

    std::process::exit(status.code().unwrap_or(127))
}

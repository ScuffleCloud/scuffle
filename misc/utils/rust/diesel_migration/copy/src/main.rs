//! A very small binary which copies the input to the build workspace directory.

use std::collections::BTreeMap;

use anyhow::Context;
use camino::Utf8PathBuf;
use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, env = "INPUT_PATH")]
    input_path: Utf8PathBuf,

    #[arg(long, env = "BUILD_WORKSPACE_DIRECTORY")]
    build_workspace_directory: Utf8PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let runfiles = runfiles::Runfiles::create().expect("failed to create runfiles");
    let input_path = runfiles::rlocation!(&runfiles, &args.input_path).expect("failed to get input file");
    let input_content = std::fs::read_to_string(&input_path).context("read input file")?;

    let input: BTreeMap<Utf8PathBuf, String> = serde_json::from_str(&input_content).context("parse input")?;

    for (path, content) in input {
        let dest_path = args.build_workspace_directory.join(&path);
        println!("Writing file {}", dest_path);
        std::fs::write(dest_path, content).context("write content")?;
    }

    Ok(())
}

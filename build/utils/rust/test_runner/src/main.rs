//! A custom test runner for rust tests which wraps [test_runner_lib] and invokes [nextest](https://github.com/nextest-rs/nextest).

use camino::Utf8PathBuf;
use clap::Parser;
use test_runner_lib::{Binary, Config};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(env = "RUNNER_CRATE")]
    package: String,

    #[arg(env = "RUNNER_BINARY")]
    binary: Utf8PathBuf,

    #[arg(env = "RUNNER_CONFIG")]
    config: Utf8PathBuf,

    #[arg(env = "RUNNER_PROFILE")]
    profile: String,

    #[arg(env = "TEST_TMPDIR")]
    tmp_dir: Utf8PathBuf,

    #[arg(env = "XML_OUTPUT_FILE")]
    xml_output_file: Option<Utf8PathBuf>,

    #[arg(env = "TEST_TARGET")]
    target: Option<String>,
}

fn main() {
    let args = Args::parse();

    test_runner_lib::run_nextest(Config {
        config_path: args.config,
        package: args.package.clone(),
        profile: args.profile,
        tmp_dir: args.tmp_dir,
        xml_output_file: args.xml_output_file,
        binaries: vec![Binary {
            name: args.target.unwrap_or(args.package),
            path: args.binary,
        }],
    });
}

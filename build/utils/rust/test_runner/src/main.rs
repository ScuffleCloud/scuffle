//! A custom test runner for rust tests which wraps [test_runner_lib] and invokes [nextest](https://github.com/nextest-rs/nextest).

use camino::Utf8PathBuf;
use clap::Parser;
use test_runner_lib::{Binary, Config};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, env = "RUNNER_CRATE")]
    package: String,

    #[arg(long, env = "RUNNER_BINARY")]
    binary: Utf8PathBuf,

    #[arg(long, env = "RUNNER_CONFIG")]
    config: Utf8PathBuf,

    #[arg(long, env = "RUNNER_PROFILE")]
    profile: String,

    #[arg(long, env = "RUNNER_INSTA")]
    insta: bool,

    #[arg(long, env = "RUNNER_SOURCE_DIR")]
    source_dir: Utf8PathBuf,

    #[arg(long, env = "TEST_TMPDIR")]
    tmp_dir: Utf8PathBuf,

    #[arg(long, env = "XML_OUTPUT_FILE")]
    xml_output_file: Option<Utf8PathBuf>,

    #[arg(long, env = "TEST_TARGET")]
    target: Option<String>,

    #[arg(long, env = "TEST_UNDECLARED_OUTPUTS_DIR")]
    test_output_dir: Option<Utf8PathBuf>,

    #[command(flatten)]
    args: test_runner_lib::Args,
}

fn main() {
    let args = Args::parse();

    test_runner_lib::run_nextest(Config {
        config_path: args.config,
        package: args.package.clone(),
        profile: args.profile,
        tmp_dir: args.tmp_dir,
        xml_output_file: args.xml_output_file,
        args: args.args,
        insta: args.insta,
        source_dir: args.source_dir,
        test_output_dir: args.test_output_dir,
        binaries: vec![Binary {
            name: args.target.unwrap_or(args.package),
            path: args.binary,
        }],
    });
}

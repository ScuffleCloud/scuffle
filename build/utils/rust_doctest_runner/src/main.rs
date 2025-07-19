use camino::Utf8PathBuf;
use clap::Parser;
use nextest_runner_lib::{Binary, Config};
use rust_doctest_common::Manifest;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(env = "RUNNER_CRATE")]
    package: String,

    #[arg(env = "RUNNER_DOCTEST_OUT")]
    doctest_out: Utf8PathBuf,

    #[arg(env = "RUNNER_CONFIG")]
    config: Utf8PathBuf,

    #[arg(env = "RUNNER_PROFILE")]
    profile: String,

    #[arg(env = "TEST_TMPDIR")]
    tmp_dir: Utf8PathBuf,

    #[arg(env = "XML_OUTPUT_FILE")]
    xml_output_file: Option<Utf8PathBuf>,
}

fn main() {
    let args = Args::parse();

    let manifest_path = args.doctest_out.join("manifest.json");
    let manifest = std::fs::read_to_string(manifest_path).expect("invalid manifest");
    let manifest: Manifest = serde_json::from_str(&manifest).expect("malformed manifest");

    for (var, path) in manifest.standalone_binary_envs {
        // Safety: This function is safe to call in a single-threaded program.
        // We are starting up so at this point there are no other threads.
        unsafe { std::env::set_var(var, args.doctest_out.join(path)) };
    }

    nextest_runner_lib::run_nextest(Config {
        config_path: args.config,
        package: args.package.clone(),
        profile: args.profile,
        tmp_dir: args.tmp_dir,
        xml_output_file: args.xml_output_file,
        binaries: manifest
            .test_binaries
            .into_iter()
            .map(|binary| Binary {
                name: binary.name,
                path: args.doctest_out.join(binary.path),
            })
            .collect(),
    });
}

use std::collections::BTreeSet;

use camino::Utf8PathBuf;
use clap::Parser;
use nextest_filtering::ParseContext;
use nextest_metadata::{BuildPlatform, RustBinaryId, RustTestBinaryKind};
use nextest_runner::cargo_config::{CargoConfigs, EnvironmentMap};
use nextest_runner::config::NextestConfig;
use nextest_runner::double_spawn::DoubleSpawnInfo;
use nextest_runner::input::InputHandlerKind;
use nextest_runner::list::{RustBuildMeta, RustTestArtifact, TestExecuteContext};
use nextest_runner::platform::BuildPlatforms;
use nextest_runner::reporter::structured::StructuredReporter;
use nextest_runner::reporter::{ReporterBuilder, ReporterStderr};
use nextest_runner::reuse_build::PathMapper;
use nextest_runner::signal::SignalHandlerKind;
use nextest_runner::target_runner::TargetRunner;
use nextest_runner::test_filter::{FilterBound, RunIgnored, TestFilterBuilder};

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
    let cwd = Utf8PathBuf::from_path_buf(std::env::current_dir().unwrap()).unwrap();

    let metadata = serde_json::json!({
        "version": 1,
        "workspace_members": [],
        "workspace_default_members": [],
        "packages": [
            {
                "name": args.package,
                "version": "0.0.0",
                "id": args.package,
                "license": null,
                "license_file": null,
                "description": null,
                "source": null,
                "dependencies": [],
                "targets": [],
                "features": {},
                "manifest_path": cwd.join("Cargo.toml"),
                "metadata": null,
                "publish": null,
                "readme": null,
                "repository": null,
                "homepage": null,
                "documentation": null,
                "edition": "2024",
                "links": null,
                "default_run": null,
                "rust_version": null
            },
        ],
        "resolve": null,
        "workspace_root": cwd,
        "target_directory": cwd,
    })
    .to_string();

    let metadata = guppy::CargoMetadata::parse_json(metadata).unwrap();
    let graph = metadata.build_graph().unwrap();

    let package = graph.packages().find(|p| p.name() == args.package).unwrap();

    let double_spawn = DoubleSpawnInfo::disabled();
    let target_runner = TargetRunner::empty();

    let ctx = TestExecuteContext {
        double_spawn: &double_spawn,
        profile_name: "test",
        target_runner: &target_runner,
    };

    let artifact = RustTestArtifact {
        binary_id: RustBinaryId::new(args.target.as_ref().unwrap_or(&args.package)),
        binary_name: args.binary.file_name().unwrap().to_string(),
        binary_path: args.binary,
        cwd: cwd.clone(),
        build_platform: BuildPlatform::Target,
        kind: RustTestBinaryKind::LIB,
        non_test_binaries: BTreeSet::new(),
        package,
    };

    let mut nextest_config = std::fs::read_to_string(&args.config)
        .unwrap()
        .parse::<toml_edit::DocumentMut>()
        .unwrap();

    nextest_config["store"]["dir"] = "".to_string().into();

    let nextest_config_path = args.tmp_dir.join("__nextest-config.toml");

    std::fs::write(&nextest_config_path, nextest_config.to_string()).unwrap();

    let build_platforms = BuildPlatforms::new_with_no_target().unwrap();
    let config = NextestConfig::from_sources(
        &args.tmp_dir,
        &ParseContext::new(&graph),
        Some(&nextest_config_path),
        [],
        &BTreeSet::new(),
    )
    .unwrap();

    let profile = config.profile(&args.profile).unwrap().apply_build_platforms(&build_platforms);
    let meta = RustBuildMeta::new(cwd, build_platforms).map_paths(&PathMapper::noop());
    let filter = TestFilterBuilder::default_set(RunIgnored::Default);
    let env = EnvironmentMap::new(&CargoConfigs::new([] as [&str; 0]).unwrap());

    let list = nextest_runner::list::TestList::new(
        &ctx,
        Some(artifact),
        meta,
        &filter,
        Utf8PathBuf::new(),
        env,
        &profile,
        FilterBound::DefaultSet,
        1,
    )
    .unwrap();

    let runner = nextest_runner::runner::TestRunnerBuilder::default()
        .build(
            &list,
            &profile,
            Vec::new(),
            SignalHandlerKind::Standard,
            InputHandlerKind::Standard,
            double_spawn,
            target_runner,
        )
        .unwrap();

    let mut reporter = ReporterBuilder::default()
        .set_colorize(true)
        .set_hide_progress_bar(true)
        .build(&list, &profile, ReporterStderr::Terminal, StructuredReporter::new());

    let r = runner
        .execute(|event| {
            reporter.report_event(event).unwrap();
        })
        .unwrap();

    reporter.finish();

    if let (Some(junit), Some(output)) = (profile.junit(), args.xml_output_file.as_ref()) {
        let junit = std::fs::read(junit.path()).unwrap();
        std::fs::write(output, junit).unwrap();
    }

    if r.has_failures() {
        std::process::exit(1)
    }
}

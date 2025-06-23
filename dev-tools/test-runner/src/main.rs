use std::collections::BTreeSet;

use camino::Utf8PathBuf;
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

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(metadata_path) = args.next() else {
        panic!(
            "invalid usage: {} <metadata.json> <package> <test-binary>",
            std::env::args().next().unwrap_or_default()
        )
    };

    let Some(package) = args.next() else {
        panic!(
            "invalid usage: {} <metadata.json> <package> <test-binary>",
            std::env::args().next().unwrap_or_default()
        )
    };

    let Some(test_binary_path) = args.next() else {
        panic!(
            "invalid usage: {} <metadata.json> <package> <test-binary>",
            std::env::args().next().unwrap_or_default()
        )
    };

    let metadata_path = Utf8PathBuf::from(metadata_path);
    let test_binary_path = Utf8PathBuf::from(test_binary_path);

    let metadata = std::fs::read_to_string(metadata_path).unwrap();
    let metadata = guppy::CargoMetadata::parse_json(metadata).unwrap();
    let graph = metadata.build_graph().unwrap();

    let package = graph.packages().find(|p| p.name() == package).unwrap();

    let double_spawn = DoubleSpawnInfo::disabled();
    let target_runner = TargetRunner::empty();

    let ctx = TestExecuteContext {
        double_spawn: &double_spawn,
        profile_name: "test",
        target_runner: &target_runner,
    };

    let artifact = RustTestArtifact {
        binary_id: RustBinaryId::new("test-id"),
        binary_name: test_binary_path.file_name().unwrap().to_string(),
        binary_path: test_binary_path,
        cwd: Utf8PathBuf::from(std::env::current_dir().unwrap().to_string_lossy().as_ref()),
        build_platform: BuildPlatform::Target,
        kind: RustTestBinaryKind::LIB,
        non_test_binaries: BTreeSet::new(),
        package: package,
    };

    let build_platforms = BuildPlatforms::new_with_no_target().unwrap();
    let config =
        NextestConfig::from_sources(Utf8PathBuf::new(), &ParseContext::new(&graph), None, [], &BTreeSet::new()).unwrap();
    let profile = config.profile("default").unwrap().apply_build_platforms(&build_platforms);
    let meta = RustBuildMeta::new("", build_platforms).map_paths(&PathMapper::noop());
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
        .set_colorize(false)
        .set_hide_progress_bar(true)
        .build(&list, &profile, ReporterStderr::Terminal, StructuredReporter::new());

    let r = runner
        .execute(|event| {
            reporter.report_event(event).unwrap();
        })
        .unwrap();

    if r.has_failures() {
        std::process::exit(1)
    }
}

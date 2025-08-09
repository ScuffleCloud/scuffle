/// A library which exposes some minimal configurable targets for creating [nextest](https://github.com/nextest-rs/nextest) targets.
use std::collections::BTreeSet;

use camino::Utf8PathBuf;
use nextest_filtering::{Filterset, FiltersetKind, ParseContext};
use nextest_metadata::{BuildPlatform, RustBinaryId, RustTestBinaryKind};
use nextest_runner::cargo_config::{CargoConfigs, EnvironmentMap};
use nextest_runner::config::core::NextestConfig;
use nextest_runner::double_spawn::DoubleSpawnInfo;
use nextest_runner::input::InputHandlerKind;
use nextest_runner::list::{RustBuildMeta, RustTestArtifact, TestExecuteContext};
use nextest_runner::platform::BuildPlatforms;
use nextest_runner::reporter::structured::StructuredReporter;
use nextest_runner::reporter::{ReporterBuilder, ReporterStderr};
use nextest_runner::reuse_build::PathMapper;
use nextest_runner::signal::SignalHandlerKind;
use nextest_runner::target_runner::TargetRunner;
use nextest_runner::test_filter::{FilterBound, RunIgnored, TestFilterBuilder, TestFilterPatterns};

pub struct Config {
    pub package: String,
    pub config_path: Utf8PathBuf,
    pub tmp_dir: Utf8PathBuf,
    pub profile: String,
    pub xml_output_file: Option<Utf8PathBuf>,
    pub binaries: Vec<Binary>,
    pub args: Args,
}

#[derive(clap::Args, Debug)]
pub struct Args {
    #[arg(long = "expr", short = 'E')]
    pub expressions: Vec<String>,
    #[arg(long = "skip")]
    pub skipped: Vec<String>,
    #[arg(long)]
    pub exact: bool,
    #[arg(long)]
    pub ignored: bool,
    #[arg(long)]
    pub include_ignored: bool,
    #[arg(name = "TEST")]
    pub tests: Vec<String>,
}

pub struct Binary {
    pub name: String,
    pub path: Utf8PathBuf,
}

pub fn run_nextest(config: Config) {
    let cwd = &Utf8PathBuf::from_path_buf(std::env::current_dir().unwrap()).unwrap();

    let metadata = serde_json::json!({
        "version": 1,
        "workspace_members": [],
        "workspace_default_members": [],
        "packages": [
            {
                "name": config.package,
                "version": "0.0.0",
                "id": config.package,
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

    let package = graph.packages().find(|p| p.name() == config.package).unwrap();

    let double_spawn = DoubleSpawnInfo::disabled();
    let build_platforms = BuildPlatforms::new_with_no_target().unwrap();
    let configs = CargoConfigs::new([] as [&str; 0]).unwrap();
    let target_runner = TargetRunner::new(&configs, &build_platforms).unwrap();

    let ctx = TestExecuteContext {
        double_spawn: &double_spawn,
        profile_name: "test",
        target_runner: &target_runner,
    };

    let artifacts = config.binaries.iter().map(|binary| RustTestArtifact {
        binary_id: RustBinaryId::new(&binary.name),
        binary_name: binary.name.clone(),
        binary_path: binary.path.clone(),
        cwd: cwd.clone(),
        build_platform: BuildPlatform::Target,
        kind: RustTestBinaryKind::LIB,
        non_test_binaries: BTreeSet::new(),
        package,
    });

    let mut nextest_config = std::fs::read_to_string(&config.config_path)
        .unwrap()
        .parse::<toml_edit::DocumentMut>()
        .unwrap();

    nextest_config["store"]["dir"] = "".to_string().into();

    let nextest_config_path = config.tmp_dir.join("__nextest-config.toml");

    std::fs::write(&nextest_config_path, nextest_config.to_string()).unwrap();

    let build_platforms = BuildPlatforms::new_with_no_target().unwrap();
    let nextest_config = NextestConfig::from_sources(
        &config.tmp_dir,
        &ParseContext::new(&graph),
        Some(&nextest_config_path),
        [],
        &BTreeSet::new(),
    )
    .unwrap();

    let run_ignored = match (config.args.ignored, config.args.include_ignored) {
        (true, _) => RunIgnored::Only,
        (false, true) => RunIgnored::All,
        (false, false) => RunIgnored::Default,
    };

    let mut patterns = TestFilterPatterns::default();

    for test in config.args.tests {
        if config.args.exact {
            patterns.add_exact_pattern(test);
        } else {
            patterns.add_substring_pattern(test);
        }
    }

    for test in config.args.skipped {
        if config.args.exact {
            patterns.add_skip_exact_pattern(test);
        } else {
            patterns.add_skip_pattern(test);
        }
    }

    let exprs = config
        .args
        .expressions
        .into_iter()
        .map(|expr| Filterset::parse(expr, &ParseContext::new(&graph), FiltersetKind::Test))
        .collect::<Result<Vec<_>, _>>()
        .expect("failed to parse exprs");

    let profile = nextest_config
        .profile(&config.profile)
        .unwrap()
        .apply_build_platforms(&build_platforms);
    let meta = RustBuildMeta::new(cwd, build_platforms).map_paths(&PathMapper::noop());
    let filter = TestFilterBuilder::new(run_ignored, None, patterns, exprs).unwrap();
    let env = EnvironmentMap::new(&configs);

    let list = match nextest_runner::list::TestList::new(
        &ctx,
        artifacts,
        meta,
        &filter,
        Utf8PathBuf::new(),
        env,
        &profile,
        FilterBound::DefaultSet,
        1,
    ) {
        Ok(l) => l,
        Err(err) => {
            panic!("{err:#}");
        }
    };

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
        .build(&list, &profile, &configs, ReporterStderr::Terminal, StructuredReporter::new());

    let r = runner
        .execute(|event| {
            reporter.report_event(event).unwrap();
        })
        .unwrap();

    reporter.finish();

    if let (Some(junit), Some(output)) = (profile.junit(), config.xml_output_file.as_ref()) {
        let junit = std::fs::read(junit.path()).unwrap();
        std::fs::write(output, junit).unwrap();
    }

    if r.has_failures() {
        std::process::exit(1)
    }
}

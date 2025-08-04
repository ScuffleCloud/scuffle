use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;

use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};

use crate::{bazel_command, deserialize_file_content};

#[derive(Debug, serde_derive::Deserialize)]
struct AqueryOutput {
    artifacts: Vec<Artifact>,
    actions: Vec<Action>,
    #[serde(rename = "pathFragments")]
    path_fragments: Vec<PathFragment>,
}

#[derive(Debug, serde_derive::Deserialize)]
struct Artifact {
    id: u32,
    #[serde(rename = "pathFragmentId")]
    path_fragment_id: u32,
}

#[derive(Debug, serde_derive::Deserialize)]
struct PathFragment {
    id: u32,
    label: String,
    #[serde(rename = "parentId")]
    parent_id: Option<u32>,
}

#[derive(Debug, serde_derive::Deserialize)]
struct Action {
    #[serde(rename = "outputIds")]
    output_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde_derive::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CrateSpec {
    pub aliases: BTreeMap<String, String>,
    pub crate_id: String,
    pub display_name: String,
    pub edition: String,
    pub root_module: String,
    pub is_workspace_member: bool,
    pub deps: BTreeSet<String>,
    pub proc_macro_dylib_path: Option<String>,
    pub source: Option<CrateSpecSource>,
    pub cfg: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub target: String,
    pub crate_type: CrateType,
    pub is_test: bool,
    pub build: Option<CrateSpecBuild>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde_derive::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CrateSpecBuild {
    pub label: String,
    pub build_file: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, serde_derive::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CrateSpecSource {
    pub exclude_dirs: Vec<String>,
    pub include_dirs: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde_derive::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CrateType {
    Bin,
    Rlib,
    Lib,
    Dylib,
    Cdylib,
    Staticlib,
    ProcMacro,
}

#[allow(clippy::too_many_arguments)]
pub fn get_crate_specs(
    bazel: &Utf8Path,
    output_base: &Utf8Path,
    workspace: &Utf8Path,
    execution_root: &Utf8Path,
    bazel_startup_options: &[String],
    bazel_args: &[String],
    targets: &[String],
) -> anyhow::Result<impl IntoIterator<Item = CrateSpec>> {
    log::info!("running bazel aquery...");
    log::debug!("Get crate specs with targets: {targets:?}");
    let target_pattern = fmtools::fmt(|f| {
        f.write_str("deps(")?;
        for (idx, target) in targets.iter().enumerate() {
            if idx != 0 {
                f.write_char('+')?;
            }
            f.write_char('"')?;
            f.write_str(target)?;
            f.write_char('"')?;
        }
        f.write_char(')')?;
        Ok(())
    });

    let mut aquery_command = bazel_command(bazel, Some(workspace), Some(output_base));
    aquery_command
        .args(bazel_startup_options)
        .arg("aquery")
        .args(bazel_args)
        .arg("--build")
        .arg("--include_aspects")
        .arg("--include_artifacts")
        .arg("--aspects=//build/utils/rust:rust_analyzer.bzl%rust_analyzer_aspect")
        .arg("--output_groups=rust_analyzer_crate_spec,rust_generated_srcs,rust_analyzer_proc_macro_dylib,rust_analyzer_src")
        .arg(format!(r#"outputs(".*\.rust_analyzer_crate_spec\.json",{target_pattern})"#))
        .arg("--output=jsonproto");

    log::trace!("Running aquery: {aquery_command:#?}");
    let aquery_output = aquery_command.output().context("Failed to spawn aquery command")?;

    log::info!("bazel aquery finished; parsing spec files...");

    let aquery_results = String::from_utf8(aquery_output.stdout).context("Failed to decode aquery results as utf-8.")?;

    let crate_spec_files = parse_aquery_output_files(execution_root, &aquery_results)?;

    let crate_specs = crate_spec_files
        .into_iter()
        .map(|file| deserialize_file_content(&file, output_base, workspace, execution_root))
        .collect::<anyhow::Result<Vec<CrateSpec>>>()?;

    consolidate_crate_specs(crate_specs)
}

fn parse_aquery_output_files(execution_root: &Utf8Path, aquery_stdout: &str) -> anyhow::Result<Vec<Utf8PathBuf>> {
    let out: AqueryOutput = serde_json::from_str(aquery_stdout).map_err(|_| {
        // Parsing to `AqueryOutput` failed, try parsing into a `serde_json::Value`:
        match serde_json::from_str::<serde_json::Value>(aquery_stdout) {
            Ok(serde_json::Value::Object(_)) => {
                // If the JSON is an object, it's likely that the aquery command failed.
                anyhow::anyhow!("Aquery returned an empty result, are there any Rust targets in the specified paths?.")
            }
            _ => {
                anyhow::anyhow!("Failed to parse aquery output as JSON")
            }
        }
    })?;

    let artifacts = out.artifacts.iter().map(|a| (a.id, a)).collect::<BTreeMap<_, _>>();
    let path_fragments = out.path_fragments.iter().map(|pf| (pf.id, pf)).collect::<BTreeMap<_, _>>();

    let mut output_files: Vec<Utf8PathBuf> = Vec::new();
    for action in out.actions {
        for output_id in action.output_ids {
            let artifact = artifacts.get(&output_id).expect("internal consistency error in bazel output");
            let path = path_from_fragments(artifact.path_fragment_id, &path_fragments)?;
            let path = execution_root.join(path);
            if path.exists() {
                output_files.push(path);
            } else {
                log::warn!("Skipping missing crate_spec file: {path:?}");
            }
        }
    }

    Ok(output_files)
}

fn path_from_fragments(id: u32, fragments: &BTreeMap<u32, &PathFragment>) -> anyhow::Result<Utf8PathBuf> {
    let path_fragment = fragments.get(&id).expect("internal consistency error in bazel output");

    let buf = match path_fragment.parent_id {
        Some(parent_id) => path_from_fragments(parent_id, fragments)?.join(Utf8PathBuf::from(&path_fragment.label.clone())),
        None => Utf8PathBuf::from(&path_fragment.label.clone()),
    };

    Ok(buf)
}

/// Read all crate specs, deduplicating crates with the same ID. This happens when
/// a rust_test depends on a rust_library, for example.
fn consolidate_crate_specs(crate_specs: Vec<CrateSpec>) -> anyhow::Result<impl IntoIterator<Item = CrateSpec>> {
    let mut consolidated_specs: BTreeMap<String, CrateSpec> = BTreeMap::new();
    for mut spec in crate_specs.into_iter() {
        if let Some(existing) = consolidated_specs.get_mut(&spec.crate_id) {
            existing.deps.extend(spec.deps);
            existing.env.extend(spec.env);
            existing.aliases.extend(spec.aliases);

            if let Some(source) = &mut existing.source {
                if let Some(mut new_source) = spec.source {
                    new_source.exclude_dirs.retain(|src| !source.exclude_dirs.contains(src));
                    new_source.include_dirs.retain(|src| !source.include_dirs.contains(src));
                    source.exclude_dirs.extend(new_source.exclude_dirs);
                    source.include_dirs.extend(new_source.include_dirs);
                }
            } else {
                existing.source = spec.source;
            }

            spec.cfg.retain(|cfg| !existing.cfg.contains(cfg));
            existing.cfg.extend(spec.cfg);

            // display_name should match the library's crate name because Rust Analyzer
            // seems to use display_name for matching crate entries in rust-project.json
            // against symbols in source files. For more details, see
            // https://github.com/bazelbuild/rules_rust/issues/1032
            if spec.crate_type == CrateType::Rlib {
                existing.display_name = spec.display_name;
                existing.crate_type = CrateType::Rlib;
                existing.is_test = spec.is_test;
            }

            // We want to use the test target's build label to provide
            // unit tests codelens actions for library crates in IDEs.
            if spec.is_test {
                if let Some(build) = spec.build {
                    existing.build = Some(build);
                }
            }

            // For proc-macro crates that exist within the workspace, there will be a
            // generated crate-spec in both the fastbuild and opt-exec configuration.
            // Prefer proc macro paths with an opt-exec component in the path.
            if let Some(dylib_path) = spec.proc_macro_dylib_path.as_ref() {
                const OPT_PATH_COMPONENT: &str = "-opt-exec-";
                if dylib_path.contains(OPT_PATH_COMPONENT) {
                    existing.proc_macro_dylib_path.replace(dylib_path.clone());
                }
            }
        } else {
            consolidated_specs.insert(spec.crate_id.clone(), spec);
        }
    }

    Ok(consolidated_specs.into_values())
}

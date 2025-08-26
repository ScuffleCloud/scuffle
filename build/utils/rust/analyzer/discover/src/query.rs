use std::collections::{BTreeMap, BTreeSet};

use anyhow::Context;
use camino::{Utf8Path, Utf8PathBuf};

use crate::{bazel_command, deserialize_file_content};

#[derive(Clone, Debug, serde_derive::Deserialize)]
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
    pub cfg: BTreeSet<String>,
    pub env: BTreeMap<String, String>,
    pub target: String,
    pub crate_type: CrateType,
    pub is_test: bool,
    pub build: Option<CrateSpecBuild>,
    pub info: Option<InfoFile>,
}

#[derive(Clone, Debug, serde_derive::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CrateSpecBuild {
    pub label: String,
    pub build_file: String,
}

#[derive(Clone, Debug, serde_derive::Deserialize)]
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

#[derive(serde_derive::Deserialize)]
struct CQueryOutput {
    specs: Vec<Utf8PathBuf>,
    info: Utf8PathBuf,
}

#[derive(Clone, Debug, serde_derive::Deserialize)]
pub struct InfoFile {
    pub id: String,
    pub crate_label: Option<String>,
    pub test_label: Option<String>,
    pub doc_test_label: Option<String>,
    pub clippy_label: Option<String>,
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
    log::info!("running bazel query...");
    log::debug!("Get crate specs with targets: {targets:?}");

    let query_output = bazel_command(bazel, Some(workspace), Some(output_base))
        .args(bazel_startup_options)
        .arg("query")
        .arg(format!(r#"kind("rust_analyzer_info rule", set({}))"#, targets.join(" ")))
        .output()
        .context("bazel query")?;

    if !query_output.status.success() {
        anyhow::bail!("failed to run bazel query: {}", String::from_utf8_lossy(&query_output.stderr))
    }

    let stdout = String::from_utf8_lossy(&query_output.stdout);
    let queried_targets: Vec<_> = stdout.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

    let mut command = bazel_command(bazel, Some(workspace), Some(output_base));
    command
        .args(bazel_startup_options)
        .arg("cquery")
        .args(bazel_args)
        .arg(format!("set({})", queried_targets.join(" ")))
        .arg("--output=starlark")
        .arg(r#"--starlark:expr={"specs":[file.path for file in target.output_groups.rust_analyzer_crate_spec.to_list()],"info":target.output_groups.rust_analyzer_info.to_list()[0].path}"#)
        .arg("--build")
        .arg("--output_groups=rust_analyzer_info,rust_analyzer_proc_macro_dylib,rust_analyzer_src,rust_analyzer_crate_spec");

    log::trace!("Running cquery: {command:#?}");
    let cquery_output = command.output().context("Failed to spawn cquery command")?;

    log::info!("bazel cquery finished; parsing spec files...");

    let mut outputs = Vec::new();
    for line in String::from_utf8_lossy(&cquery_output.stdout)
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
    {
        outputs.push(serde_json::from_str::<CQueryOutput>(line).context("parse line")?);
    }

    let spec_files: BTreeSet<_> = outputs.iter().flat_map(|out| out.specs.iter()).collect();
    let info_files = outputs
        .iter()
        .map(|out| &out.info)
        .map(|file| {
            deserialize_file_content::<InfoFile>(&execution_root.join(file), output_base, workspace, execution_root)
                .map(|file| (file.id.clone(), file))
        })
        .collect::<anyhow::Result<BTreeMap<_, _>>>()
        .context("info file parse")?;

    let crate_specs = spec_files
        .into_iter()
        .map(|file| deserialize_file_content(&execution_root.join(file), output_base, workspace, execution_root))
        .collect::<anyhow::Result<Vec<_>>>()?;

    consolidate_crate_specs(crate_specs, info_files)
}

/// Read all crate specs, deduplicating crates with the same ID. This happens when
/// a rust_test depends on a rust_library, for example.
fn consolidate_crate_specs(
    crate_specs: Vec<CrateSpec>,
    mut infos: BTreeMap<String, InfoFile>,
) -> anyhow::Result<impl IntoIterator<Item = CrateSpec>> {
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

            existing.cfg.extend(spec.cfg);

            // display_name should match the library's crate name because Rust Analyzer
            // seems to use display_name for matching crate entries in rust-project.json
            // against symbols in source files. For more details, see
            // https://github.com/bazelbuild/rules_rust/issues/1032
            if spec.crate_type == CrateType::Rlib {
                existing.display_name = spec.display_name;
                existing.crate_type = CrateType::Rlib;
            }

            // We want to use the test target's build label to provide
            // unit tests codelens actions for library crates in IDEs.
            if spec.is_test {
                existing.is_test = true;
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
            if let Some(info) = infos.remove(&spec.crate_id) {
                spec.info = Some(info);
            }
            consolidated_specs.insert(spec.crate_id.clone(), spec);
        }
    }

    Ok(consolidated_specs.into_values())
}

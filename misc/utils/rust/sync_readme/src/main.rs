//! This is a helper binary which converts a crate + rustdoc json into a
//! nicer markdown for readme generation.
//!
//! We copied a lot of the rustdoc extract code from <https://github.com/gifnksm/cargo-sync-rdme>

use anyhow::Context;
use camino::Utf8PathBuf;
use clap::Parser;

use crate::config::{Metadata, Package};

mod config;
mod content;
mod render;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    cargo_toml: Utf8PathBuf,

    #[arg(long)]
    rustdoc_json: Utf8PathBuf,

    #[arg(long)]
    readme_md: Utf8PathBuf,

    #[arg(long)]
    render_output: Utf8PathBuf,
}

#[derive(Clone, Default, serde_derive::Deserialize)]
struct ManifestMetadata {
    #[serde(alias = "sync-readme")]
    sync_readme: Metadata,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let cargo_toml =
        cargo_toml::Manifest::<ManifestMetadata>::from_path_with_metadata(args.cargo_toml).context("cargo toml")?;
    let pkg = cargo_toml.package();

    let package = Package {
        name: pkg.name.clone(),
        license: pkg.license().map(|l| l.to_owned()),
        version: pkg.version().to_owned(),
        rustdoc_json: args.rustdoc_json,
        metadata: pkg.metadata.clone().unwrap_or_default().sync_readme,
    };

    let content = content::create(&package).context("content")?;
    let readme = std::fs::read_to_string(&args.readme_md).context("readme read")?;
    let render = render::render(&readme, &content).context("render")?;

    let output = serde_json::to_string_pretty(&sync_readme_common::SyncReadmeRenderOutput {
        rendered: render,
        source: readme,
        path: args.readme_md,
    })
    .context("json output")?;

    std::fs::write(&args.render_output, output).context("write output")?;

    Ok(())
}

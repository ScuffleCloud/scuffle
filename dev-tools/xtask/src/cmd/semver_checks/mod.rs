use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use crate::utils::{cargo_cmd, metadata};

mod utils;

use utils::{checkout_baseline, metadata_from_dir, workspace_crates_in_folder};

/// Semver-checks can run in two ways:
/// 1. Provide a baseline git revision branch to compare against, such as `main`:
///    `cargo xtask semver-checks --baseline-rev main`
///
/// 2. Provide a hash to compare against:
///    `cargo xtask semver-checks --baseline-hash some_hash`
///
/// By default, cargo-semver-checks will run against the `main` branch.
#[derive(Debug, Clone, Parser)]
pub struct SemverChecks {
    /// Baseline git revision branch to compare against
    #[clap(long, default_value = "main")]
    baseline_rev: String,

    #[clap(long)]
    /// If provided, explicitly sets the baseline commit hash, skipping lookup
    baseline_hash: Option<String>,
}

impl SemverChecks {
    pub fn run(self) -> Result<()> {
        let current_metadata = metadata().context("current metadata")?;
        let current_crates_set = workspace_crates_in_folder(&current_metadata, "crates");

        let tmp_dir = PathBuf::from("target/semver-baseline");

        match self.baseline_hash {
            Some(hash) => {
                println!("Using explicitly provided commit hash: {}", hash);
                checkout_baseline(&hash, &tmp_dir).context("checking out baseline by hash")?;
            }
            None => {
                checkout_baseline(&self.baseline_rev, &tmp_dir).context("checking out baseline")?;
            }
        };

        let baseline_metadata = metadata_from_dir(&tmp_dir).context("baseline metadata")?;
        let baseline_crates_set = workspace_crates_in_folder(&baseline_metadata, &tmp_dir.join("crates").to_string_lossy());

        let common_crates: HashSet<_> = current_metadata
            .packages
            .iter()
            .map(|p| p.name.clone())
            .filter(|name| current_crates_set.contains(name) && baseline_crates_set.contains(name))
            .collect();

        println!("Semver-checks will run on crates: {:?}", common_crates);

        for package in &common_crates {
            println!("Running semver-checks for {}", package);
            let status = cargo_cmd()
                .args([
                    "semver-checks",
                    "check-release",
                    "--package",
                    package,
                    "--baseline-root",
                    tmp_dir.to_str().unwrap(),
                    "--all-features",
                ])
                .status()
                .context("running semver-checks")?;

            if !status.success() {
                anyhow::bail!("Semver check failed for crate '{}'", package);
            }
        }

        Ok(())
    }
}

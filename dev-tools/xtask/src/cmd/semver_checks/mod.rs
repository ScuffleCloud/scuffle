use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use crate::utils::{cargo_cmd, metadata};

mod utils;

use utils::{checkout_baseline, metadata_from_dir, workspace_crates_in_folder};

/// Semver-checks can run in two ways:
/// 1. Provide a baseline git revision branch to compare against, such as `main`:
///    `cargo xtask semver-checks --baseline main`
///
/// 2. Provide a hash to compare against:
///    `cargo xtask semver-checks --baseline some_hash`
///
/// By default, cargo-semver-checks will run against the `main` branch.
#[derive(Debug, Clone, Parser)]
pub struct SemverChecks {
    /// Baseline git revision branch to compare against
    #[clap(long, default_value = "main")]
    baseline: String,

    #[clap(long, default_value = "false")]
    disable_hakari: bool,
}

impl SemverChecks {
    pub fn run(self) -> Result<()> {
        let current_metadata = metadata().context("current metadata")?;
        let current_crates_set = workspace_crates_in_folder(&current_metadata, "crates");

        let tmp_dir = PathBuf::from("target/semver-baseline");

        // Checkout baseline (auto-cleanup on Drop)
        let _worktree_cleanup = checkout_baseline(&self.baseline, &tmp_dir).context("checking out baseline")?;

        let baseline_metadata = metadata_from_dir(&tmp_dir).context("baseline metadata")?;
        let baseline_crates_set = workspace_crates_in_folder(&baseline_metadata, &tmp_dir.join("crates").to_string_lossy());

        let common_crates: HashSet<_> = current_metadata
            .packages
            .iter()
            .map(|p| p.name.clone())
            .filter(|name| current_crates_set.contains(name) && baseline_crates_set.contains(name))
            .collect();

        println!("Semver-checks will run on crates: {:?}", common_crates);

        if self.disable_hakari {
            println!("Disabling hakari");
            cargo_cmd().args(["hakari", "disable"]).status().context("disabling hakari")?;
        }

        let mut args = vec![
            "semver-checks",
            "check-release",
            "--baseline-root",
            tmp_dir.to_str().unwrap(),
            "--all-features",
        ];

        for package in &common_crates {
            args.push("--package");
            args.push(package);
        }

        let status = cargo_cmd().args(&args).status().context("running semver-checks")?;

        if !status.success() {
            anyhow::bail!("Semver checks failed for one or more crates");
        }

        Ok(())
    }
}

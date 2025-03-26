use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use console::Term;
use regex::Regex;

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

        let output = cargo_cmd().args(&args)
            .output()
            .context("running semver-checks")?;

        // Combine both stdout and stderr.
        let mut semver_output = String::new();
        semver_output.push_str(&String::from_utf8_lossy(&output.stdout));
        semver_output.push_str(&String::from_utf8_lossy(&output.stderr));

        let stdout_term = Term::stdout();

        // If there's no output, warn the user.
        if semver_output.trim().is_empty() {
            stdout_term.write_line("No semver-checks output received. The command may have failed.")?;
        } else {
            // Regex to capture "Checking" lines in two formats:
            // 1. "Checking <crate> vX.Y.Z (current)"
            // 2. "Checking <crate> vX.Y.Z -> vX.Y.Z (no change)"
            let check_re = Regex::new(
                r"^Checking\s+(?P<crate>[^\s]+)\s+v(?P<curr>\d+\.\d+\.\d+)(?:\s+->\s+v(?P<baseline>\d+\.\d+\.\d+))?\s+\((?P<status>[^)]+)\)"
            )
            .context("compiling checking regex")?;

            // Regex for a summary line that indicates an update is required.
            // Example: "Summary semver requires new major version: 1 major and 0 minor checks failed"
            let summary_re = Regex::new(
                r"^Summary semver requires new (?P<update_type>major|minor) version:"
            )
            .context("compiling summary regex")?;

            let mut current_crate: Option<(String, String)> = None;

            // Process output line-by-line.
            for line in semver_output.lines() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("Checking") {
                    // Try to capture crate name and version.
                    if let Some(caps) = check_re.captures(line) {
                        let crate_name = caps.name("crate").unwrap().as_str().to_string();
                        let current_version = caps.name("curr").unwrap().as_str().to_string();
                        current_crate = Some((crate_name, current_version));
                    }
                    stdout_term.write_line(line)?;
                } else if trimmed.starts_with("Checked") {
                    stdout_term.write_line(line)?;
                } else if let Some(caps) = summary_re.captures(line) {
                    let update_type = caps.name("update_type").unwrap().as_str();
                    if let Some((crate_name, current_version)) = current_crate.take() {
                        let new_version = Self::new_version_number(&current_version, update_type)
                            .with_context(|| {
                                format!(
                                    "bumping version for crate {} with update_type {}",
                                    crate_name, update_type
                                )
                            })?;
                        stdout_term.write_line(&format!("⚠️ -> {} update required for `{}`.", update_type, crate_name))?;
                        stdout_term.write_line(&format!("🛠️ -> Please update the version from {} to {}.", current_version, new_version))?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Bumps the version based on the update type.
    ///
    /// For a minor update required, the patch is incremented:
    ///   vX.Y.Z -> vX.Y.(Z+1)
    ///
    /// For a major update required, the minor is incremented:
    ///   vX.Y.Z -> vX.(Y+1).Z
    fn new_version_number(version: &str, update_type: &str) -> Result<String> {
        // Remove leading 'v' if present.
        let version = version.strip_prefix('v').unwrap_or(version);
        let mut parts: Vec<u64> = version
            .split('.')
            .map(|s| s.parse::<u64>())
            .collect::<Result<_, _>>()
            .context("parsing version numbers")?;

        if parts.len() != 3 {
            anyhow::bail!("expected version format vX.Y.Z, got: {}", version);
        }

        match update_type {
            // bump patch version: vX.Y.Z -> vX.Y.(Z+1)
            "minor" => {
                parts[2] += 1;
            }
            // bump minor version: vX.Y.Z -> vX.(Y+1).Z
            "major" => {
                parts[1] += 1;
            }
            _ => {
                anyhow::bail!("Failed to parse update type: {update_type}");
            }
        }

        Ok(format!("v{}.{}.{}", parts[0], parts[1], parts[2]))
    }
}

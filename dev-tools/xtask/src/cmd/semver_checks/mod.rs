use std::collections::HashSet;
use std::iter::Peekable;
use std::path::PathBuf;
use std::process::Stdio;

use anyhow::{Context, Result};
use clap::Parser;
use regex::Regex;

use crate::utils::{cargo_cmd, metadata};

mod utils;
use utils::{checkout_baseline, metadata_from_dir, workspace_crates_in_folder};

#[derive(Debug, Clone, Parser)]
pub struct SemverChecks {
    /// Baseline git revision branch to compare against
    #[clap(long, default_value = "main")]
    baseline: String,

    /// Disable hakari
    #[clap(long, default_value = "false")]
    disable_hakari: bool,
}

impl SemverChecks {
    pub fn run(self) -> Result<()> {
        let current_metadata = metadata().context("getting current metadata")?;
        let current_crates_set = workspace_crates_in_folder(&current_metadata, "crates");

        let tmp_dir = PathBuf::from("target/semver-baseline");

        // Checkout baseline (auto-cleanup on Drop)
        let _worktree_cleanup = checkout_baseline(&self.baseline, &tmp_dir).context("checking out baseline")?;

        let baseline_metadata = metadata_from_dir(&tmp_dir).context("getting baseline metadata")?;
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

        let output = cargo_cmd()
            .env("CARGO_TERM_COLOR", "never")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .context("running semver-checks")?;

        let mut semver_output = String::new();
        semver_output.push_str(&String::from_utf8_lossy(&output.stdout));
        semver_output.push_str(&String::from_utf8_lossy(&output.stderr));

        if semver_output.trim().is_empty() {
            anyhow::bail!("No semver-checks output received. The command may have failed.");
        }

        process_semver_output(&semver_output)?;

        Ok(())
    }
}

fn process_semver_output(output: &str) -> Result<()> {
    // Regex to capture "Checking" lines (ignoring leading whitespace).
    // Supports both formats:
    //   "Checking <crate> vX.Y.Z (current)"
    //   "Checking <crate> vX.Y.Z -> vX.Y.Z (no change)"
    let check_re = Regex::new(r"^\s*Checking\s+(?P<crate>\S+)\s+v(?P<curr>\d+\.\d+\.\d+)(?:\s+->\s+v\d+\.\d+\.\d+)?")
        .context("compiling check regex")?;

    // Regex for summary lines that indicate an update is required.
    // Example:
    //   "Summary semver requires new major version: 1 major and 0 minor checks failed"
    let summary_re = Regex::new(r"^\s*Summary semver requires new (?P<update_type>major|minor) version:")
        .context("compiling summary regex")?;

    let mut current_crate: Option<(String, String)> = None;
    let mut summary_errors: Vec<String> = Vec::new();

    let mut lines = output.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("Checking") {
            // Capture crate name and version without printing.
            if let Some(caps) = check_re.captures(line) {
                let crate_name = caps.name("crate").unwrap().as_str().to_string();
                let current_version = caps.name("curr").unwrap().as_str().to_string();
                current_crate = Some((crate_name, current_version));
            }
        } else if trimmed.starts_with("Summary") {
            // If summary indicates an update, capture details.
            if let Some(caps) = summary_re.captures(line) {
                let update_type = caps.name("update_type").unwrap().as_str();
                if let Some((crate_name, current_version)) = current_crate.take() {
                    let new_version = new_version_number(&current_version, update_type).with_context(|| {
                        format!("bumping version for crate {} with update_type {}", crate_name, update_type)
                    })?;
                    summary_errors.push(format!(
                        "⚠️ -> {update_type} update required for `{crate_name}`.\n🛠️ -> Please update the version from {current_version} to {new_version}."
                    ));
                }
            }
        } else if trimmed.starts_with("---") {
            println!("{}", line);
            process_failure_block(&mut lines)?;
        }
    }

    // Print deferred update messages.
    if !summary_errors.is_empty() {
        println!("\n--- ERRORS FOUND ---\n");
        for error in summary_errors {
            println!("{}", error);
        }
    } else {
        println!("\nNo errors found!\n");
    }

    Ok(())
}

// Process failure blocks by printing lines until a new section is encountered.
fn process_failure_block<'a>(lines: &mut Peekable<impl Iterator<Item = &'a str>>) -> Result<()> {
    // Continue printing lines until a new section is encountered.
    let mut i = 0;
    while let Some(&next_line) = lines.peek() {
        let trimmed = next_line.trim_start();
        if trimmed.starts_with("Checking")
            || trimmed.starts_with("Built")
            || trimmed.starts_with("Parsing")
            || trimmed.starts_with("Finished")
            || trimmed.starts_with("Summary")
        {
            break;
        }
        i += 1;
        println!("{}, {}", lines.next().unwrap(), i);
    }
    Ok(())
}

fn new_version_number(version: &str, update_type: &str) -> Result<String> {
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
        "minor" => parts[2] += 1,
        "major" => parts[1] += 1,
        _ => anyhow::bail!("Failed to parse update type: {update_type}"),
    }
    Ok(format!("v{}.{}.{}", parts[0], parts[1], parts[2]))
}

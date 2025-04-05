use std::collections::HashSet;
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

        // for package in &common_crates {
        //     args.push("--package");
        //     args.push(package);
        // }
        args.push("--package");
        args.push("scuffle-h264");

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

        // empty print to separate from "info: contents updated"
        println!();
        process_semver_output(&semver_output)?;

        Ok(())
    }
}

fn process_semver_output(output: &str) -> Result<()> {
    // Regex to capture "Checking" lines (ignoring leading whitespace).
    // Supports both formats:
    //   "Checking <crate> vX.Y.Z (current)"
    //   "Checking <crate> vX.Y.Z -> vX.Y.Z (no change)"
    let check_re = Regex::new(r"^Checking\s+(?P<crate>\S+)\s+v(?P<curr>\d+\.\d+\.\d+)(?:\s+->\s+v\d+\.\d+\.\d+)?")
        .context("compiling check regex")?;

    // Regex for summary lines that indicate an update is required.
    // Example:
    //   "Summary semver requires new major version: 1 major and 0 minor checks failed"
    let summary_re = Regex::new(r"^Summary semver requires new (?P<update_type>major|minor) version:")
        .context("compiling summary regex")?;

    let mut current_crate: Option<(String, String)> = None;
    let mut summary: Vec<String> = Vec::new();
    let mut error_count = 0;

    let mut lines = output.lines().peekable();
    while let Some(line) = lines.next() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("Checking") {
            // Capture crate name and version without printing.
            if let Some(caps) = check_re.captures(trimmed) {
                let crate_name = caps.name("crate").unwrap().as_str().to_string();
                let current_version = caps.name("curr").unwrap().as_str().to_string();
                current_crate = Some((crate_name, current_version));
            }
        } else if trimmed.starts_with("Summary") {
            if let Some(caps) = summary_re.captures(trimmed) {
                let update_type = caps.name("update_type").unwrap().as_str();
                if let Some((crate_name, current_version)) = current_crate.take() {
                    let new_version = new_version_number(&current_version, update_type)?;
                    summary.push(format!("⚠️ -> {} update required for `{}`.", update_type, crate_name));
                    summary.push(format!(
                        "🛠️ -> Please update the version from {} to {}.\n",
                        current_version, new_version
                    ));
                    error_count += 1;

                    let mut i = 0;

                    // Capture summary description
                    'outer: loop {
                        let next_trimmed = lines.peek().unwrap().trim_start();
                        summary.push(format!("test1: {next_trimmed}")); // this should have "---"

                        if next_trimmed.starts_with("---") {
                            lines.next(); // consume the line with '---'

                            while let Some(&desc_line) = lines.peek() {
                                let desc_trimmed = desc_line.trim_start();
                                summary.push(format!("test2: {desc_trimmed}"));

                                if desc_trimmed.starts_with("Checking")
                                    || desc_trimmed.starts_with("Built")
                                    || desc_trimmed.starts_with("Building")
                                    || desc_trimmed.starts_with("Parsing")
                                    || desc_trimmed.starts_with("Parsed")
                                    || desc_trimmed.starts_with("Finished")
                                    || desc_trimmed.starts_with("Summary")
                                {
                                    break 'outer;
                                }
                                summary.push(desc_trimmed.into());
                            }
                        }

                        i += 1;

                        if i > 10 {
                            break;
                        }
                    }
                }
            }
        }
    }

    // Print deferred update and failure block messages.
    if error_count > 0 {
        println!("🚩 --- {} ERROR(S) FOUND --- 🚩", error_count);

        for line in summary {
            println!("{}", line);
        }
    } else {
        println!("✅ No errors found! ✅");
    }

    // print an empty line to separate output from worktree cleanup line
    println!();
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

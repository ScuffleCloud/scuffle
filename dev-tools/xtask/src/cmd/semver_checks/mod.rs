use std::collections::HashSet;
use std::io::Read;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use next_version::NextVersion;
use regex::Regex;
use semver::Version;

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
        println!("<details>");
        println!("<summary> 🛫 Startup details 🛫 </summary>");
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

        let mut crates: Vec<_> = common_crates.iter().cloned().collect();
        crates.sort();

        println!("<details>");
        // need an extra empty line for the bullet list to format correctly
        println!("<summary> 📦 Processing crates 📦 </summary>\n");
        for krate in crates {
            println!("- `{krate}`");
        }
        // close crate details
        println!("</details>");

        if self.disable_hakari {
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

        let mut command = cargo_cmd();
        command.env("CARGO_TERM_COLOR", "never");
        command.args(&args);

        let (mut reader, writer) = os_pipe::pipe()?;
        let writer_clone = writer.try_clone()?;
        command.stdout(writer);
        command.stderr(writer_clone);

        let mut handle = command.spawn()?;

        drop(command);

        let mut semver_output = String::new();
        reader.read_to_string(&mut semver_output)?;
        handle.wait()?;

        if semver_output.trim().is_empty() {
            anyhow::bail!("No semver-checks output received. The command may have failed.");
        }

        // save the original output for debugging purposes
        println!("<details>");
        println!("<summary> Original semver output: </summary>\n");
        for line in semver_output.lines() {
            println!("{line}");
        }
        println!("</details>");

        // close startup details
        // extra line to separate from startup details
        println!("</details>\n");

        // Regex for summary lines that indicate an update is required.
        // Example:
        //   "Summary semver requires new major version: 1 major and 0 minor checks failed"
        let summary_re = Regex::new(r"^Summary semver requires new (?P<update_type>major|minor) version:")
            .context("compiling summary regex")?;

        let commit_hash = std::env::var("SHA")?;
        let scuffle_commit_url = format!("https://github.com/ScuffleCloud/scuffle/blob/{commit_hash}");

        let mut current_crate: Option<(String, String)> = None;
        let mut summary: Vec<String> = Vec::new();
        let mut description: Vec<String> = Vec::new();
        let mut error_count = 0;

        let mut lines = semver_output.lines().peekable();
        while let Some(line) = lines.next() {
            let trimmed = line.trim_start();

            if trimmed.starts_with("Checking") {
                // example line: Checking nutype-enum v0.1.2 -> v0.1.2 (no change)
                // sometimes the (no change) part is missing if the crate has already been updated.
                let split_line = trimmed.split_whitespace().collect::<Vec<_>>();
                current_crate = Some((split_line[1].to_string(), split_line[2].to_string()));
            } else if trimmed.starts_with("Summary") {
                if let Some(summary_line) = summary_re.captures(trimmed) {
                    let (crate_name, current_version_str) = current_crate.take().unwrap();
                    let update_type = summary_line.name("update_type").unwrap().as_str();
                    let new_version = new_version_number(&current_version_str, update_type)?;

                    // capitalize first letter of update_type
                    let update_type = format!("{}{}", update_type.chars().next().unwrap().to_uppercase(), &update_type[1..]);
                    error_count += 1;

                    // need to escape the #{error_count} otherwise it will refer to an actual pr
                    summary.push(format!("### 🔖 Error `#{error_count}`"));
                    summary.push(format!("{update_type} update required for `{crate_name}` ⚠️"));
                    summary.push(format!(
                        "Please update the version from `{current_version_str}` to `v{new_version}` 🛠️"
                    ));

                    summary.push("<details>".to_string());
                    summary.push(format!("<summary> 📜 {crate_name} logs 📜 </summary>\n"));
                    summary.append(&mut description);
                    summary.push("</details>".to_string());

                    // add a new line after the description
                    summary.push("".to_string());
                }
            } else if trimmed.starts_with("---") {
                let mut is_failed_in_block = false;

                while let Some(desc_line) = lines.peek() {
                    let desc_trimmed = desc_line.trim_start();

                    if desc_trimmed.starts_with("Summary") {
                        // sometimes an empty new line isn't detected before the description ends
                        // in that case, add a closing `</details>` for the "Failed in" block.
                        if is_failed_in_block {
                            description.push("</details>".to_string());
                        }
                        break;
                    } else if desc_trimmed.starts_with("Failed in:") {
                        // create detail block for "Failed in" block
                        is_failed_in_block = true;
                        description.push("<details>".to_string());
                        description.push("<summary> 🎈 Failed in the following locations 🎈 </summary>".to_string());
                    } else if desc_trimmed.is_empty() && is_failed_in_block {
                        // close detail close for "Failed in" block
                        is_failed_in_block = false;
                        description.push("</details>".to_string());
                    } else if is_failed_in_block {
                        // need new line to allow for bullet list formatting
                        description.push("".to_string());

                        let file_loc = desc_trimmed
                            .split_whitespace()
                            .last()
                            .unwrap()
                            .strip_prefix("/home/runner/work/scuffle/scuffle/")
                            .unwrap()
                            .replace(":", "#L");

                        description.push(format!("- {scuffle_commit_url}/{file_loc}"));
                    } else {
                        description.push(desc_trimmed.to_string());
                    }

                    lines.next();
                }
            }
        }

        // Print deferred update and failure block messages.
        println!("# Semver-checks summary");
        if error_count > 0 {
            let s = if error_count == 1 { "" } else { "S" };
            println!("\n### 🚩 {error_count} ERROR{s} FOUND 🚩");

            // if there are 5+ errors, shrink the details by default.
            if error_count >= 5 {
                summary.insert(0, "<details>".to_string());
                summary.insert(1, "<summary> 🦗 Open for error description 🦗 </summary>".to_string());
                summary.push("</details>".to_string());
            }

            for line in summary {
                println!("{line}");
            }
        } else {
            println!("## ✅ No semver violations found! ✅");
        }

        Ok(())
    }
}

fn new_version_number(crate_version: &str, update_type: &str) -> Result<Version> {
    let update_is_major = update_type.eq_ignore_ascii_case("major");

    let version_stripped = crate_version.strip_prefix('v').unwrap();
    let version_parsed = Version::parse(version_stripped)?;

    let bumped = if update_is_major {
        major_update(&version_parsed)
    } else {
        minor_update(&version_parsed)
    };

    Ok(bumped)
}

fn major_update(current_version: &Version) -> Version {
    if !current_version.pre.is_empty() {
        current_version.increment_prerelease()
    } else if current_version.major == 0 && current_version.minor == 0 {
        current_version.increment_patch()
    } else if current_version.major == 0 {
        current_version.increment_minor()
    } else {
        current_version.increment_major()
    }
}

fn minor_update(current_version: &Version) -> Version {
    if !current_version.pre.is_empty() {
        current_version.increment_prerelease()
    } else if current_version.major == 0 {
        current_version.increment_minor()
    } else {
        current_version.increment_patch()
    }
}

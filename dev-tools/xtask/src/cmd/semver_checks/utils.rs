use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use cargo_metadata::{Metadata, MetadataCommand};

pub struct WorktreeCleanup {
    path: PathBuf,
}

impl Drop for WorktreeCleanup {
    fn drop(&mut self) {
        println!("Cleaning up git worktree at {:?}", self.path);
        let status = Command::new("git")
            .args(["worktree", "remove", "--force", self.path.to_str().unwrap()])
            .status();

        match status {
            Ok(status) if status.success() => {
                println!("Successfully removed git worktree");
            }
            Ok(status) => {
                eprintln!("Failed to remove git worktree. Exit code: {}", status);
            }
            Err(e) => {
                eprintln!("Error removing git worktree: {:?}", e);
            }
        }
    }
}

pub fn metadata_from_dir(dir: impl AsRef<Path>) -> Result<Metadata> {
    MetadataCommand::new()
        .manifest_path(dir.as_ref().join("Cargo.toml"))
        .exec()
        .context("fetching cargo metadata from directory")
}

pub fn checkout_baseline(baseline_rev_or_hash: &str, target_dir: &PathBuf) -> Result<WorktreeCleanup> {
    if target_dir.exists() {
        std::fs::remove_dir_all(target_dir)?;
    }

    // Attempt to resolve the revision locally first
    let rev_parse_output = Command::new("git")
        .args(["rev-parse", "--verify", baseline_rev_or_hash])
        .output()
        .context("git rev-parse failed")?;

    let commit_hash = if rev_parse_output.status.success() {
        String::from_utf8(rev_parse_output.stdout)?.trim().to_string()
    } else {
        println!("Revision {} not found locally. Fetching from origin...", baseline_rev_or_hash);

        Command::new("git")
            .args(["fetch", "--depth", "1", "origin", baseline_rev_or_hash])
            .status()
            .context("git fetch failed")?
            .success()
            .then_some(())
            .context("git fetch unsuccessful")?;

        // Retry resolving after fetch
        let retry_output = Command::new("git")
            .args(["rev-parse", "--verify", "FETCH_HEAD"])
            .output()
            .context("git rev-parse after fetch failed")?;

        retry_output
            .status
            .success()
            .then(|| String::from_utf8(retry_output.stdout).unwrap().trim().to_string())
            .context(format!("Failed to resolve revision {}", baseline_rev_or_hash))?
    };

    println!("Checking out commit {} into {:?}", commit_hash, target_dir);

    Command::new("git")
        .args(["worktree", "add", "--detach", target_dir.to_str().unwrap(), &commit_hash])
        .status()
        .context("git worktree add failed")?
        .success()
        .then_some(())
        .context("git worktree add unsuccessful")?;

    Ok(WorktreeCleanup {
        path: target_dir.clone(),
    })
}

pub fn workspace_crates_in_folder(meta: &Metadata, folder: &str) -> HashSet<String> {
    let folder_path = std::fs::canonicalize(folder).expect("folder should exist");

    meta.packages
        .iter()
        .filter(|p| {
            // All crate examples have publish = false.
            // The scuffle-bootstrap-derive crate doesn't work with the semver-checks tool at the moment.
            let manifest_path = p.manifest_path.parent().unwrap();
            manifest_path.starts_with(&folder_path)
                && p.publish.as_ref().map(|v| !v.is_empty()).unwrap_or(true)
                && p.name != "scuffle-bootstrap-derive"
                && p.name != "scuffle-metrics-derive"
        })
        .map(|p| p.name.clone())
        .collect()
}

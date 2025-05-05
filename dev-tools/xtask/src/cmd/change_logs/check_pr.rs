use std::collections::HashSet;
use std::path::PathBuf;

use anyhow::Context;

use super::util::Fragment;
use crate::cmd::IGNORED_PACKAGES;
use crate::cmd::change_logs::util::package_to_logs;

#[derive(Debug, Clone, clap::Parser)]
pub struct CheckPr {
    /// The PR number to check
    pr_number: u64,
    #[clap(long, default_value = "true", action = clap::ArgAction::Set)]
    required: bool,
}

impl CheckPr {
    pub fn run(self) -> anyhow::Result<()> {
        let start = std::time::Instant::now();

        let metadata = crate::utils::metadata()?;

        let workspace_package_ids = metadata.workspace_members.iter().cloned().collect::<HashSet<_>>();

        let path = metadata
            .workspace_root
            .join("changes.d")
            .join(format!("pr-{}.toml", self.pr_number));

        eprintln!("checking {path}");

        if !self.required && !path.exists() {
            return Ok(());
        }

        let fragment = Fragment::new(self.pr_number, &PathBuf::from(path))?;

        let workspace_package_names = metadata
            .packages
            .iter()
            .filter(|p| workspace_package_ids.contains(&p.id) && !IGNORED_PACKAGES.contains(&p.name.as_str()))
            .map(|p| p.name.as_str())
            .collect::<HashSet<_>>();

        let mut has_logs = false;

        for (package, item) in fragment.packages() {
            anyhow::ensure!(
                workspace_package_names.contains(package),
                "package `{}` is not in the workspace",
                package
            );

            let logs = package_to_logs(self.pr_number, item.clone()).context("parse")?;

            anyhow::ensure!(!logs.is_empty(), "no change logs found for package `{}`", package);

            has_logs = true;
        }

        anyhow::ensure!(has_logs, "no change logs found for any package");

        eprintln!("Done in {:?}", start.elapsed());

        Ok(())
    }
}

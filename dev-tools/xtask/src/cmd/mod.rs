use anyhow::Context;

mod change_logs;
mod dev_tools;
mod power_set;
mod semver_checks;
mod workspace_deps;

const IGNORED_PACKAGES: &[&str] = &["scuffle-workspace-hack", "xtask"];

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Commands {
    #[clap(alias = "powerset")]
    PowerSet(power_set::PowerSet),
    WorkspaceDeps(workspace_deps::WorkspaceDeps),
    #[clap(alias = "change-log", subcommand)]
    ChangeLogs(change_logs::Commands),
    DevTools(dev_tools::DevTools),
    SemverChecks(semver_checks::SemverChecks),
}

impl Commands {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Commands::PowerSet(cmd) => cmd.run().context("power set"),
            Commands::WorkspaceDeps(cmd) => cmd.run().context("workspace deps"),
            Commands::ChangeLogs(cmd) => cmd.run().context("change logs"),
            Commands::DevTools(cmd) => cmd.run().context("dev tools"),
            Commands::SemverChecks(cmd) => cmd.run().context("semver checks"),
        }
    }
}

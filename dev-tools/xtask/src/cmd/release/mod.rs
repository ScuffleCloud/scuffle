use anyhow::Context;

mod check;
mod publish;
mod update;
mod utils;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Commands {
    /// Update manifests & changelogs for packages that have been changed
    Update(update::Update),
    /// Find and report errors in packages
    Check(check::Check),
    /// Publish release
    Publish(publish::Publish),
}

impl Commands {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Commands::Update(cmd) => cmd.run().context("update"),
            Commands::Check(cmd) => cmd.run().context("check"),
            Commands::Publish(cmd) => cmd.run().context("publish"),
        }
    }
}

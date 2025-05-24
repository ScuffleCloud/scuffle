use anyhow::Context;

mod dev_tools;
mod power_set;
mod release;

const IGNORED_PACKAGES: &[&str] = &["scuffle-workspace-hack", "xtask"];

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Commands {
    #[clap(alias = "powerset")]
    PowerSet(power_set::PowerSet),
    DevTools(dev_tools::DevTools),
    #[clap(subcommand)]
    Release(release::Commands),
}

impl Commands {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Commands::PowerSet(cmd) => cmd.run().context("power set"),
            Commands::DevTools(cmd) => cmd.run().context("dev tools"),
            Commands::Release(cmd) => cmd.run().context("release pr"),
        }
    }
}

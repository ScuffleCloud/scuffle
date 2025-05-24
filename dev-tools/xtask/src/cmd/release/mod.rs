use anyhow::Context;

mod check;
mod fix;
mod generate_pr;
mod publish;
mod utils;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Commands {
    /// Generate release pull-request
    GeneratePr(generate_pr::GeneratePr),
    /// Check everything is correct for a merge
    Check(check::Check),
    /// Fix all problems.
    Fix(fix::Fix),
    /// Publish release
    Publish(publish::Publish),
}

impl Commands {
    pub fn run(self) -> anyhow::Result<()> {
        match self {
            Commands::GeneratePr(cmd) => cmd.run().context("pr"),
            Commands::Check(cmd) => cmd.run().context("check"),
            Commands::Fix(cmd) => cmd.run().context("fix"),
            Commands::Publish(cmd) => cmd.run().context("publish"),
        }
    }
}

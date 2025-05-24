use clap::error::ErrorKind;
use clap::{CommandFactory, Parser};
use cmd::Commands;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod cmd;
mod utils;

#[derive(Debug, clap::Parser)]
#[command(
    name = "cargo xtask",
    bin_name = "cargo xtask",
    about = "A utility for running commands in the workspace"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with_writer(std::io::stderr)
        .init();

    if let Err(err) = Cli::parse().command.run() {
        Cli::command()
            .error(ErrorKind::InvalidValue, format!("{err:#}\n\n{err:?}"))
            .exit()
    }
}

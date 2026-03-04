//! UTS Cli
use crate::commands::Commands;
use clap::Parser;

mod client;
mod commands;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    Cli::parse().command.run().await
}

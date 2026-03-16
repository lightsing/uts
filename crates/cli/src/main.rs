//! UTS Cli
use crate::commands::Commands;
use clap::Parser;
use tracing::warn;
use tracing_subscriber::{EnvFilter, filter::LevelFilter};

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

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .without_time()
        .with_target(false)
        .init();

    warn!("UTS is current in TESTING, not for production use.");
    Cli::parse().command.run().await
}

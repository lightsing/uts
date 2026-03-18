use clap::Subcommand;

mod inspect;
mod purge;
mod stamp;
mod upgrade;
mod verify;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Inspect an ots file
    Inspect(inspect::Inspect),
    /// Verify an ots file against a file
    Verify(verify::Verify),
    /// Create timestamp
    Stamp(stamp::Stamp),
    /// Upgrade timestamp
    Upgrade(upgrade::Upgrade),
    /// Purge stale pending attestations
    Purge(purge::Purge),
}

impl Commands {
    pub async fn run(self) -> eyre::Result<()> {
        match self {
            Commands::Inspect(cmd) => cmd.run(),
            Commands::Verify(cmd) => cmd.run().await,
            Commands::Stamp(cmd) => cmd.run().await,
            Commands::Upgrade(cmd) => cmd.run().await,
            Commands::Purge(cmd) => cmd.run().await,
        }
    }
}

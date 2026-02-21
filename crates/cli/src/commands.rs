use clap::Subcommand;

mod inspect;
mod verify;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Inspect an ots file
    Inspect(inspect::Inspect),
    /// Verify an ots file against a file
    Verify(verify::Verify),
}

impl Commands {
    pub async fn run(self) -> eyre::Result<()> {
        match self {
            Commands::Inspect(cmd) => cmd.run(),
            Commands::Verify(cmd) => cmd.run().await,
        }
    }
}

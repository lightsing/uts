use clap::Args;
use std::path::PathBuf;
use tracing::{error, info, warn};
use uts_core::codec::{Decode, Encode, VersionedProof, v1::DetachedTimestamp};
use uts_sdk::Sdk;

#[derive(Debug, Args)]
pub struct Purge {
    /// Files to purge pending attestations from. May be specified multiple times.
    #[arg(value_name = "FILE", num_args = 1..)]
    files: Vec<PathBuf>,
    /// Skip the interactive confirmation prompt and purge immediately.
    #[arg(short = 'y', long = "yes", default_value_t = false)]
    yes: bool,
}

impl Purge {
    pub async fn run(self) -> eyre::Result<()> {
        for path in &self.files {
            if let Err(e) = self.purge_one(path).await {
                error!("[{}] failed to purge: {e}", path.display());
            }
        }
        Ok(())
    }

    async fn purge_one(&self, path: &PathBuf) -> eyre::Result<()> {
        let file = tokio::fs::read(path).await?;
        let mut proof = VersionedProof::<DetachedTimestamp>::decode(&mut &*file)?;

        let pending = Sdk::list_pending(&proof);
        if pending.is_empty() {
            info!("[{}] no pending attestations found, skipping", path.display());
            return Ok(());
        }

        info!(
            "[{}] found {} pending attestation(s):",
            path.display(),
            pending.len()
        );
        for uri in &pending {
            info!("  - {uri}");
        }

        if !self.yes {
            eprint!(
                "Purge {} pending attestation(s) from {}? [y/N] ",
                pending.len(),
                path.display()
            );
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                info!("[{}] skipped", path.display());
                return Ok(());
            }
        }

        let result = Sdk::purge_pending(&mut proof);

        if !result.has_remaining {
            warn!(
                "[{}] all attestations were pending — the file is now empty and should be removed",
                path.display()
            );
            tokio::fs::remove_file(path).await?;
            info!("[{}] removed empty file", path.display());
            return Ok(());
        }

        let mut buf = Vec::new();
        proof.encode(&mut buf)?;
        tokio::fs::write(path, buf).await?;
        info!(
            "[{}] purged {} pending attestation(s)",
            path.display(),
            result.purged.len()
        );
        Ok(())
    }
}

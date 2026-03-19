use clap::Args;
use std::{collections::HashSet, path::PathBuf};
use tracing::{error, info, warn};
use uts_core::codec::{
    Decode, Encode, VersionedProof,
    v1::{Attestation, DetachedTimestamp, PendingAttestation},
};
use uts_sdk::Sdk;

#[derive(Debug, Args)]
pub struct Purge {
    /// Files to purge pending attestations from. May be specified multiple times.
    #[arg(value_name = "FILE", num_args = 1..)]
    files: Vec<PathBuf>,
    /// Skip the interactive confirmation prompt and purge all pending attestations.
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
        let proof = VersionedProof::<DetachedTimestamp>::decode(&mut &*file)?;

        let pending = proof
            .attestations()
            .filter(|att| att.tag == PendingAttestation::TAG)
            .map(|att| PendingAttestation::from_raw(att).map(|p| p.uri))
            .collect::<Result<Vec<_>, _>>()?;
        if pending.is_empty() {
            info!(
                "[{}] no pending attestations found, skipping",
                path.display()
            );
            return Ok(());
        }

        info!(
            "[{}] found {} pending attestation(s):",
            path.display(),
            pending.len()
        );
        for (i, uri) in pending.iter().enumerate() {
            info!("  [{}] {uri}", i + 1);
        }

        let uris_to_purge = if self.yes {
            // Purge all when --yes flag is used
            pending.into_iter().collect()
        } else {
            // Interactive selection
            print!("Enter numbers to purge (comma-separated), 'all', or 'none' to skip: ");
            use std::io::Write;
            std::io::stdout().flush()?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.eq_ignore_ascii_case("none") || input.is_empty() {
                info!("[{}] skipped", path.display());
                return Ok(());
            }

            if input.eq_ignore_ascii_case("all") {
                pending.into_iter().collect()
            } else {
                let mut selected = HashSet::new();
                for part in input.split(',') {
                    let part = part.trim();
                    match part.parse::<usize>() {
                        Ok(n) if n >= 1 && n <= pending.len() => {
                            selected.insert(pending[n - 1].clone());
                        }
                        _ => {
                            warn!("ignoring invalid selection: {part}");
                        }
                    }
                }
                if selected.is_empty() {
                    info!("[{}] no valid selections, skipping", path.display());
                    return Ok(());
                }
                selected
            }
        };

        let Some(result) = Sdk::filter_pending_by_uris(&proof, |uri| uris_to_purge.contains(uri))
        else {
            error!("won't purge [{}], results in empty proof", path.display());
            return Ok(());
        };

        if result.purged.is_empty() {
            info!("[{}] nothing to purge", path.display());
            return Ok(());
        }

        let mut buf = Vec::new();
        VersionedProof::new(result.new_stamp).encode(&mut buf)?;
        tokio::fs::write(path, buf).await?;
        info!(
            "purged {} pending attestation(s) from [{}] ",
            result.purged.len(),
            path.display(),
        );
        Ok(())
    }
}

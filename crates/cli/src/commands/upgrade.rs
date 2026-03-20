use clap::Args;
use std::path::PathBuf;
use tracing::{error, info, warn};
use uts_core::codec::{Decode, Encode, VersionedProof, v1::DetachedTimestamp};
use uts_sdk::{Sdk, UpgradeResult};

#[derive(Debug, Args)]
pub struct Upgrade {
    /// Files to timestamp. May be specified multiple times.
    #[arg(value_name = "FILE", num_args = 1..)]
    files: Vec<PathBuf>,
    /// Whether to keep pending attestations in the proof. Default is false, which means pending
    /// attestations will be removed from the proof once the upgrade process is complete.
    #[arg(short, long, default_value_t = false)]
    keep_pending: bool,
    /// Timeout in seconds to wait for calendar responses.
    #[arg(long = "timeout")]
    timeout: Option<u64>,
}

impl Upgrade {
    pub async fn run(self) -> eyre::Result<()> {
        let mut builder = Sdk::builder();
        if self.keep_pending {
            builder = builder.keep_pending();
        }
        if let Some(timeout) = self.timeout {
            builder = builder.with_timeout_seconds(timeout);
        }
        let sdk = builder.build()?;

        let results = futures::future::join_all(
            self.files
                .iter()
                .map(|path| upgrade_one(sdk.clone(), path.clone())),
        )
        .await
        .into_iter()
        .collect::<Vec<_>>();

        for (path, result) in self.files.iter().zip(results) {
            if let Err(e) = result {
                error!("[{}] failed to upgrade: {e}", path.display())
            }
        }
        Ok(())
    }
}

async fn upgrade_one(sdk: Sdk, path: PathBuf) -> eyre::Result<()> {
    info!("[{}] upgrading attestation...", path.display());
    let file = tokio::fs::read(&path).await?;
    let mut proof = VersionedProof::<DetachedTimestamp>::decode(&mut &*file)?;
    let results = sdk.upgrade(&mut proof).await?;

    let mut changed = false;
    for (calendar_server, result) in results {
        match result {
            UpgradeResult::Upgraded => {
                info!(
                    "[{}] attestation from {calendar_server} upgraded successfully",
                    path.display()
                );
                changed = true;
            }
            UpgradeResult::Pending => info!(
                "[{}] attestation from {calendar_server} is still pending, skipping",
                path.display()
            ),
            UpgradeResult::Failed(e) => warn!(
                "[{}] failed to upgrade attestation from {calendar_server}: {e}",
                path.display()
            ),
        }
    }

    if changed {
        let mut buf = Vec::new();
        proof.encode(&mut buf)?;
        tokio::fs::write(path, buf).await?;
    }
    Ok(())
}

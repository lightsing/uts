use crate::client::CLIENT;
use clap::Args;
use futures::TryFutureExt;
use reqwest::StatusCode;
use std::{fs, future::ready, path::PathBuf, time::Duration};
use tracing::{error, info, warn};
use url::Url;
use uts_core::{
    codec::{
        Decode, Encode, VersionedProof,
        v1::{Attestation, DetachedTimestamp, PendingAttestation, Timestamp},
    },
    utils::Hexed,
};

#[derive(Debug, Args)]
pub struct Upgrade {
    /// Files to timestamp. May be specified multiple times.
    #[arg(value_name = "FILE", num_args = 1..)]
    files: Vec<PathBuf>,
    /// Whether to keep pending attestations in the proof. Default is false, which means pending
    /// attestations will be removed from the proof once the upgrade process is complete.
    #[arg(short, long, default_value_t = false)]
    keep_pending: bool,
    /// Timeout in seconds to wait for calendar responses. Default is 5 seconds.
    #[arg(long = "timeout", default_value = "5")]
    timeout: u64,
}

impl Upgrade {
    pub async fn run(self) -> eyre::Result<()> {
        // timestamp files are small, so we can read them all synchronously before upgrading.
        let files = self
            .files
            .iter()
            .map(fs::read)
            .collect::<Result<Vec<_>, _>>()?;

        let results = futures::future::join_all(
            self.files
                .iter()
                .cloned()
                .zip(files)
                .map(|(path, file)| upgrade_one(path, file, self.keep_pending, self.timeout)),
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

async fn upgrade_one(
    path: PathBuf,
    file: Vec<u8>,
    keep_pending: bool,
    timeout: u64,
) -> eyre::Result<()> {
    let mut proof = VersionedProof::<DetachedTimestamp>::decode(&mut &*file)?;

    for step in proof.proof.pending_attestations_mut() {
        let (calendar_server, retrieve_uri) = {
            let Timestamp::Attestation(attestation) = step else {
                unreachable!("bug: PendingAttestationIterMut should only yield Attestations");
            };
            let commitment = attestation.value().expect("finalized when decode");
            let calendar_server = PendingAttestation::from_raw(&*attestation)?.uri;

            let retrieve_uri =
                Url::parse(&calendar_server)?.join(&format!("timestamp/{}", Hexed(commitment)))?;

            (calendar_server, retrieve_uri)
        };

        let result = CLIENT
            .get(retrieve_uri)
            .header("Accept", "application/vnd.opentimestamps.v1")
            .timeout(Duration::from_secs(timeout))
            .send()
            .and_then(|r| ready(r.error_for_status()))
            .and_then(|r| r.bytes())
            .await;

        match result {
            Ok(response) => {
                let attestation = Timestamp::decode(&mut &*response)?;
                info!(
                    "[{}] successfully upgraded pending attestation from {calendar_server}",
                    path.display()
                );

                *step = if keep_pending {
                    Timestamp::merge(uts_core::alloc::vec![attestation, step.clone()])
                } else {
                    attestation
                };
            }
            Err(e) => {
                if let Some(status) = e.status()
                    && status == StatusCode::NOT_FOUND
                {
                    // calendar not ready yet
                    info!(
                        "[{}] attestation from {calendar_server} not ready yet, skipping",
                        path.display()
                    );
                    continue;
                }
                warn!(
                    "[{}] failed to upgrade pending attestation from {calendar_server}: {e}",
                    path.display()
                );
                continue;
            }
        }
    }

    let mut buf = Vec::new();
    proof.encode(&mut buf)?;
    tokio::fs::write(path, buf).await?;

    Ok(())
}

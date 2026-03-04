use crate::client::CLIENT;
use clap::Args;
use eyre::bail;
use futures::TryFutureExt;
use reqwest::StatusCode;
use std::{fs, future::ready, path::PathBuf, time::Duration};
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
            .map(|path| fs::read(path))
            .collect::<Result<Vec<_>, _>>()?;

        let results = futures::future::join_all(
            self.files
                .iter()
                .cloned()
                .zip(files)
                .into_iter()
                .map(|(path, file)| upgrade_one(path, file, self.timeout)),
        )
        .await
        .into_iter()
        .collect::<Vec<_>>();
        for (path, result) in self.files.iter().zip(results) {
            match result {
                Ok(_) => eprintln!("Upgraded: {}", path.display()),
                Err(e) => eprintln!("Failed to upgrade {}: {e}", path.display()),
            }
        }
        Ok(())
    }
}

async fn upgrade_one(path: PathBuf, file: Vec<u8>, timeout: u64) -> eyre::Result<()> {
    let mut proof = VersionedProof::<DetachedTimestamp>::decode(&mut &*file)?;

    for step in proof.proof.pending_attestations_mut() {
        let pending_uri = {
            let Timestamp::Attestation(attestation) = step else {
                unreachable!("bug: PendingAttestationIterMut should only yield Attestations");
            };
            let commitment = attestation.value().expect("finalized when decode");
            let pending_uri = PendingAttestation::from_raw(&*attestation)?.uri;
            Url::parse(&pending_uri)?.join(&format!("timestamp/{}", Hexed(commitment)))?
        };

        let result = CLIENT
            .get(pending_uri)
            .header("Accept", "application/vnd.opentimestamps.v1")
            .timeout(Duration::from_secs(timeout))
            .send()
            .and_then(|r| ready(r.error_for_status()))
            .and_then(|r| r.bytes())
            .await;

        match result {
            Ok(response) => {
                let attestation = Timestamp::decode(&mut &*response)?;
                *step = Timestamp::merge(vec![attestation, step.clone()])
            }
            Err(e) => {
                if let Some(status) = e.status()
                    && status == StatusCode::NOT_FOUND
                {
                    bail!("calendar not ready yet.");
                }
                return Err(e.into());
            }
        }
    }

    let mut buf = Vec::new();
    proof.encode(&mut buf)?;
    tokio::fs::write(path, buf).await?;

    Ok(())
}

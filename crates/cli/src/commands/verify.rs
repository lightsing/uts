use clap::Args;
use std::{fs::File, path::PathBuf};
use tracing::info;
use uts_core::codec::{Decode, Reader, VersionedProof, v1::DetachedTimestamp};
use uts_sdk::Sdk;

#[derive(Debug, Args)]
pub struct Verify {
    file: PathBuf,
    stamp_file: Option<PathBuf>,
}

impl Verify {
    pub async fn run(self) -> eyre::Result<()> {
        let sdk = Sdk::new();

        let stamp_file = self.stamp_file.unwrap_or_else(|| {
            let mut default = self.file.clone();
            default.add_extension("ots");
            default
        });
        let timestamp =
            VersionedProof::<DetachedTimestamp>::decode(&mut Reader(File::open(&stamp_file)?))?
                .proof;
        let results = sdk.verify(&self.file, &timestamp).await?;

        for result in results.iter() {
            info!(
                "Attestation: {}, Status: {:?}",
                result.attestation, result.status
            );
        }

        let overall = sdk.aggregate_verify_results(&results);
        info!("Overall verification result: {overall:?}");
        Ok(())
    }
}

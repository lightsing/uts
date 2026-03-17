use crate::{Result, Sdk, error::Error};
use http::{Method, StatusCode};
use std::collections::BTreeMap;
use tracing::{debug, warn};
use url::Url;
use uts_core::{
    alloc::Allocator,
    codec::{
        DecodeIn,
        v1::{Attestation, DetachedTimestamp, PendingAttestation, Timestamp},
    },
    utils::Hexed,
};

/// Result of attempting to upgrade a pending attestation.
#[derive(Debug)]
pub enum UpgradeResult {
    /// The attestation has been successfully upgraded.
    Upgraded,
    /// The attestation is still pending and not ready to be upgraded.
    Pending,
    /// The attestation upgrade failed due to an error.
    Failed(Error),
}

impl Sdk {
    /// Upgrades all pending attestations in the given detached timestamp.
    pub async fn upgrade<A: Allocator + Clone>(
        &self,
        stamp: &mut DetachedTimestamp<A>,
    ) -> Result<BTreeMap<String, UpgradeResult>> {
        let mut results = BTreeMap::new();

        let alloc = stamp.allocator().clone();
        for step in stamp.pending_attestations_mut() {
            let (calendar_server, retrieve_uri) = {
                let Timestamp::Attestation(attestation) = step else {
                    unreachable!("bug: PendingAttestationIterMut should only yield Attestations");
                };
                let commitment = attestation.value().expect("finalized when decode");
                let calendar_server = PendingAttestation::from_raw(&*attestation)?.uri;

                let retrieve_uri = Url::parse(&calendar_server)?
                    .join(&format!("timestamp/{}", Hexed(commitment)))?;

                (calendar_server.to_string(), retrieve_uri)
            };

            let result = self
                .http_request_with_retry(
                    Method::GET,
                    retrieve_uri,
                    10 * 1024 * 1024, // 10 MiB response size limit
                    |req| req.header("Accept", "application/vnd.opentimestamps.v1"),
                )
                .await;

            let result = match result {
                Ok((parts, _)) if parts.status == StatusCode::NOT_FOUND => {
                    debug!("attestation from {calendar_server} not ready yet, skipping");
                    UpgradeResult::Pending
                }
                Ok((_, response)) => {
                    let attestation = Timestamp::decode_in(&mut &*response, alloc.clone())?;

                    *step = if self.inner.keep_pending {
                        Timestamp::merge_in(
                            uts_core::alloc::vec![in alloc.clone(); attestation, step.clone()],
                            alloc.clone(),
                        )
                    } else {
                        attestation
                    };
                    UpgradeResult::Upgraded
                }
                Err(e) => {
                    warn!("failed to upgrade pending attestation from {calendar_server}: {e}");
                    UpgradeResult::Failed(e)
                }
            };

            if let Some(old) = results.insert(calendar_server.clone(), result) {
                warn!(
                    "multiple pending attestations from {calendar_server}, previous result was {old:?}, you should only attest to a calendar once per timestamp"
                );
            }
        }
        Ok(results)
    }
}

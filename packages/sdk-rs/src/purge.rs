use tracing::info;
use uts_core::{
    alloc::Allocator,
    codec::v1::{Attestation, DetachedTimestamp, PendingAttestation},
};

use crate::Sdk;

/// Result of a purge operation on a detached timestamp.
#[derive(Debug)]
pub struct PurgeResult {
    /// URIs of the pending attestations that were purged.
    pub purged: Vec<String>,
    /// Whether the timestamp still has any attestations remaining.
    pub has_remaining: bool,
}

impl Sdk {
    /// Lists all pending attestation URIs in the given detached timestamp.
    pub fn list_pending<A: Allocator>(
        stamp: &DetachedTimestamp<A>,
    ) -> Vec<String> {
        stamp
            .attestations()
            .filter_map(|att| {
                PendingAttestation::from_raw(att)
                    .ok()
                    .map(|p| p.uri.to_string())
            })
            .collect()
    }

    /// Purges all pending attestations from the given detached timestamp.
    ///
    /// Returns a [`PurgeResult`] containing the URIs of purged attestations
    /// and whether the timestamp still has remaining (non-pending) attestations.
    ///
    /// If all attestations were pending, the timestamp becomes invalid and
    /// `has_remaining` will be `false` — callers should handle this case
    /// (e.g., by deleting the file).
    pub fn purge_pending<A: Allocator>(
        stamp: &mut DetachedTimestamp<A>,
    ) -> PurgeResult {
        let pending_uris = Self::list_pending(stamp);
        let count = pending_uris.len();

        if count == 0 {
            info!("no pending attestations found");
            return PurgeResult {
                purged: Vec::new(),
                has_remaining: true,
            };
        }

        let result = stamp.purge_pending();
        match result {
            Some(purged) => {
                info!("purged {purged} pending attestation(s)");
                PurgeResult {
                    purged: pending_uris,
                    has_remaining: true,
                }
            }
            None => {
                info!("all attestations were pending, timestamp is now empty");
                PurgeResult {
                    purged: pending_uris,
                    has_remaining: false,
                }
            }
        }
    }
}

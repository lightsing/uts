use std::collections::HashSet;
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
    /// (e.g., by not writing the file).
    pub fn purge_pending<A: Allocator>(
        stamp: &mut DetachedTimestamp<A>,
    ) -> PurgeResult {
        Self::purge_pending_by_uris(stamp, None)
    }

    /// Purges selected pending attestations from the given detached timestamp.
    ///
    /// If `uris_to_purge` is `None`, all pending attestations are purged.
    /// If `uris_to_purge` is `Some(set)`, only pending attestations whose URI
    /// is in the set are purged.
    ///
    /// This is implemented using [`Timestamp::retain_attestations`] under the hood.
    ///
    /// Returns a [`PurgeResult`] containing the URIs of purged attestations
    /// and whether the timestamp still has remaining (non-pending) attestations.
    pub fn purge_pending_by_uris<A: Allocator>(
        stamp: &mut DetachedTimestamp<A>,
        uris_to_purge: Option<&HashSet<String>>,
    ) -> PurgeResult {
        let pending_uris = Self::list_pending(stamp);

        if pending_uris.is_empty() {
            info!("no pending attestations found");
            return PurgeResult {
                purged: Vec::new(),
                has_remaining: true,
            };
        }

        let purged_uris: Vec<String> = match &uris_to_purge {
            Some(set) => pending_uris.iter().filter(|u| set.contains(*u)).cloned().collect(),
            None => pending_uris,
        };

        if purged_uris.is_empty() {
            info!("no matching pending attestations to purge");
            return PurgeResult {
                purged: Vec::new(),
                has_remaining: true,
            };
        }

        let result = stamp.retain_attestations(&|att| {
            if att.tag != PendingAttestation::TAG {
                return true; // keep non-pending attestations
            }
            match &uris_to_purge {
                None => false, // purge all pending
                Some(set) => {
                    let uri = PendingAttestation::from_raw(att)
                        .map(|p| p.uri.to_string())
                        .unwrap_or_default();
                    !set.contains(&uri) // keep if NOT in the purge set
                }
            }
        });

        match result {
            Some(purged) => {
                info!("purged {purged} pending attestation(s)");
                PurgeResult {
                    purged: purged_uris,
                    has_remaining: true,
                }
            }
            None => {
                info!("all attestations were pending, timestamp is now empty");
                PurgeResult {
                    purged: purged_uris,
                    has_remaining: false,
                }
            }
        }
    }
}

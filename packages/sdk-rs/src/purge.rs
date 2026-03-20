use crate::Sdk;
use uts_core::{
    alloc::{Allocator, vec::Vec},
    codec::v1::{Attestation, DetachedTimestamp, PendingAttestation},
};

/// Represents the result of filtering pending attestations from a detached timestamp.
#[derive(Debug)]
pub struct PurgeResult<A: Allocator> {
    /// URIs of the pending attestations that were excluded (purged) during the filtering process.
    pub purged: Vec<String, A>,
    /// A new detached timestamp instance containing only the retained attestations.
    pub new_stamp: DetachedTimestamp<A>,
}

impl Sdk {
    /// Filters out all pending attestations from the given detached timestamp.
    ///
    /// This method creates a new `DetachedTimestamp` excluding all entries tagged as `Pending`.
    /// The original `stamp` remains unchanged.
    ///
    /// # Arguments
    ///
    /// * `stamp` - A reference to the source `DetachedTimestamp`.
    /// * `purge_malformed` - A boolean flag indicating whether malformed `PendingAttestation` entries
    ///   should be purged (`true`) or retained (`false`) in the new timestamp.
    ///
    /// # Returns
    ///
    /// Returns `Some(PurgeResult)` if results in a valid timestamp, otherwise returns `None`.
    pub fn filter_pending<A: Allocator + Clone>(
        stamp: &DetachedTimestamp<A>,
        purge_malformed: bool,
    ) -> Option<PurgeResult<A>> {
        Self::filter_pending_by_uris(stamp, |_| true, purge_malformed)
    }

    /// Filters pending attestations from the given detached timestamp based on a predicate.
    ///
    /// This function iterates over the attestations in `stamp`. For each attestation tagged as
    /// `Pending`, it attempts to decode the URI and applies the `predicate`.
    /// - If the predicate returns `true`, the attestation is excluded from the new timestamp.
    /// - If the predicate returns `false`, or if the attestation is not pending, it is retained.
    ///
    /// # Arguments
    ///
    /// * `stamp` - A reference to the source `DetachedTimestamp`.
    /// * `predicate` - A closure that determines whether a specific pending attestation URI
    ///   should be excluded. Returns `true` to exclude, `false` to retain.
    /// * `purge_malformed` - A boolean flag indicating whether malformed `PendingAttestation` entries
    ///   should be purged (`true`) or retained (`false`) in the new timestamp.
    ///
    /// # Note
    ///
    /// - Non-pending attestations are always retained.
    /// - Malformed `PendingAttestation` entries (those that fail to decode) are safely retained
    ///   in the new timestamp to prevent data loss.
    ///
    /// # Returns
    ///
    /// Returns `Some(PurgeResult)` if it results in a valid timestamp, otherwise returns `None`.
    pub fn filter_pending_by_uris<A: Allocator + Clone, F>(
        stamp: &DetachedTimestamp<A>,
        mut predicate: F,
        purge_malformed: bool,
    ) -> Option<PurgeResult<A>>
    where
        F: FnMut(&str) -> bool,
    {
        let mut purged = Vec::new_in(stamp.allocator().clone());

        stamp
            .filter_attestations(|att| {
                if att.tag != PendingAttestation::TAG {
                    return false; // keep non-pending attestations
                }
                let Ok(uri) = PendingAttestation::from_raw(att).map(|p| p.uri) else {
                    return purge_malformed;
                };
                let result = predicate(&uri);
                if result {
                    purged.push(uri.to_string())
                }
                result
            })
            .map(|new_stamp| {
                let new_stamp = DetachedTimestamp::<A>::from_parts(*stamp.header(), new_stamp);
                PurgeResult { purged, new_stamp }
            })
    }
}

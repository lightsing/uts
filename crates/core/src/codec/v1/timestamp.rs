//! ** The implementation here is subject to change as this is a read-only version. **

use crate::{
    codec::v1::{attestation::RawAttestation, opcode::OpCode},
    utils::{Hexed, OnceLock},
};
use alloc::{alloc::Global, vec::Vec};
use core::{alloc::Allocator, fmt::Debug};

mod builder;
mod decode;
mod encode;
mod fmt;

/// Proof that that one or more attestations commit to a message.
///
/// This should not be confused with [`DetachedTimestamp`](crate::codec::v1::DetachedTimestamp),
/// single [`Timestamp`]s **DO NOT** include the digest of the message they commit to.
///
/// Sample Timestamp:
/// ```text
/// execute APPEND 7d9472db4ae254e8
/// execute SHA256
/// execute APPEND 65191d41c625e4505a442928ec4211b3
/// execute SHA256
/// execute APPEND 000639ee5837a935dce596c85f1ce323d5219afe84ee0832ee6614924f4c6598
/// execute SHA256
/// execute PREPEND 6944db61
/// execute APPEND 0ef41e45bb5534b3
/// result attested by Pending: update URI https://alice.btc.calendar.opentimestamps.org
/// ```
#[derive(Clone, Debug)]
pub enum Timestamp<A: Allocator = Global> {
    Step(Step<A>),
    Attestation(RawAttestation<A>),
}

/// An execution Step.
#[derive(Clone)]
pub struct Step<A: Allocator = Global> {
    op: OpCode,
    data: Vec<u8, A>,
    input: OnceLock<Vec<u8, A>>,
    next: Vec<Timestamp<A>, A>,
}

impl<A: Allocator> PartialEq for Timestamp<A> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Timestamp::Step(s1), Timestamp::Step(s2)) => s1 == s2,
            (Timestamp::Attestation(a1), Timestamp::Attestation(a2)) => a1 == a2,
            _ => false,
        }
    }
}
impl<A: Allocator> Eq for Timestamp<A> {}

impl<A: Allocator> PartialEq for Step<A> {
    fn eq(&self, other: &Self) -> bool {
        self.op == other.op && self.data == other.data && self.next == other.next
    }
}
impl<A: Allocator> Eq for Step<A> {}

impl<A: Allocator + Debug> Debug for Step<A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut f = f.debug_struct("Step");
        f.field("op", &self.op);
        if self.op.has_immediate() {
            f.field("data", &Hexed(&self.data));
        }
        f.field("next", &self.next).finish()
    }
}

impl<A: Allocator> Timestamp<A> {
    /// Returns the opcode of this timestamp node.
    pub fn op(&self) -> OpCode {
        match self {
            Timestamp::Step(step) => {
                debug_assert_ne!(
                    step.op,
                    OpCode::ATTESTATION,
                    "sanity check failed: Step with ATTESTATION opcode"
                );
                step.op
            }
            Timestamp::Attestation(_) => OpCode::ATTESTATION,
        }
    }

    /// Returns this timestamp as a step, if it is one.
    #[inline]
    pub fn as_step(&self) -> Option<&Step<A>> {
        match self {
            Timestamp::Step(step) => Some(step),
            Timestamp::Attestation(_) => None,
        }
    }

    /// Returns this timestamp as an attestation, if it is one.
    #[inline]
    pub fn as_attestation(&self) -> Option<&RawAttestation<A>> {
        match self {
            Timestamp::Attestation(attestation) => Some(attestation),
            Timestamp::Step(_) => None,
        }
    }

    /// Returns the input data for this timestamp node, if finalized.
    #[inline]
    pub fn input(&self) -> Option<&[u8]> {
        match self {
            Timestamp::Step(step) => step.input.get().map(|v| v.as_slice()),
            Timestamp::Attestation(attestation) => attestation.value.get().map(|v| v.as_slice()),
        }
    }

    /// Returns the allocator used by this timestamp node.
    #[inline]
    pub fn allocator(&self) -> &A {
        match self {
            Self::Attestation(attestation) => attestation.allocator(),
            Self::Step(step) => step.allocator(),
        }
    }
}

impl<A: Allocator + Clone> Timestamp<A> {
    /// Finalizes the timestamp with the given input data.
    ///
    /// # Panics
    ///
    /// Panics if the timestamp is already finalized with different input data.
    pub fn finalize(&self, input: Vec<u8, A>) {
        match self {
            Self::Attestation(attestation) => {
                if let Some(already) = attestation.value.get() {
                    assert_eq!(&input, already, "trying to finalize with different input");
                    return;
                }
                let _ = attestation.value.get_or_init(|| input);
            }
            Self::Step(step) => {
                if let Some(already) = step.input.get() {
                    assert_eq!(&input, already, "trying to finalize with different input");
                    return;
                }
                let input = step.input.get_or_init(|| input);

                match step.op {
                    OpCode::FORK => {
                        debug_assert!(step.next.len() >= 2, "FORK must have at least two children");
                        for child in &step.next {
                            child.finalize(input.clone());
                        }
                    }
                    OpCode::ATTESTATION => unreachable!("should not happen"),
                    op => {
                        let output = op.execute_in(input, &step.data, step.allocator().clone());
                        debug_assert!(step.next.len() == 1, "non-FORK must have exactly one child");
                        step.next[0].finalize(output);
                    }
                }
            }
        }
    }
}

impl<A: Allocator> Step<A> {
    /// Returns the allocator used by this step.
    pub(crate) fn allocator(&self) -> &A {
        self.data.allocator()
    }
}

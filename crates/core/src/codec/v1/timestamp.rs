//! ** The implementation here is subject to change as this is a read-only version. **

use crate::{
    codec::v1::{FinalizationError, MayHaveInput, attestation::RawAttestation, opcode::OpCode},
    utils::{Hexed, OnceLock},
};
use alloc::{alloc::Global, vec::Vec};
use core::{alloc::Allocator, fmt::Debug};

pub(crate) mod builder;
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

impl Timestamp {
    /// Creates a new timestamp builder.
    pub fn builder() -> builder::TimestampBuilder<Global> {
        builder::TimestampBuilder::new_in(Global)
    }

    /// Merges multiple timestamps into a single FORK timestamp.
    ///
    /// # Panics
    ///
    /// This will panic if there are conflicting inputs when finalizing unfinalized timestamps.
    pub fn merge(timestamps: Vec<Timestamp, Global>) -> Timestamp {
        Self::merge_in(timestamps, Global)
    }

    /// Try to merge multiple timestamps into a single FORK timestamp.
    ///
    /// Returns an error if there are conflicting inputs when finalizing unfinalized timestamps.
    pub fn try_merge(timestamps: Vec<Timestamp, Global>) -> Result<Timestamp, FinalizationError> {
        Self::try_merge_in(timestamps, Global)
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

    /// Returns the allocator used by this timestamp node.
    #[inline]
    pub fn allocator(&self) -> &A {
        match self {
            Self::Attestation(attestation) => attestation.allocator(),
            Self::Step(step) => step.allocator(),
        }
    }

    /// Returns true if this timestamp is finalized.
    #[inline]
    pub fn is_finalized(&self) -> bool {
        self.input().is_some()
    }

    /// Iterates over all attestations in this timestamp.
    #[inline]
    pub fn attestations(&self) -> AttestationIter<'_, A> {
        AttestationIter { stack: vec![self] }
    }
}

impl<A: Allocator + Clone> Timestamp<A> {
    /// Creates a new timestamp builder with the given allocator.
    pub fn builder_in(alloc: A) -> builder::TimestampBuilder<A> {
        builder::TimestampBuilder::new_in(alloc)
    }

    /// Finalizes the timestamp with the given input data.
    ///
    /// # Panics
    ///
    /// Panics if the timestamp is already finalized with different input data.
    #[inline]
    pub fn finalize(&self, input: &[u8]) {
        self.try_finalize(input)
            .expect("conflicting inputs when finalizing timestamp")
    }

    /// Try finalizes the timestamp with the given input data.
    ///
    /// Returns an error if the timestamp is already finalized with different input data.
    pub fn try_finalize(&self, input: &[u8]) -> Result<(), FinalizationError> {
        let init_fn = || input.to_vec_in(self.allocator().clone());
        match self {
            Self::Attestation(attestation) => {
                if let Some(already) = attestation.value.get() {
                    return if &input != already {
                        Err(FinalizationError)
                    } else {
                        Ok(())
                    };
                }
                let _ = attestation.value.get_or_init(init_fn);
            }
            Self::Step(step) => {
                if let Some(already) = step.input.get() {
                    return if &input != already {
                        Err(FinalizationError)
                    } else {
                        Ok(())
                    };
                }
                let input = step.input.get_or_init(init_fn);

                match step.op {
                    OpCode::FORK => {
                        debug_assert!(step.next.len() >= 2, "FORK must have at least two children");
                        for child in &step.next {
                            child.try_finalize(input)?;
                        }
                    }
                    OpCode::ATTESTATION => unreachable!("should not happen"),
                    op => {
                        let output = op.execute_in(input, &step.data, step.allocator().clone());
                        debug_assert!(step.next.len() == 1, "non-FORK must have exactly one child");
                        step.next[0].try_finalize(&output)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Merges multiple timestamps into a single FORK timestamp.
    ///
    /// # Panics
    ///
    /// This will panic if there are conflicting inputs when finalizing unfinalized timestamps.
    pub fn merge_in(timestamps: Vec<Timestamp<A>, A>, alloc: A) -> Timestamp<A> {
        Self::try_merge_in(timestamps, alloc).expect("conflicting inputs when merging timestamps")
    }

    /// Merges multiple timestamps into a single FORK timestamp.
    ///
    /// This will attempt to finalize unfinalized timestamps if any of the input timestamps are finalized.
    ///
    /// Returns an error if there are conflicting inputs when finalizing unfinalized timestamps.
    pub fn try_merge_in(
        timestamps: Vec<Timestamp<A>, A>,
        alloc: A,
    ) -> Result<Timestamp<A>, FinalizationError> {
        // if any timestamp is finalized, ensure they are with the same input,
        // finalize unfinalized timestamps with that input
        let finalized_input = timestamps.iter().find_map(|ts| ts.input());
        if let Some(ref input) = finalized_input {
            for ts in timestamps.iter().filter(|ts| !ts.is_finalized()) {
                ts.try_finalize(input)?;
            }
        }

        Ok(Timestamp::Step(Step {
            op: OpCode::FORK,
            data: Vec::new_in(alloc.clone()),
            input: OnceLock::new(),
            next: timestamps,
        }))
    }
}

impl<A: Allocator> MayHaveInput for Timestamp<A> {
    #[inline]
    fn input(&self) -> Option<&[u8]> {
        match self {
            Timestamp::Step(step) => step.input(),
            Timestamp::Attestation(attestation) => attestation.input(),
        }
    }
}

impl<A: Allocator> Step<A> {
    /// Returns the opcode of this step.
    pub fn op(&self) -> OpCode {
        self.op
    }

    /// Returns the immediate data of this step.
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    /// Returns the next timestamps of this step.
    pub fn next(&self) -> &[Timestamp<A>] {
        self.next.as_slice()
    }

    /// Returns the allocator used by this step.
    pub fn allocator(&self) -> &A {
        self.data.allocator()
    }
}

impl<A: Allocator> MayHaveInput for Step<A> {
    #[inline]
    fn input(&self) -> Option<&[u8]> {
        self.input.get().map(|v| v.as_slice())
    }
}

#[must_use = "AttestationIter is an iterator, it does nothing unless consumed"]
pub struct AttestationIter<'a, A: Allocator> {
    stack: Vec<&'a Timestamp<A>>,
}

impl<'a, A: Allocator> Iterator for AttestationIter<'a, A> {
    type Item = &'a RawAttestation<A>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(ts) = self.stack.pop() {
            match ts {
                Timestamp::Step(step) => {
                    for next in step.next().iter().rev() {
                        self.stack.push(next);
                    }
                }
                Timestamp::Attestation(attestation) => return Some(attestation),
            }
        }
        None
    }
}

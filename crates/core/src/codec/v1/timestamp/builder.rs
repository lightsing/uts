//! Timestamp Builder

use crate::{
    codec::v1::{
        Attestation, Timestamp,
        opcode::{DigestOpExt, OpCode},
        timestamp::Step,
    },
    error::EncodeError,
    utils::OnceLock,
};
use alloc::alloc::{Allocator, Global};
use uts_bmt::{NodePosition, SiblingIter};

#[derive(Debug, Clone)]
pub struct TimestampBuilder<A: Allocator = Global> {
    steps: Vec<LinearStep<A>, A>,
}

#[derive(Debug, Clone)]
struct LinearStep<A: Allocator> {
    op: OpCode,
    data: Vec<u8, A>,
}

impl<A: Allocator + Clone> TimestampBuilder<A> {
    /// Creates a new `TimestampBuilder`.
    pub fn new_in(alloc: A) -> TimestampBuilder<A> {
        TimestampBuilder {
            steps: Vec::new_in(alloc),
        }
    }

    /// Pushes a new execution step with immediate data to the timestamp.
    ///
    /// # Panics
    ///
    /// Panics if the opcode is not an opcode with immediate data.
    pub(crate) fn push_immediate_step(&mut self, op: OpCode, data: Vec<u8, A>) -> &mut Self {
        assert!(op.has_immediate());
        self.steps.push(LinearStep { op, data });
        self
    }

    /// Pushes a new execution step without immediate data to the timestamp.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - the opcode is control opcode
    /// - the opcode is an opcode with immediate data
    pub fn push_step(&mut self, op: OpCode) -> &mut Self {
        self.steps.push(LinearStep {
            op,
            data: Vec::new_in(self.allocator().clone()),
        });
        self
    }

    /// Pushes a new digest step to the timestamp.
    pub fn digest<D: DigestOpExt>(&mut self) -> &mut Self {
        self.push_step(D::OPCODE.to_opcode());
        self
    }

    /// Pushes the steps corresponding to the given Merkle proof to the timestamp.
    pub fn merkle_proof<D: DigestOpExt>(&mut self, proof: SiblingIter<'_, D>) -> &mut Self {
        let alloc = self.allocator().clone();
        for (side, sibling_hash) in proof {
            match side {
                NodePosition::Left => self
                    .prepend([uts_bmt::INNER_NODE_PREFIX].to_vec_in(alloc.clone()))
                    .append(sibling_hash.to_vec_in(alloc.clone())),
                NodePosition::Right => self
                    .prepend(sibling_hash.to_vec_in(alloc.clone()))
                    .prepend([uts_bmt::INNER_NODE_PREFIX].to_vec_in(alloc.clone())),
            }
            .digest::<D>();
        }
        self
    }

    /// Computes the commitment of the timestamp for the given input.
    ///
    /// In this context, the **commitment** is the deterministic result of
    /// executing the timestamp's linear chain of operations over the input
    /// bytes. It is computed by:
    ///
    /// 1. Taking the provided `input` bytes as the initial value.
    /// 2. Iterating over all steps in the order they were added to the builder.
    /// 3. For each step, applying its opcode to the current value together
    ///    with the step's immediate data via [`OpCode::execute_in`], and using
    ///    the result as the new current value.
    ///
    /// The final value after all steps have been applied is returned as the
    /// commitment.
    pub fn commitment(&self, input: impl AsRef<[u8]>) -> Vec<u8, A> {
        let alloc = self.allocator().clone();
        let mut commitment = input.as_ref().to_vec_in(alloc.clone());
        for step in &self.steps {
            commitment = step.op.execute_in(&commitment, &step.data, alloc.clone());
        }
        commitment
    }

    /// Finalizes the timestamp with the given attestation.
    ///
    /// # Notes
    ///
    /// The built timestamp does not include any input data. The input data must be
    /// provided later using the `finalize` method on the `Timestamp` object.
    pub fn attest<'a, T: Attestation<'a>>(
        self,
        attestation: T,
    ) -> Result<Timestamp<A>, EncodeError> {
        let current = Timestamp::Attestation(attestation.to_raw_in(self.allocator().clone())?);
        Ok(self.concat(current))
    }

    /// Append the given timestamp after the steps in the builder.
    pub fn concat(self, timestamp: Timestamp<A>) -> Timestamp<A> {
        let alloc = self.allocator().clone();

        let mut current = timestamp;

        for step in self.steps.into_iter().rev() {
            let step_node = Step {
                op: step.op,
                data: step.data,
                input: OnceLock::new(),
                next: {
                    let mut v = Vec::with_capacity_in(1, alloc.clone());
                    v.push(current);
                    v
                },
            };
            current = Timestamp::Step(step_node);
        }

        current
    }

    #[inline]
    fn allocator(&self) -> &A {
        self.steps.allocator()
    }
}

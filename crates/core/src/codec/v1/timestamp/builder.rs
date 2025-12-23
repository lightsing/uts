//! Timestamp Builder

use crate::{
    codec::v1::{Attestation, Timestamp, opcode::OpCode, timestamp::Step},
    error::EncodeError,
    utils::OnceLock,
};
use alloc::alloc::{Allocator, Global};

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
    pub(crate) fn push_immediate_step(mut self, op: OpCode, data: Vec<u8, A>) -> Self {
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
    pub(crate) fn push_step(mut self, op: OpCode) -> Self {
        self.steps.push(LinearStep {
            op,
            data: Vec::new_in(self.allocator().clone()),
        });
        self
    }

    /// Computes the commitment of the timestamp.
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
        let alloc = self.allocator().clone();

        let mut current = Timestamp::Attestation(attestation.to_raw_in(alloc.clone())?);

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

        Ok(current)
    }

    #[inline]
    fn allocator(&self) -> &A {
        self.steps.allocator()
    }
}

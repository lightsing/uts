use super::*;
use crate::{
    codec::{Encode, Encoder},
    error::EncodeError,
};
use std::io::Write;

impl Encode for Timestamp {
    fn encode(&self, mut writer: impl Encoder) -> Result<(), EncodeError> {
        self.encode_step_recurse(&mut writer, &self.steps.last().unwrap())
    }
}

impl Timestamp {
    fn encode_step_recurse<W: Write>(
        &self,
        writer: &mut W,
        step: &Step,
    ) -> Result<(), EncodeError> {
        // 1. Write OpCode
        // Note: We need a way to serialize the OpCode (e.g., as u8)
        writer.encode(step.opcode)?;

        // 2. Write data
        match step.opcode {
            OpCode::ATTESTATION => {
                // SAFETY: caller ensures step is attestation step
                let attest_idx = unsafe { self.get_attest_idx(step) };
                let attest = &self.attestations[attest_idx as usize];
                attest.encode(&mut *writer)?;
            }
            _ if step.data_len != 0 => {
                // SAFETY: caller ensures step is valid
                let step_data = unsafe { self.get_step_data(step) };
                writer.encode_bytes(step_data)?;
            }
            _ => {}
        }

        if let Some(first_child) = resolve_ptr(step.first_child) {
            let mut current = &self.steps[first_child];
            if let OpCode::FORK = step.opcode {
                // Encode the first child
                self.encode_step_recurse(writer, current)?;

                // Logic: Child -> FORK -> Child -> ... -> LastChild
                while let Some(next_sibling_idx) = resolve_ptr(current.next_sibling) {
                    let next = &self.steps[next_sibling_idx];
                    // Encode current child
                    self.encode_step_recurse(writer, next)?;
                    // Write Separator FORK
                    let continues = resolve_ptr(next.next_sibling).is_some();
                    if continues {
                        writer.encode(OpCode::FORK)?;
                    }

                    // Move to next
                    current = next;
                }
                Ok(())
            } else {
                // FIXME: tailcall optimization
                self.encode_step_recurse(writer, current)
            }
        } else {
            Ok(())
        }
    }
}

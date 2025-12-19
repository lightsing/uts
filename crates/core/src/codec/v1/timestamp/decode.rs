use super::*;
use crate::{
    codec::{Decode, Decoder},
    error::DecodeError,
};
use std::io::BufRead;

impl Decode for Timestamp {
    fn decode(mut reader: impl Decoder) -> Result<Timestamp, DecodeError> {
        let mut steps = Vec::new();
        let mut data = Vec::new();
        let mut attestations = Vec::new();

        Self::decode_step_recurse(
            &mut reader,
            &mut steps,
            &mut data,
            &mut attestations,
            None,
            RECURSION_LIMIT,
        )?;

        Ok(Timestamp {
            steps,
            data,
            attestations,
        })
    }
}

impl Timestamp {
    fn decode_step_recurse<R: BufRead>(
        reader: &mut R,
        steps: &mut Vec<Step>,
        data: &mut Vec<u8>,
        attestations: &mut Vec<Attestation>,
        op: Option<OpCode>,
        recursion_limit: usize,
    ) -> Result<StepPtr, DecodeError> {
        if recursion_limit == 0 {
            return Err(DecodeError::RecursionLimit);
        }

        let op = match op {
            Some(op) => op,
            None => reader.decode()?,
        };

        let step = match op {
            OpCode::ATTESTATION => {
                let attest = Attestation::decode(reader)?;
                let attest_idx = attestations.len();
                attestations.push(attest);
                let (data_offset, data_len) =
                    Self::push_to_heap(data, &(attest_idx as u32).to_le_bytes());
                Step {
                    opcode: op,
                    data_len,
                    data_offset,
                    ..Default::default()
                }
            }
            OpCode::FORK => {
                let mut first_child: StepPtr = None;
                let mut prev_sibling_idx: Option<usize> = None;

                let mut next_op = OpCode::FORK;

                while next_op == OpCode::FORK {
                    let child_ptr = Self::decode_step_recurse(
                        reader,
                        steps,
                        data,
                        attestations,
                        None,
                        recursion_limit - 1,
                    )?;

                    // LCRS:
                    // if prev sibling exist, link its next_sibling to current child
                    // else it's first_child
                    if let Some(prev) = prev_sibling_idx {
                        steps[prev].next_sibling = child_ptr;
                    } else {
                        first_child = child_ptr;
                    }

                    // update prev_sibling_idx to current child
                    prev_sibling_idx = resolve_ptr(child_ptr);

                    next_op = reader.decode()?;
                }

                let child_ptr = Self::decode_step_recurse(
                    reader,
                    steps,
                    data,
                    attestations,
                    Some(next_op),
                    recursion_limit - 1,
                )?;
                if let Some(prev) = prev_sibling_idx {
                    steps[prev].next_sibling = child_ptr;
                } else {
                    first_child = child_ptr;
                }

                Step {
                    opcode: op,
                    data_len: 0,
                    data_offset: 0,
                    first_child,
                    ..Default::default()
                }
            }
            _ => {
                debug_assert!(!op.is_control());
                let (data_offset, data_len) = if op.has_immediate() {
                    let length = reader.decode_ranged(1..=MAX_OP_LENGTH)?;
                    // SAFETY: We will fill the buffer right after getting it.
                    let (data_offset, data_len, buffer) =
                        unsafe { Self::get_buffer_from_heap(data, length) };
                    reader.read_exact(buffer)?;
                    (data_offset, data_len)
                } else {
                    (0, 0)
                };

                let next = Self::decode_step_recurse(
                    reader,
                    steps,
                    data,
                    attestations,
                    None,
                    recursion_limit - 1,
                )?;

                Step {
                    opcode: op,
                    data_len,
                    data_offset,
                    first_child: next,
                    ..Default::default()
                }
            }
        };

        let step_idx = steps.len();
        steps.push(step);
        Ok(make_ptr(step_idx))
    }
}

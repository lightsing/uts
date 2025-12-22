use super::*;
use crate::{
    codec::{Decode, DecodeIn, Decoder},
    error::DecodeError,
};

const RECURSION_LIMIT: usize = 256;

impl<A: Allocator + Copy> DecodeIn<A> for Timestamp<A> {
    fn decode_in(decoder: &mut impl Decoder, alloc: A) -> Result<Self, DecodeError> {
        Self::decode_recursive(decoder, RECURSION_LIMIT, alloc)
    }
}

impl<A: Allocator + Copy> Timestamp<A> {
    fn decode_recursive(
        decoder: &mut impl Decoder,
        recursion_limit: usize,
        alloc: A,
    ) -> Result<Timestamp<A>, DecodeError> {
        if recursion_limit == 0 {
            return Err(DecodeError::RecursionLimit);
        }
        let op = OpCode::decode(&mut *decoder)?;

        Self::decode_from_op(op, decoder, recursion_limit, alloc)
    }

    fn decode_from_op(
        op: OpCode,
        decoder: &mut impl Decoder,
        limit: usize,
        alloc: A,
    ) -> Result<Timestamp<A>, DecodeError> {
        match op {
            OpCode::ATTESTATION => {
                let attestation = RawAttestation::decode_in(decoder, alloc)?;
                Ok(Timestamp::Attestation(attestation))
            }
            OpCode::FORK => {
                let mut children = Vec::new_in(alloc);
                let mut next_op = OpCode::FORK;
                while next_op == OpCode::FORK {
                    let child = Self::decode_recursive(&mut *decoder, limit - 1, alloc)?;
                    children.push(child);
                    next_op = OpCode::decode(&mut *decoder)?;
                }
                children.push(Self::decode_from_op(next_op, decoder, limit - 1, alloc)?);
                Ok(Timestamp::Step(Step {
                    op: OpCode::FORK,
                    data: Vec::new_in(alloc),
                    next: children,
                }))
            }
            _ => {
                let data = if op.has_immediate() {
                    const MAX_OP_LENGTH: usize = 4096;
                    let length = decoder.decode_ranged(1..MAX_OP_LENGTH)?;
                    let mut data = Vec::with_capacity_in(length, alloc);
                    data.resize(length, 0);
                    decoder.read_exact(&mut data)?;

                    data
                } else {
                    Vec::new_in(alloc)
                };

                let mut next = Vec::with_capacity_in(1, alloc);
                next.push(Self::decode_recursive(decoder, limit - 1, alloc)?);

                Ok(Timestamp::Step(Step { op, data, next }))
            }
        }
    }
}

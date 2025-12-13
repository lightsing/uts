use crate::codec::{
    Proof, Version,
    v1::{Attestation, DigestHeader, opcode::OpCode},
};
use std::num::NonZeroU32;

type StepPtr = Option<NonZeroU32>;

mod decode;
mod encode;
mod fmt;
mod builder;

const RECURSION_LIMIT: usize = 256;
const MAX_OP_LENGTH: usize = 4096;

/// Fully decoded OpenTimestamps proof for version 1.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Timestamp {
    header: DigestHeader,
    steps: Vec<Step>,
    data: Vec<u8>,
    attestations: Vec<Attestation>,
}

/// An OpenTimestamps step.
#[derive(Clone, PartialEq, Eq, Debug)]
#[repr(C)]
struct Step {
    opcode: OpCode,
    _padding: u8,
    data_len: u16,
    data_offset: u32,
    // LCRS tree structure
    first_child: StepPtr,
    next_sibling: StepPtr,
}
// cache line aligned
const _: () = assert!(size_of::<Step>() == 16);

impl Default for Step {
    fn default() -> Self {
        Step {
            opcode: OpCode::ATTESTATION,
            _padding: 0,
            data_len: 0,
            data_offset: 0,
            first_child: None,
            next_sibling: None,
        }
    }
}

impl Proof for Timestamp {
    const VERSION: Version = 1;
}

impl Timestamp {
    /// Returns the data slice associated with a step.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the step was constructed from this timestamp's data buffer.
    #[inline]
    unsafe fn get_step_data(&self, step: &Step) -> &[u8] {
        if step.data_len == 0 {
            return &[];
        }
        let start = step.data_offset as usize;
        debug_assert!(start < self.data.len());
        let end = start + step.data_len as usize;
        debug_assert!(end <= self.data.len());
        // SAFETY: bounds checked above
        unsafe { self.data.get_unchecked(start..end) }
    }

    /// Returns the attestation index encoded by an attestation step.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the step is an attestation step and that the
    /// safety requirements of [`Self::get_step_data`] also hold.
    #[inline]
    unsafe fn get_attest_idx(&self, step: &Step) -> u32 {
        debug_assert!(step.opcode == OpCode::ATTESTATION);
        debug_assert_eq!(step.data_len as usize, size_of::<u32>());
        let data = unsafe { self.get_step_data(step) };
        u32::from_le_bytes(data.try_into().unwrap())
    }

    #[inline]
    fn push_to_heap(heap: &mut Vec<u8>, data: &[u8]) -> (u32, u16) {
        if data.is_empty() {
            return (0, 0);
        }

        let offset = heap.len();
        let len = data.len();

        assert!(offset <= u32::MAX as usize, "Data heap overflow (max 4GB)");
        assert!(len <= u16::MAX as usize, "Ref data too large (max 65KB)");

        heap.extend_from_slice(data);

        (offset as u32, len as u16)
    }

    /// Returns a mutable buffer from the heap.
    ///
    /// # Safety
    ///
    /// The caller must write exactly `len` bytes into the returned buffer.
    #[inline]
    unsafe fn get_buffer_from_heap(heap: &mut Vec<u8>, len: usize) -> (u32, u16, &mut [u8]) {
        let offset = heap.len();
        assert!(offset <= u32::MAX as usize, "Data heap overflow (max 4GB)");
        assert!(len <= u16::MAX as usize, "Ref data too large (max 65KB)");

        heap.reserve(len);

        // SAFETY: we just reserved enough space
        let buffer = unsafe {
            heap.set_len(offset + len);
            heap.get_unchecked_mut(offset..offset + len)
        };

        (offset as u32, len as u16, buffer)
    }
}

#[inline]
fn make_ptr(idx: usize) -> StepPtr {
    assert!(idx < u32::MAX as usize);
    NonZeroU32::new((idx + 1) as u32)
}

#[inline]
fn resolve_ptr(ptr: StepPtr) -> Option<usize> {
    ptr.map(|nz| (nz.get() - 1) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        codec::{Decode, Encode, proof::VersionedProof},
        fixtures,
    };

    #[test]
    fn round_trip() {
        let mut encoded_small = vec![];
        let mut encoded_large = vec![];

        let ots = VersionedProof::<Timestamp>::decode(fixtures::SMALL_DETACHED_TIMESTAMP);
        assert!(ots.is_ok());
        assert!(ots.unwrap().encode(&mut encoded_small).is_ok());
        assert_eq!(encoded_small, fixtures::SMALL_DETACHED_TIMESTAMP);

        let ots = VersionedProof::<Timestamp>::decode(fixtures::LARGE_DETACHED_TIMESTAMP);
        assert!(ots.is_ok());
        assert!(ots.unwrap().encode(&mut encoded_large).is_ok());
        assert_eq!(encoded_large, fixtures::LARGE_DETACHED_TIMESTAMP);
    }
}

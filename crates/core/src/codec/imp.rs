use crate::codec::*;
use alloc::vec::Vec;

#[cfg(feature = "bytes")]
mod bytes;
mod primitives;
#[cfg(feature = "std")]
mod std_io;

#[cfg(feature = "std")]
pub use std_io::{Reader, Writer};

impl<A: Allocator> Encoder for Vec<u8, A> {
    fn encode_byte(&mut self, byte: u8) -> Result<(), EncodeError> {
        self.push(byte);
        Ok(())
    }

    fn write_all(&mut self, buf: impl AsRef<[u8]>) -> Result<(), EncodeError> {
        self.extend_from_slice(buf.as_ref());
        Ok(())
    }
}

impl Decoder for &[u8] {
    fn decode_byte(&mut self) -> Result<u8, DecodeError> {
        let Some((a, b)) = self.split_at_checked(1) else {
            return Err(DecodeError::UnexpectedEof);
        };
        *self = b;
        Ok(a[0])
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DecodeError> {
        let len = buf.len();
        let Some((a, b)) = self.split_at_checked(len) else {
            return Err(DecodeError::UnexpectedEof);
        };
        buf.copy_from_slice(a);
        *self = b;
        Ok(())
    }
}

use crate::codec::*;
use smallvec::SmallVec;

impl<const N: usize> Encoder for SmallVec<[u8; N]> {
    fn encode_byte(&mut self, byte: u8) -> Result<(), EncodeError> {
        self.push(byte);
        Ok(())
    }

    fn write_all(&mut self, buf: impl AsRef<[u8]>) -> Result<(), EncodeError> {
        self.extend_from_slice(buf.as_ref());
        Ok(())
    }
}

use crate::codec::*;
use bytes::{BufMut, BytesMut};

impl Encoder for BytesMut {
    fn encode_byte(&mut self, byte: u8) -> Result<(), EncodeError> {
        self.put_u8(byte);
        Ok(())
    }

    fn write_all(&mut self, buf: impl AsRef<[u8]>) -> Result<(), EncodeError> {
        self.put_slice(buf.as_ref());
        Ok(())
    }
}

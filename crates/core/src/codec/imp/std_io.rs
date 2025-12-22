use crate::codec::*;
use std::io::{Read, Write};

pub struct Writer<W: Write>(pub W);
pub struct Reader<R: Read>(pub R);

impl<W: Write> Encoder for Writer<W> {
    fn encode_byte(&mut self, byte: u8) -> Result<(), EncodeError> {
        self.write_all(&[byte])?;
        Ok(())
    }

    #[inline]
    fn write_all(&mut self, buf: impl AsRef<[u8]>) -> Result<(), EncodeError> {
        self.0.write_all(buf.as_ref())?;
        Ok(())
    }
}

impl<R: Read> Decoder for Reader<R> {
    fn decode_byte(&mut self) -> Result<u8, DecodeError> {
        let mut byte = [0];
        self.read_exact(&mut byte)?;
        Ok(byte[0])
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), DecodeError> {
        self.0.read_exact(buf)?;
        Ok(())
    }
}

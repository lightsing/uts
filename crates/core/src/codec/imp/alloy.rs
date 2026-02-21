use crate::codec::{Decode, DecodeError, Decoder, Encode, EncodeError, Encoder};
use alloy_chains::Chain;
use alloy_primitives::{Address, ChainId, FixedBytes};

impl<const N: usize> Encode for FixedBytes<N> {
    fn encode(&self, encoder: &mut impl Encoder) -> Result<(), EncodeError> {
        encoder.write_all(self)
    }
}

impl<const N: usize> Decode for FixedBytes<N> {
    fn decode(decoder: &mut impl Decoder) -> Result<Self, DecodeError> {
        let mut buf = [0u8; N];
        decoder.read_exact(&mut buf)?;
        Ok(Self::new(buf))
    }
}

impl Encode for Address {
    fn encode(&self, encoder: &mut impl Encoder) -> Result<(), EncodeError> {
        encoder.write_all(self.0)
    }
}

impl Decode for Address {
    fn decode(decoder: &mut impl Decoder) -> Result<Self, DecodeError> {
        let inner: FixedBytes<20> = decoder.decode()?;
        Ok(Self::from(inner))
    }
}

impl Encode for Chain {
    fn encode(&self, encoder: &mut impl Encoder) -> Result<(), EncodeError> {
        self.id().encode(encoder)
    }
}

impl Decode for Chain {
    fn decode(decoder: &mut impl Decoder) -> Result<Self, DecodeError> {
        let id: ChainId = decoder.decode()?;
        Ok(Chain::from_id(id))
    }
}

use crate::codec::{
    Decode, DecodeIn, Encode, Encoder, Proof, Version,
    v1::{DigestHeader, Timestamp},
};
use alloc::alloc::{Allocator, Global};
use core::{fmt, fmt::Formatter};

/// A file containing a timestamp for another file
/// Contains a timestamp, along with a header and the digest of the file.
///
/// This is not equivalent to the python DetachedTimestamp structure,
/// which don't encode/decode the magic and version.
/// The Python version is equivalent to `VersionedProof<DetachedTimestamp>`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DetachedTimestamp<A: Allocator = Global> {
    header: DigestHeader,
    timestamp: Timestamp<A>,
}

impl<A: Allocator + Clone> Proof<A> for DetachedTimestamp<A> {
    const VERSION: Version = 1;
}

impl<A: Allocator + Clone> DecodeIn<A> for DetachedTimestamp<A> {
    fn decode_in(
        decoder: &mut impl crate::codec::Decoder,
        alloc: A,
    ) -> Result<Self, crate::error::DecodeError> {
        let header = DigestHeader::decode(decoder)?;
        let timestamp = Timestamp::decode_in(decoder, alloc)?;
        Ok(DetachedTimestamp { header, timestamp })
    }
}

impl<A: Allocator> Encode for DetachedTimestamp<A> {
    fn encode(&self, encoder: &mut impl Encoder) -> Result<(), crate::error::EncodeError> {
        self.header.encode(encoder)?;
        self.timestamp.encode(encoder)?;
        Ok(())
    }
}

impl<A: Allocator + Clone> fmt::Display for DetachedTimestamp<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "digest of {}", self.header)?;

        self.timestamp.fmt(Some(self.header.digest()), f)
    }
}

impl<A: Allocator> DetachedTimestamp<A> {
    /// Returns the digest header.
    pub fn header(&self) -> &DigestHeader {
        &self.header
    }

    /// Returns the timestamp.
    pub fn timestamp(&self) -> &Timestamp<A> {
        &self.timestamp
    }

    /// Returns the allocator used by this detached timestamp.
    #[inline]
    pub fn allocator(&self) -> &A {
        self.timestamp.allocator()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        codec::{Decode, Encoder, proof::VersionedProof},
        fixtures,
    };

    #[test]
    fn round_trip() {
        let mut encoded_small = vec![];
        let mut encoded_large = vec![];

        let ots =
            VersionedProof::<DetachedTimestamp>::decode(&mut &*fixtures::SMALL_DETACHED_TIMESTAMP)
                .unwrap();
        println!("{:#?}", ots);
        println!("{}", ots);
        assert!(encoded_small.encode(&ots).is_ok());
        assert_eq!(encoded_small, fixtures::SMALL_DETACHED_TIMESTAMP);

        let ots =
            VersionedProof::<DetachedTimestamp>::decode(&mut &*fixtures::LARGE_DETACHED_TIMESTAMP)
                .unwrap();
        assert!(encoded_large.encode(&ots).is_ok());
        assert_eq!(encoded_large, fixtures::LARGE_DETACHED_TIMESTAMP);
    }
}

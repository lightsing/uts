use crate::codec::{
    Decode, Encode, Encoder, Proof, Version,
    v1::{DigestHeader, Timestamp},
};
use core::{fmt, fmt::Formatter};
use smallvec::ToSmallVec;

/// A file containing a timestamp for another file
/// Contains a timestamp, along with a header and the digest of the file.
///
/// This is not equivalent to the python DetachedTimestamp structure,
/// which don't encode/decode the magic and version.
/// The Python version is equivalent to `VersionedProof<DetachedTimestamp>`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DetachedTimestamp {
    header: DigestHeader,
    timestamp: Timestamp,
}

impl Proof for DetachedTimestamp {
    const VERSION: Version = 1;
}

impl Decode for DetachedTimestamp {
    fn decode(decoder: &mut impl crate::codec::Decoder) -> Result<Self, crate::error::DecodeError> {
        let header = DigestHeader::decode(decoder)?;
        let timestamp = Timestamp::decode(decoder)?;
        Ok(DetachedTimestamp { header, timestamp })
    }
}

impl Encode for DetachedTimestamp {
    fn encode(&self, encoder: &mut impl Encoder) -> Result<(), crate::error::EncodeError> {
        self.header.encode(encoder)?;
        self.timestamp.encode(encoder)?;
        Ok(())
    }
}

impl fmt::Display for DetachedTimestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "digest of {}", self.header)?;

        self.timestamp
            .fmt(Some(&self.header.digest().to_smallvec()), f)
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

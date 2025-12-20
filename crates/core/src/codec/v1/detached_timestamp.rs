use crate::codec::{
    Decode, Encode, Proof, Version,
    v1::{DigestHeader, Timestamp, timestamp},
};
use smallvec::ToSmallVec;
use std::{fmt, fmt::Formatter};

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
    fn decode(mut reader: impl crate::codec::Decoder) -> Result<Self, crate::error::DecodeError> {
        let header = DigestHeader::decode(&mut reader)?;
        let timestamp = Timestamp::decode(&mut reader)?;
        Ok(DetachedTimestamp { header, timestamp })
    }
}

impl Encode for DetachedTimestamp {
    fn encode(
        &self,
        mut writer: impl crate::codec::Encoder,
    ) -> Result<(), crate::error::EncodeError> {
        self.header.encode(&mut writer)?;
        self.timestamp.encode(&mut writer)?;
        Ok(())
    }
}

impl fmt::Display for DetachedTimestamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "digest of {}", self.header)?;

        timestamp::fmt::fmt(
            &self.timestamp,
            Some(&self.header.digest().to_smallvec()),
            f,
        )
    }
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

        let ots = VersionedProof::<DetachedTimestamp>::decode(fixtures::SMALL_DETACHED_TIMESTAMP);
        assert!(ots.is_ok());
        assert!(ots.unwrap().encode(&mut encoded_small).is_ok());
        assert_eq!(encoded_small, fixtures::SMALL_DETACHED_TIMESTAMP);

        let ots = VersionedProof::<DetachedTimestamp>::decode(fixtures::LARGE_DETACHED_TIMESTAMP);
        assert!(ots.is_ok());
        assert!(ots.unwrap().encode(&mut encoded_large).is_ok());
        assert_eq!(encoded_large, fixtures::LARGE_DETACHED_TIMESTAMP);
    }
}

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

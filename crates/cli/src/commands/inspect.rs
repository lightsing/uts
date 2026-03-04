use clap::Args;
use std::{fs, io, io::Seek, path::PathBuf};
use uts_core::codec::{
    Decode, Reader, VersionedProof,
    v1::{DetachedTimestamp, Timestamp},
};

#[derive(Debug, Args)]
pub struct Inspect {
    /// Path to the OTS file to inspect
    ots_file: PathBuf,
}

impl Inspect {
    pub fn run(self) -> eyre::Result<()> {
        let mut fh = fs::File::open(&self.ots_file)?;

        match VersionedProof::<DetachedTimestamp>::decode(&mut Reader(&mut fh)) {
            Ok(ots) => {
                eprintln!("OTS Detached Timestamp found:\n{ots}");
                return Ok(());
            }
            Err(e) => {
                eprintln!("Not a valid Detached Timestamp OTS file (trying raw timestamp): {e}\n");
            }
        };

        fh.seek(io::SeekFrom::Start(0))?;

        let raw = Timestamp::decode(&mut Reader(&mut fh))?;
        eprintln!("Raw Timestamp found:\n{raw}");
        Ok(())
    }
}

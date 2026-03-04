use alloy_primitives::Address;
use alloy_provider::ProviderBuilder;
use clap::Args;
use digest::{Digest, DynDigest};
use eyre::bail;
use jiff::{Timestamp, tz::TimeZone};
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
    process,
};
use uts_contracts::eas::EAS_ADDRESSES;
use uts_core::{
    codec::{
        Decode, Reader, VersionedProof,
        v1::{
            Attestation, DetachedTimestamp, EASAttestation, EASTimestamped, PendingAttestation,
            opcode::*,
        },
    },
    utils::Hexed,
    verifier::{AttestationVerifier, EASVerifier},
};

#[derive(Debug, Args)]
pub struct Verify {
    file: PathBuf,
    stamp_file: Option<PathBuf>,
    /// Optional Ethereum provider URL for verifying Ethereum UTS attestations.
    /// If not provided, a default provider will be used based on the chain ID.
    #[arg(long)]
    eth_provider: Option<String>,
    /// Optional EAS contract address for verifying EAS attestations. If not provided, the default
    /// EAS contract for the chain will be used.
    #[arg(long)]
    eas: Option<Address>,
}

impl Verify {
    pub async fn run(self) -> eyre::Result<()> {
        let stamp_file = self.stamp_file.unwrap_or_else(|| {
            let mut default = self.file.clone();
            default.add_extension("ots");
            default
        });
        let timestamp =
            VersionedProof::<DetachedTimestamp>::decode(&mut Reader(File::open(stamp_file)?))?
                .proof;

        let digest_header = timestamp.header();
        let mut hasher = match digest_header.kind().tag() {
            SHA1 => Box::new(sha1::Sha1::new()) as Box<dyn DynDigest>,
            RIPEMD160 => Box::new(ripemd::Ripemd160::new()) as Box<dyn DynDigest>,
            SHA256 => Box::new(sha2::Sha256::new()) as Box<dyn DynDigest>,
            KECCAK256 => Box::new(sha3::Keccak256::new()) as Box<dyn DynDigest>,
            _ => bail!("Unsupported digest type: {}", digest_header.kind()),
        };

        let mut file = BufReader::new(File::open(self.file)?);
        let mut buffer = [0u8; 64 * 1024]; // 64KB buffer
        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }
        let expected = hasher.finalize();

        if *expected != *digest_header.digest() {
            eprintln!(
                "Digest mismatch! Expected: {}, Found: {}",
                Hexed(&expected),
                Hexed(digest_header.digest())
            );
            process::exit(1);
        }
        eprintln!("Digest matches: {}", Hexed(&expected));

        timestamp.try_finalize()?;

        for attestation in timestamp.attestations() {
            if attestation.tag == PendingAttestation::TAG {
                continue; // skip pending attestations
            }
            if attestation.tag != EASAttestation::TAG && attestation.tag != EASTimestamped::TAG {
                eprintln!("Unknown attestation type: {}", Hexed(&attestation.tag));
            }

            let expected = attestation
                .value()
                .expect("Attestation value should be finalized");
            let chain = if attestation.tag == EASAttestation::TAG {
                EASAttestation::from_raw(attestation)?.chain
            } else {
                EASTimestamped::from_raw(attestation)?.chain
            };

            let provider_url = if let Some(url) = self.eth_provider.as_deref() {
                url
            } else {
                match chain.id() {
                    1 => "https://0xrpc.io/eth",
                    11155111 => "https://0xrpc.io/sep",
                    534352 => "https://rpc.scroll.io",
                    534351 => "https://sepolia-rpc.scroll.io",
                    _ => bail!("Unsupported chain: {chain}"),
                }
            };

            let eas_address = if let Some(addr) = self.eas {
                addr
            } else {
                EAS_ADDRESSES
                    .get(&chain.id())
                    .copied()
                    .ok_or_else(|| eyre::eyre!("No default EAS contract for chain: {chain}"))?
            };

            let provider = ProviderBuilder::new().connect(provider_url).await?;
            let verifier = EASVerifier::new(eas_address, provider);

            let time: u64;
            if attestation.tag == EASAttestation::TAG {
                let eas_attestation = EASAttestation::from_raw(attestation)?;
                let result = verifier.verify(&eas_attestation, expected).await?;
                eprintln!("EAS Onchain Attestation: {}", result.uid);
                time = result.time;
                eprintln!("\tattester: {}", result.attester);
            } else {
                let timestamped = EASTimestamped::from_raw(attestation)?;
                time = verifier.verify(&timestamped, expected).await?;
                eprintln!("EAS Timestamped");
            }

            let ts = Timestamp::from_second(time as i64)?;
            let zdt = ts.to_zoned(TimeZone::system());
            eprintln!("\ttime attested: {zdt}");
            eprintln!("\tmerkle root: {}", Hexed(&expected));
        }

        Ok(())
    }
}

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
use uts_core::{
    codec::{
        Decode, Reader, VersionedProof,
        v1::{
            Attestation, DetachedTimestamp, EthereumUTSAttestation, PendingAttestation, opcode::*,
        },
    },
    utils::Hexed,
    verifier::{AttestationVerifier, EthereumUTSVerifier},
};

#[derive(Debug, Args)]
pub struct Verify {
    file: PathBuf,
    stamp_file: Option<PathBuf>,
    /// Optional Ethereum provider URL for verifying Ethereum UTS attestations.
    /// If not provided, a default provider will be used based on the chain ID.
    #[arg(long)]
    eth_provider: Option<String>,
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

            if attestation.tag == EthereumUTSAttestation::TAG {
                let eth_attestation = EthereumUTSAttestation::from_raw(&attestation)?;
                eprintln!("Attested by {eth_attestation}");
                let provider_url = if let Some(url) = self.eth_provider.as_deref() {
                    url
                } else {
                    match eth_attestation.chain.id() {
                        1 => "https://0xrpc.io/eth",
                        11155111 => "https://0xrpc.io/sep",
                        534352 => "https://rpc.scroll.io",
                        534351 => "https://sepolia-rpc.scroll.io",
                        _ => bail!("Unsupported chain: {}", eth_attestation.chain),
                    }
                };
                let provider = ProviderBuilder::new().connect(provider_url).await?;
                let verifier = EthereumUTSVerifier::new(provider).await?;
                let result = verifier
                    .verify(&eth_attestation, attestation.value().unwrap())
                    .await?;
                if let Some(block_number) = result.block_number {
                    if let Some(block_hash) = result.block_hash {
                        eprintln!("\tblock: #{block_number} {block_hash}");
                    } else {
                        eprintln!("\tblock: {block_number}");
                    }
                }
                if let Some(log_index) = result.log_index {
                    eprintln!("\tlog index: {log_index}");
                }
                if let Some(transaction_hash) = result.transaction_hash {
                    if let Some((_, etherscan_url)) = eth_attestation.chain.etherscan_urls() {
                        eprintln!("\ttransaction: {etherscan_url}/tx/{transaction_hash}");
                    } else {
                        eprintln!("\ttransaction hash: {transaction_hash}");
                    }
                }

                if let Some((_, etherscan_url)) = eth_attestation.chain.etherscan_urls() {
                    eprintln!(
                        "\tuts contract: {etherscan_url}/address/{}",
                        result.inner.address
                    );
                } else {
                    eprintln!("\tuts contract: {}", result.inner.address);
                }
                if let Some((_, etherscan_url)) = eth_attestation.chain.etherscan_urls() {
                    eprintln!(
                        "\ttx sender: {etherscan_url}/address/{}",
                        result.inner.sender
                    );
                } else {
                    eprintln!("\ttx sender: {}", result.inner.sender);
                }
                let ts = Timestamp::from_second(result.inner.timestamp.to())?;
                let zdt = ts.to_zoned(TimeZone::system());
                eprintln!("\ttime attested: {zdt}");
                eprintln!("\tmerkle root: {}", result.inner.root);
                continue;
            }

            eprintln!("Unverifiable attestation: {attestation}");
        }

        Ok(())
    }
}

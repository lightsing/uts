use alloy_provider::ProviderBuilder;
use digest::{Digest, DynDigest};
use jiff::{Timestamp, tz::TimeZone};
use std::{
    env, fs,
    io::{BufReader, Read},
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

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file> <ots> [eth provider url]", args[0]);
        process::exit(1);
    }

    let mut fh = BufReader::new(if args.len() >= 3 {
        fs::File::open(&args[2])?
    } else {
        fs::File::open(format!("{}.ots", &args[1]))?
    });
    let timestamp = VersionedProof::<DetachedTimestamp>::decode(&mut Reader(&mut fh))?.proof;

    let digest_header = timestamp.header();
    let mut hasher = match digest_header.kind().tag() {
        SHA1 => Box::new(sha1::Sha1::new()) as Box<dyn DynDigest>,
        RIPEMD160 => Box::new(ripemd::Ripemd160::new()) as Box<dyn DynDigest>,
        SHA256 => Box::new(sha2::Sha256::new()) as Box<dyn DynDigest>,
        KECCAK256 => Box::new(sha3::Keccak256::new()) as Box<dyn DynDigest>,
        _ => {
            eprintln!("Unsupported digest type: {}", digest_header.kind());
            process::exit(1);
        }
    };

    let mut file = BufReader::new(fs::File::open(&args[1])?);
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
            let provider_url = if args.len() >= 4 {
                &*args[3]
            } else {
                match eth_attestation.chain.id() {
                    1 => "https://0xrpc.io/eth",
                    11155111 => "https://0xrpc.io/sep",
                    534352 => "https://rpc.scroll.io",
                    534351 => "https://sepolia-rpc.scroll.io",
                    _ => panic!("Unsupported chain: {}", eth_attestation.chain),
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
        }
    }

    Ok(())
}

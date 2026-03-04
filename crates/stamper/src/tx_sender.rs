//! Attest Tx Sender
//!
//! # Note
//!
//! We need to separate the merkle tree creation and transaction sending into two separate tasks.
//!
//! The first task (merkle tree creation) is infallible in most cases unless there is a storage
//! failure, and any error should consider as **FATAL** and requires devops intervention.
//!
//! The second task (transaction sending) is more likely to encounter transient errors
//! (e.g. network issues, RPC errors, etc.).
//!
//! By separating these two tasks, we can ensure that transient errors in transaction sending do
//! not affect the merkle tree creation process, and the stamper can continue to consume journal
//! entries and create merkle trees without being blocked by transaction sending issues.

use crate::{sql, sql::AttestTx};
use alloy_primitives::ChainId;
use alloy_provider::Provider;
use eyre::bail;
use std::{future, pin::Pin};
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;
use uts_contracts::eas::EAS;

pub(super) struct TxSender<P> {
    /// The contract to interact with
    ///
    /// The underlying provider should configure with retry layer to handle transient errors, and
    /// the TxSender will rely on the provider's retry mechanism.
    pub eas: EAS<P>,
    /// The SQL db
    pub sql_storage: sqlx::SqlitePool,
    /// Waker
    pub waker: Receiver<()>,
    /// Cancellation token to signal shutdown
    pub token: CancellationToken,
}

impl<P: Provider + 'static> TxSender<P> {
    pub async fn run_until_cancelled(mut self) {
        let Ok(chain_id) = self.eas.provider().get_chain_id().await else {
            error!("Failed to get chain id, TxSender cannot start without it.");
            self.token.cancel(); // signal cancellation to the main stamper loop
            return;
        };

        info!("TxSender started on chain id: {}", chain_id);
        loop {
            let may_retry = match self.process_all_pending(chain_id).await {
                Err(e) => {
                    error!(error = ?e, "Error processing pending attestations, will retry in the next cycle.");
                    Box::pin(tokio::time::sleep(std::time::Duration::from_secs(10)))
                        as Pin<Box<dyn Future<Output = ()> + Send>>
                }
                Ok(()) => Box::pin(future::pending()),
            };

            tokio::select! {
                _ = self.token.cancelled() => {
                    info!("Cancellation received, stopping TxSender...");
                    break;
                }
                _ = self.waker.recv() => {
                    // got a wake-up signal, process pending attestations immediately
                    debug!("Wake up signal received, processing pending attestations...");
                }
                _ = may_retry => {
                    // either an error happened and we need to retry after a delay
                    debug!("Retrying to process pending attestations after delay...");
                }
            }
        }
    }

    async fn process_all_pending(&self, chain_id: ChainId) -> eyre::Result<()> {
        let pending_attestations = sql::load_all_pending_attestations(&self.sql_storage).await?;
        for attestation in pending_attestations {
            if let Err(e) = self.process_one(&attestation, chain_id).await {
                error!(error = ?e, "Error processing attestation id {}, will retry later.", attestation.id);
                // log the failed attempt
                sql::new_attest_attempt(&self.sql_storage, attestation.id, chain_id, None).await?;
            }
        }

        Ok(())
    }

    async fn process_one(
        &self,
        attestation: &sql::PendingAttestation,
        chain_id: ChainId,
    ) -> eyre::Result<()> {
        let res = self.eas.timestamp(attestation.trie_root).send().await;
        match res {
            Ok(pending_tx) => {
                let receipt = pending_tx.get_receipt().await?;
                let Some(block_number) = receipt.block_number else {
                    bail!(
                        "Transaction for trie root {} is not yet included in a block",
                        attestation.trie_root
                    );
                };

                sql::new_attest_attempt(
                    &self.sql_storage,
                    attestation.id,
                    chain_id,
                    Some(AttestTx {
                        tx_hash: receipt.transaction_hash,
                        block_number,
                    }),
                )
                .await?;
            }
            Err(e) if e.as_revert_data().is_some() => {
                // this timestamp is already timestamped
                let ts = self.eas.getTimestamp(attestation.trie_root).call().await?;

                let provider = self.eas.provider();

                // binary search the timestamp to find the block number and tx hash
                const MAX_GET_LOGS_BLOCK: u64 = 100; // getLogs
                let block_number = provider.get_block_number().await?;

                let mut low = 0;
                let mut high = block_number;

                while low <= high {
                    let mid = (low + high) / 2;
                    let Some(block) = provider.get_block(mid.into()).await? else {
                        bail!("Failed to get block {mid}");
                    };
                    if block.header.timestamp > ts {
                        high = mid - 1;
                    } else if block.header.timestamp < ts {
                        low = mid + 1;
                    }
                    if high - low <= MAX_GET_LOGS_BLOCK {
                        break;
                    }
                }

                let Some((_, log)) = self
                    .eas
                    .Timestamped_filter()
                    .from_block(low)
                    .to_block(high)
                    .topic1(attestation.trie_root)
                    .query()
                    .await?
                    .into_iter()
                    .next()
                else {
                    bail!(
                        "Failed to find log for trie root {} between blocks {} and {}",
                        attestation.trie_root,
                        low,
                        high
                    );
                };

                let Some(tx_hash) = log.transaction_hash else {
                    bail!(
                        "Log for trie root {} does not have a transaction hash",
                        attestation.trie_root
                    );
                };
                let Some(block_number) = log.block_number else {
                    bail!(
                        "Log for trie root {} does not have a block number",
                        attestation.trie_root
                    );
                };

                sql::new_attest_attempt(
                    &self.sql_storage,
                    attestation.id,
                    chain_id,
                    Some(AttestTx {
                        tx_hash,
                        block_number,
                    }),
                )
                .await?;
            }
            Err(e) => return Err(e.into()),
        }
        Ok(())
    }
}

use crate::sql;
use alloy_primitives::{BlockNumber, ChainId};
use alloy_sol_types::SolEvent;
use eyre::ContextCompat;
use futures::StreamExt;
use sqlx::{SqliteConnection, SqlitePool};
use std::{fmt::Debug, pin::Pin};
use tokio::select;
use tokio_util::sync::CancellationToken;

type BoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

const REWIND_BLOCKS: u64 = 100;

mod l2;
pub use l2::L2Indexer;

struct Subscriber<'a, Provider, Event, F> {
    chain_id: ChainId,
    db: SqlitePool,
    event_filter: alloy_contract::Event<&'a Provider, Event>,
    insert_event_fn: F,
    cancellation_token: CancellationToken,
}

impl<Provider, Event, F> Subscriber<'_, Provider, Event, F>
where
    Provider: alloy_provider::Provider + 'static,
    Event: SolEvent + Debug + 'static,
    F: for<'a> Fn(
        &'a mut SqliteConnection,
        ChainId,
        i64,
        Event,
    ) -> BoxedFuture<'a, eyre::Result<()>>,
{
    async fn run(self) -> eyre::Result<()> {
        let Self {
            chain_id,
            db,
            event_filter,
            insert_event_fn,
            cancellation_token,
        } = self;

        let subscription = event_filter.subscribe().await?;
        let mut stream = subscription.into_stream();
        loop {
            select! {
                maybe_event = stream.next() => {
                    match maybe_event {
                        Some(Ok((event, log))) => {
                            let mut tx = db.begin().await?;
                            let block_number = log.block_number.context("Log missing block number")?;
                            info!(
                                event = Event::SIGNATURE,
                                block_number,
                                transaction_hash = ?log.transaction_hash,
                                log_index = ?log.log_index,
                                event_data = ?event,
                                "Received event"
                            );
                            // This update is unsafe, cuz we might not handle all events in this block.
                            // But during the startup phase, the scanner will rewind the cursor a few blocks to make sure we don't miss any events.
                            sql::indexer::update_cursor(&mut *tx, chain_id, Event::SIGNATURE_HASH, block_number).await?;
                            let id = sql::indexer::insert_log(&mut *tx, chain_id, log).await?;
                            insert_event_fn(&mut tx, chain_id, id, event).await?;
                            tx.commit().await?;
                        },
                        Some(Err(e)) => {
                            error!("Error while receiving event: {e}");
                            cancellation_token.cancel();
                            break;
                        }
                        None => {
                            error!("Event stream ended unexpectedly");
                            cancellation_token.cancel();
                            break;
                        }
                    }
                }
                _ = cancellation_token.cancelled() => {
                    info!("Cancellation requested, stopping event subscription");
                    break;
                }
            }
        }
        Ok(())
    }
}

struct Scanner<'a, Provider, Event, F> {
    chain_id: ChainId,
    db: SqlitePool,
    default_start_block: BlockNumber,
    batch_size: u64,
    event_filter: alloy_contract::Event<&'a Provider, Event>,
    insert_event_fn: F,
    cancellation_token: CancellationToken,
}

impl<Provider, Event, F> Scanner<'_, Provider, Event, F>
where
    Provider: alloy_provider::Provider + 'static,
    Event: SolEvent + Debug + 'static,
    F: for<'a> Fn(
        &'a mut SqliteConnection,
        ChainId,
        i64,
        Event,
    ) -> BoxedFuture<'a, eyre::Result<()>>,
{
    async fn run(self) -> eyre::Result<()> {
        let cancellation_token = self.cancellation_token.clone();
        select! {
            res = self.run_inner()  => {
                res
            }
            _ = cancellation_token.cancelled() => {
                info!("Cancellation requested, stopping event scanning");
                Ok(())
            }
        }
    }

    async fn run_inner(self) -> eyre::Result<()> {
        let Self {
            chain_id,
            db,
            default_start_block,
            batch_size,
            mut event_filter,
            insert_event_fn: insert_event,
            ..
        } = self;

        let mut start_block = sql::indexer::load_cursor(&db, chain_id, Event::SIGNATURE_HASH)
            .await?
            .map(|blk| {
                // Rewind a few blocks to make sure we don't miss any events. See above comment in
                // the subscription handler for details.
                blk.saturating_sub(REWIND_BLOCKS)
            })
            .unwrap_or(default_start_block);

        let mut end_block = event_filter.provider.get_block_number().await?;

        while start_block <= end_block {
            let batch_end = std::cmp::min(start_block + batch_size - 1, end_block);
            debug!(
                event = Event::SIGNATURE,
                start_block,
                end_block = batch_end,
                "Scanning for events"
            );
            event_filter = event_filter.from_block(start_block).to_block(batch_end);

            let mut tx = db.begin().await?;
            for (event, log) in event_filter.query().await? {
                info!(
                    event = Event::SIGNATURE,
                    block_number = log.block_number,
                    transaction_hash = ?log.transaction_hash,
                    log_index = ?log.log_index,
                    event_data = ?event,
                    "Received event"
                );
                let id = sql::indexer::insert_log(&mut *tx, chain_id, log).await?;
                insert_event(&mut tx, chain_id, id, event).await?;
            }
            sql::indexer::update_cursor(&mut *tx, chain_id, Event::SIGNATURE_HASH, batch_end)
                .await?;

            start_block = batch_end + 1;
            tx.commit().await?;
            end_block = event_filter.provider.get_block_number().await?;
        }

        Ok(())
    }
}

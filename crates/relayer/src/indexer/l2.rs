use crate::{
    config::L2IndexerConfig,
    indexer::{BoxedFuture, Scanner, Subscriber},
    sql,
};
use alloy_contract::Event;
use alloy_primitives::ChainId;
use alloy_provider::Provider;
use sqlx::{SqliteConnection, SqlitePool};
use std::sync::Arc;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use uts_contracts::manager::{
    L2AnchoringManager,
    events::{L1AnchoringQueued, L1BatchArrived, L1BatchFinalized},
};

/// Indexer for L2 events.
#[derive(Debug)]
pub struct L2Indexer<P: Provider> {
    db: SqlitePool,
    chain_id: ChainId,
    manager: L2AnchoringManager<P>,
    config: L2IndexerConfig,
    cancellation_token: CancellationToken,
}

impl<P: Provider + 'static> L2Indexer<P> {
    /// Create a new instance of the indexer.
    pub async fn new(
        db: SqlitePool,
        manager: L2AnchoringManager<P>,
        config: L2IndexerConfig,
        cancellation_token: CancellationToken,
    ) -> eyre::Result<Arc<Self>> {
        let chain_id = manager.provider().get_chain_id().await?;
        Ok(Arc::new(Self {
            db,
            chain_id,
            manager,
            config,
            cancellation_token,
        }))
    }

    /// Start the scanners.
    pub async fn start_scanners(self: Arc<Self>) -> eyre::Result<()> {
        let mut join_set = JoinSet::new();

        join_set.spawn(self.clone().scan_l1anchoring_queued());
        join_set.spawn(self.clone().scan_l1batch_arrived());
        join_set.spawn(self.clone().scan_l1batch_finalized());

        for task in join_set.join_all().await {
            task?
        }
        Ok(())
    }

    /// Start the subscribers.
    pub async fn start_subscribers(self: Arc<Self>) -> eyre::Result<()> {
        let mut join_set = JoinSet::new();

        join_set.spawn(self.clone().subscribe_l1anchoring_queued());
        join_set.spawn(self.clone().subscribe_l1batch_arrived());
        join_set.spawn(self.clone().subscribe_l1batch_finalized());

        for task in join_set.join_all().await {
            task?
        }
        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn scan_l1anchoring_queued(self: Arc<Self>) -> eyre::Result<()> {
        let scanner = Scanner {
            chain_id: self.chain_id,
            db: self.db.clone(),
            default_start_block: self.config.start_block,
            batch_size: self.config.batch_size,
            event_filter: self.l1anchoring_queued_filter(),
            insert_event_fn: insert_l1anchoring_queued,
            cancellation_token: self.cancellation_token.clone(),
        };

        if let Err(e) = scanner.run().await {
            error!("Error while scanning for events: {e:?}");
            self.cancellation_token.cancel();
            return Err(e);
        }
        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn subscribe_l1anchoring_queued(self: Arc<Self>) -> eyre::Result<()> {
        let subscriber = Subscriber {
            chain_id: self.chain_id,
            db: self.db.clone(),
            event_filter: self.l1anchoring_queued_filter(),
            insert_event_fn: insert_l1anchoring_queued,
            cancellation_token: self.cancellation_token.clone(),
        };

        if let Err(e) = subscriber.run().await {
            error!("Error while subscribing to events: {e:?}");
            self.cancellation_token.cancel();
        }
        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn scan_l1batch_arrived(self: Arc<Self>) -> eyre::Result<()> {
        let scanner = Scanner {
            chain_id: self.chain_id,
            db: self.db.clone(),
            default_start_block: self.config.start_block,
            batch_size: self.config.batch_size,
            event_filter: self.l1batch_arrived_filter(),
            insert_event_fn: insert_l1batch_arrived,
            cancellation_token: self.cancellation_token.clone(),
        };

        if let Err(e) = scanner.run().await {
            error!("Error while scanning for events: {e:?}");
            self.cancellation_token.cancel();
        }
        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn subscribe_l1batch_arrived(self: Arc<Self>) -> eyre::Result<()> {
        let subscriber = Subscriber {
            chain_id: self.chain_id,
            db: self.db.clone(),
            event_filter: self.l1batch_arrived_filter(),
            insert_event_fn: insert_l1batch_arrived,
            cancellation_token: self.cancellation_token.clone(),
        };

        if let Err(e) = subscriber.run().await {
            error!("Error while subscribing to events: {e:?}");
            self.cancellation_token.cancel();
        }

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn scan_l1batch_finalized(self: Arc<Self>) -> eyre::Result<()> {
        let scanner = Scanner {
            chain_id: self.chain_id,
            db: self.db.clone(),
            default_start_block: self.config.start_block,
            batch_size: self.config.batch_size,
            event_filter: self.l1batch_finalized_filter(),
            insert_event_fn: insert_l1batch_finalized,
            cancellation_token: self.cancellation_token.clone(),
        };

        if let Err(e) = scanner.run().await {
            error!("Error while scanning for events: {e:?}");
            self.cancellation_token.cancel();
        }
        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn subscribe_l1batch_finalized(self: Arc<Self>) -> eyre::Result<()> {
        let subscriber = Subscriber {
            chain_id: self.chain_id,
            db: self.db.clone(),
            event_filter: self.l1batch_finalized_filter(),
            insert_event_fn: insert_l1batch_finalized,
            cancellation_token: self.cancellation_token.clone(),
        };

        if let Err(e) = subscriber.run().await {
            error!("Error while subscribing to events: {e:?}");
            self.cancellation_token.cancel();
        }

        Ok(())
    }

    #[inline]
    fn l1anchoring_queued_filter(&self) -> Event<&P, L1AnchoringQueued> {
        self.manager
            .L1AnchoringQueued_filter()
            .address(*self.manager.address())
    }

    #[inline]
    fn l1batch_arrived_filter(&self) -> Event<&P, L1BatchArrived> {
        self.manager
            .L1BatchArrived_filter()
            .address(*self.manager.address())
    }

    #[inline]
    fn l1batch_finalized_filter(&self) -> Event<&P, L1BatchFinalized> {
        self.manager
            .L1BatchFinalized_filter()
            .address(*self.manager.address())
    }
}

#[inline]
fn insert_l1anchoring_queued(
    executor: &'_ mut SqliteConnection,
    _chain_id: ChainId,
    log_id: i64,
    event: L1AnchoringQueued,
) -> BoxedFuture<'_, eyre::Result<()>> {
    Box::pin(sql::anchoring_requests::insert_l1anchoring_queued(
        executor, log_id, event,
    ))
}

#[inline]
fn insert_l1batch_arrived(
    executor: &'_ mut SqliteConnection,
    chain_id: ChainId,
    log_id: i64,
    event: L1BatchArrived,
) -> BoxedFuture<'_, eyre::Result<()>> {
    Box::pin(sql::l1_batch::insert_l1batch_arrived(
        executor, chain_id, log_id, event,
    ))
}

#[inline]
fn insert_l1batch_finalized(
    executor: &'_ mut SqliteConnection,
    chain_id: ChainId,
    log_id: i64,
    event: L1BatchFinalized,
) -> BoxedFuture<'_, eyre::Result<()>> {
    Box::pin(sql::l1_batch::insert_l1batch_finalized(
        executor, chain_id, log_id, event,
    ))
}

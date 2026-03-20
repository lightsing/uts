PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS indexer_cursors (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  chain_id INTEGER NOT NULL,
  event_signature_hash TEXT NOT NULL,
  last_indexed_block INTEGER NOT NULL,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_indexer_cursors_chain_id_event_signature ON indexer_cursors (chain_id, event_signature_hash);

CREATE TABLE IF NOT EXISTS eth_block (
   id INTEGER PRIMARY KEY AUTOINCREMENT,
   chain_id INTEGER NOT NULL,
   block_hash TEXT NOT NULL,
   block_number INTEGER NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_eth_block_chain_id_block_number ON eth_block (chain_id, block_number);
CREATE UNIQUE INDEX IF NOT EXISTS idx_eth_block_chain_id_block_hash ON eth_block (chain_id, block_hash);

CREATE TABLE IF NOT EXISTS eth_transaction (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  internal_block_id     INTEGER NOT NULL REFERENCES eth_block (id) ON DELETE CASCADE,
  transaction_index INTEGER NOT NULL,
  transaction_hash  TEXT    NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_eth_transaction_block_id_transaction_index ON eth_transaction (internal_block_id, transaction_index);
CREATE UNIQUE INDEX IF NOT EXISTS idx_eth_transaction_transaction_hash ON eth_transaction (transaction_hash);

CREATE TABLE IF NOT EXISTS eth_log (
  id              INTEGER PRIMARY KEY AUTOINCREMENT,
  internal_transaction_id INTEGER NOT NULL REFERENCES eth_transaction (id) ON DELETE CASCADE,
  log_index        INTEGER NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_eth_log_transaction_id_log_index ON eth_log (internal_transaction_id, log_index);

--  /// Emitted when a user pays to have their root anchored to L1.
--  event L1AnchoringQueued(
--      bytes32 indexed attestationId,
--      bytes32 indexed root,
--      uint256 queueIndex,
--      uint256 fee,
--      uint256 blockNumber,
--      uint256 timestamp
--  )
CREATE TABLE IF NOT EXISTS l1_anchoring_queued (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  internal_log_id INTEGER NOT NULL REFERENCES eth_log (id) ON DELETE CASCADE,

  attestation_id TEXT NOT NULL,
  root TEXT NOT NULL,
  queue_index INTEGER NOT NULL,
  fee INTEGER NOT NULL,
  block_number INTEGER NOT NULL,
  timestamp INTEGER NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_l1_anchoring_queued_log_id ON l1_anchoring_queued (internal_log_id);
CREATE INDEX IF NOT EXISTS idx_l1_anchoring_queued_attestation_id ON l1_anchoring_queued (attestation_id);
CREATE INDEX IF NOT EXISTS idx_l1_anchoring_queued_root ON l1_anchoring_queued (root);
CREATE INDEX IF NOT EXISTS idx_l1_anchoring_queued_queue_index ON l1_anchoring_queued (queue_index);
CREATE INDEX IF NOT EXISTS idx_l1_anchoring_queued_block_number ON l1_anchoring_queued (block_number);
CREATE INDEX IF NOT EXISTS idx_l1_anchoring_queued_timestamp ON l1_anchoring_queued (timestamp);

CREATE TABLE IF NOT EXISTS l1_batch (
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  l2_chain_id INTEGER NOT NULL,
  start_index INTEGER NOT NULL,

  count INTEGER NOT NULL,
  root TEXT NOT NULL,

  l1_tx_hash TEXT NULL,
  l2_tx_hash TEXT NULL,

  status TEXT NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_l1_batch_start_index ON l1_batch (l2_chain_id, start_index);
CREATE INDEX IF NOT EXISTS idx_l1_batch_root ON l1_batch (root);
CREATE INDEX IF NOT EXISTS idx_l1_batch_status ON l1_batch (status);

--  /// Emitted when L1 notifies that a batch of roots has been anchored on L1.
--  /// - `claimedRoot` The Merkle root claimed to be anchored on L1.
--  /// - `startIndex` The starting index of the batch in the queue.
--  /// - `count` The number of items in the batch.
--  /// - `l1BlockAttested` The L1 block number at which the batch was anchored. It would
--  /// be 0 if the root was timestamped before the batch submission.
--  /// - `l1TimestampAttested` The timestamp at which the batch was anchored on L1.
--  /// - `l2BlockNumber` The L2 block number at which the notification is received.
--  /// - `l2TimestampReceived` The timestamp when the notification is received.
--  ///
--  event L1BatchArrived(
--      bytes32 indexed claimedRoot,
--      uint256 indexed startIndex,
--      uint256 count,
--      uint256 l1BlockAttested,
--      uint256 l1TimestampAttested,
--      uint256 l2BlockNumber,
--      uint256 l2TimestampReceived
--  );
CREATE TABLE IF NOT EXISTS l1_batch_arrived (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  l1_batch_id INTEGER NOT NULL REFERENCES l1_batch (id) ON DELETE CASCADE,
  internal_log_id INTEGER NOT NULL REFERENCES eth_log (id) ON DELETE CASCADE,

  l1_block_attested INTEGER NOT NULL,
  l1_timestamp_attested INTEGER NOT NULL,
  l2_block_number INTEGER NOT NULL,
  l2_timestamp_received INTEGER NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_l1_batch_arrived_log_id ON l1_batch_arrived (internal_log_id);
CREATE INDEX IF NOT EXISTS idx_l1_batch_arrived_l1_block_attested ON l1_batch_arrived (l1_block_attested);
CREATE INDEX IF NOT EXISTS idx_l1_batch_arrived_l1_timestamp_attested ON l1_batch_arrived (l1_timestamp_attested);
CREATE INDEX IF NOT EXISTS idx_l1_batch_arrived_l2_block_number ON l1_batch_arrived (l2_block_number);
CREATE INDEX IF NOT EXISTS idx_l1_batch_arrived_l2_timestamp_received ON l1_batch_arrived (l2_timestamp_received);

--  /// Emitted when a batch of roots is finalized after L1 confirmation.
--  /// - `merkleRoot` The Merkle root of the batch.
--  /// - `startIndex` The starting index of the batch in the queue.
--  /// - `count` The number of items in the batch.
--  /// - `l1BlockAttested` The L1 block number at which the batch was anchored. It would be
--  /// 0 if the root was timestamped before the batch submission.
--  /// - `l1TimestampAttested` The timestamp at which the batch was anchored on L1.
--  /// - `l2BlockNumber` The L2 block number at which the batch is finalized.
--  /// - `l2TimestampFinalized` The timestamp when the batch is finalized.-
--  ///
--  event L1BatchFinalized(
--      bytes32 indexed merkleRoot,
--      uint256 indexed startIndex,
--      uint256 count,
--      uint256 l1BlockAttested,
--      uint256 l1TimestampAttested,
--      uint256 l2BlockNumber,
--      uint256 l2TimestampFinalized
--  );
CREATE TABLE IF NOT EXISTS l1_batch_finalized (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  l1_batch_id INTEGER NOT NULL REFERENCES l1_batch (id) ON DELETE CASCADE,
  internal_log_id INTEGER NOT NULL REFERENCES eth_log (id) ON DELETE CASCADE,

  l2_block_number INTEGER NOT NULL,
  l2_timestamp_finalized INTEGER NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_l1_batch_finalized_log_id ON l1_batch_finalized (internal_log_id);
CREATE INDEX IF NOT EXISTS idx_l1_batch_finalized_l2_block_number ON l1_batch_finalized (l2_block_number);
CREATE INDEX IF NOT EXISTS idx_l1_batch_finalized_l2_timestamp_finalized ON l1_batch_finalized (l2_timestamp_finalized);


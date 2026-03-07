CREATE TABLE IF NOT EXISTS batch_fee (
   id INTEGER PRIMARY KEY AUTOINCREMENT,
   internal_batch_id INTEGER NOT NULL REFERENCES l1_batch (id) ON DELETE CASCADE,

   l1_gas_fee TEXT NOT NULL,
   l2_gas_fee TEXT NOT NULL,
   cross_chain_fee TEXT NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_batch_fee_batch_id ON batch_fee (internal_batch_id);

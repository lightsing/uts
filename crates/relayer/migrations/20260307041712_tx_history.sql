CREATE TABLE IF NOT EXISTS tx_receipt (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  internal_transaction_id INTEGER NOT NULL REFERENCES eth_transaction (id) ON DELETE CASCADE,

  gas_used INTEGER NOT NULL,
  effective_gas_price TEXT NOT NULL,
  from_address TEXT NOT NULL,
  to_address TEXT NOT NULL -- all tx are not contract creation, so to_address is always non-null
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_tx_receipt_transaction_id ON tx_receipt (internal_transaction_id);
CREATE INDEX IF NOT EXISTS idx_tx_receipt_from_address ON tx_receipt (from_address);
CREATE INDEX IF NOT EXISTS idx_tx_receipt_to_address ON tx_receipt (to_address);

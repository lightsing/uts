PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS pending_attestations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  trie_root TEXT NOT NULL,
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),
  updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
  result TEXT NOT NULL DEFAULT 'pending'

);
CREATE INDEX idx_attestations_result on pending_attestations (result);

CREATE TABLE IF NOT EXISTS attest_attempts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  attestation_id INTEGER NOT NULL,
  chain_id INTEGER NOT NULL,
  tx_hash TEXT,
  block_number INTEGER,
  created_at INTEGER NOT NULL DEFAULT (unixepoch()),

  FOREIGN KEY (attestation_id) REFERENCES pending_attestations (id) ON DELETE CASCADE
);
CREATE INDEX idx_attempts_attestation_id on attest_attempts (attestation_id);

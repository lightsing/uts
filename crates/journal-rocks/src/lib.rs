//! RocksDB-backed journal implementation for UTS
//!
//! This crate provides the same functionality as `uts-journal` but uses RocksDB
//! for persistence instead of a custom WAL, providing better reliability and
//! crash recovery guarantees.
//!
//! **Design**: Entries are written directly to RocksDB on commit (synchronous,
//! immediately durable). No in-memory ring buffer or background worker thread is
//! needed. The reader fetches entries from RocksDB into an internal buffer.
//!
//! It is designed as a drop-in replacement for `uts-journal` — only the
//! dependency in `Cargo.toml` and the journal construction need to change.

#[macro_use]
extern crate tracing;

use crate::{error::JournalUnavailable, reader::JournalReader};
use rocksdb::{DB, Options, WriteBatch};
use std::{
    fmt, io,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    task::Waker,
};

/// Error types.
pub mod error;
/// Journal reader.
pub mod reader;

const META_WRITE_INDEX_KEY: &[u8] = b"\x00";
const META_CONSUMED_INDEX_KEY: &[u8] = b"\x01";

/// Read a u64 from a meta key in RocksDB, returning 0 if absent.
fn read_meta(db: &DB, key: &[u8]) -> u64 {
    db.get(key)
        .ok()
        .flatten()
        .and_then(|v| <[u8; 8]>::try_from(v.as_slice()).ok())
        .map(u64::from_le_bytes)
        .unwrap_or(0)
}

/// Configuration for the journal.
#[derive(Debug, Clone)]
pub struct JournalConfig {
    /// Directory for the RocksDB database that stores journal entries and metadata.
    pub db_path: PathBuf,
}

impl Default for JournalConfig {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("journal_db"),
        }
    }
}

/// An `At-Least-Once` journal for storing fixed-size entries, with
/// RocksDB-backed persistence.
///
/// All indices here are monotonic u64, wrapping around on overflow.
///
/// Invariant: `consumed_index` <= `write_index`.
///
/// Writes go directly to RocksDB (synchronous and durable), so there is no
/// separate "persisted" boundary — `write_index` **is** the persisted boundary.
#[derive(Clone)]
pub struct Journal<const ENTRY_SIZE: usize> {
    inner: Arc<JournalInner<ENTRY_SIZE>>,
}

pub(crate) struct JournalInner<const ENTRY_SIZE: usize> {
    /// The RocksDB instance storing entries and metadata.
    db: DB,
    /// Maximum number of in-flight (written but not yet consumed) entries.
    capacity: u64,
    /// Next write position – also the durable frontier because every commit
    /// is a synchronous RocksDB write.
    write_index: AtomicU64,
    /// Last consumed index, updated by the reader's `commit()`.
    consumed_index: AtomicU64,
    /// Serialises the write path so entries are numbered consecutively.
    write_lock: Mutex<()>,
    /// Whether a reader has been acquired.
    reader_taken: AtomicBool,
    /// Waker for the consumer waiting for new entries.
    consumer_wait: Mutex<Option<ConsumerWait>>,
    /// Shutdown flag.
    shutdown: AtomicBool,
}

impl<const ENTRY_SIZE: usize> fmt::Debug for Journal<ENTRY_SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Journal").finish()
    }
}

impl<const ENTRY_SIZE: usize> Journal<ENTRY_SIZE> {
    /// Create a new journal with the specified capacity and default configuration.
    pub fn with_capacity(capacity: usize) -> io::Result<Self> {
        Self::with_capacity_and_config(capacity, JournalConfig::default())
    }

    /// Create a new journal with the specified capacity and configuration.
    pub fn with_capacity_and_config(capacity: usize, config: JournalConfig) -> io::Result<Self> {
        let capacity = capacity as u64;

        // Open RocksDB
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, &config.db_path)
            .map_err(|e| io::Error::other(format!("failed to open RocksDB: {e}")))?;

        // Recover state
        let write_index = read_meta(&db, META_WRITE_INDEX_KEY);
        let consumed_index = read_meta(&db, META_CONSUMED_INDEX_KEY);

        if consumed_index > write_index {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Consumed index {consumed_index} is greater than write index {write_index}"
                ),
            ));
        }
        info!("Journal recovered: write_index={write_index}, consumed_index={consumed_index}");

        let inner = Arc::new(JournalInner {
            db,
            capacity,
            write_index: AtomicU64::new(write_index),
            consumed_index: AtomicU64::new(consumed_index),
            write_lock: Mutex::new(()),
            reader_taken: AtomicBool::new(false),
            consumer_wait: Mutex::new(None),
            shutdown: AtomicBool::new(false),
        });

        Ok(Self { inner })
    }

    /// Acquires a reader for this journal.
    ///
    /// # Panics
    ///
    /// Panics if a reader is already taken.
    pub fn reader(&self) -> JournalReader<ENTRY_SIZE> {
        self.try_reader().expect("Journal reader already taken")
    }

    /// Try acquires a reader for this journal.
    ///
    /// If a reader is already taken, returns None.
    pub fn try_reader(&self) -> Option<JournalReader<ENTRY_SIZE>> {
        if self.inner.reader_taken.swap(true, Ordering::AcqRel) {
            return None;
        }
        Some(JournalReader::new(self.inner.clone()))
    }

    /// Commit a new entry to the journal.
    ///
    /// The entry is written to RocksDB synchronously.
    ///
    /// # Panics
    ///
    /// Panics if the journal is full or shut down.
    pub fn commit(&self, data: &[u8; ENTRY_SIZE]) {
        self.try_commit(data).expect("Journal is unavailable")
    }

    /// Try commit a new entry to the journal.
    ///
    /// The entry is written to RocksDB synchronously.
    pub fn try_commit(&self, data: &[u8; ENTRY_SIZE]) -> Result<(), JournalUnavailable> {
        if self.inner.shutdown.load(Ordering::Acquire) {
            return Err(JournalUnavailable::Shutdown);
        }

        // Serialise writes so indices are strictly consecutive.
        let _guard = self.inner.write_lock.lock().unwrap();

        let write_idx = self.inner.write_index.load(Ordering::Acquire);
        let consumed = self.inner.consumed_index.load(Ordering::Acquire);

        if write_idx.wrapping_sub(consumed) >= self.inner.capacity {
            return Err(JournalUnavailable::Full);
        }

        // Write entry + updated write_index atomically via WriteBatch.
        let new_write_idx = write_idx.wrapping_add(1);
        let mut batch = WriteBatch::default();
        batch.put(write_idx.to_le_bytes(), data);
        batch.put(META_WRITE_INDEX_KEY, new_write_idx.to_le_bytes());
        self.inner.db.write(batch).expect("RocksDB write failed");

        self.inner
            .write_index
            .store(new_write_idx, Ordering::Release);

        // drop write_lock before notifying consumer
        drop(_guard);

        // Notify consumer if it is waiting for entries.
        self.inner.notify_consumer();

        Ok(())
    }

    /// Shut down the journal.
    pub fn shutdown(&self) -> io::Result<()> {
        self.inner.shutdown.store(true, Ordering::SeqCst);
        self.inner
            .db
            .flush()
            .map_err(|e| io::Error::other(e.to_string()))?;
        Ok(())
    }
}

impl<const ENTRY_SIZE: usize> JournalInner<ENTRY_SIZE> {
    /// Wake the consumer waker if the write index has reached its target.
    fn notify_consumer(&self) {
        let mut guard = self.consumer_wait.lock().unwrap();
        if let Some(wait) = guard.as_ref() {
            if self.write_index.load(Ordering::Acquire) >= wait.target_index {
                guard.take().unwrap().waker.wake();
            }
        }
    }
}

/// A consumer wait entry.
pub(crate) struct ConsumerWait {
    pub(crate) waker: Waker,
    pub(crate) target_index: u64,
}

#[cfg(test)]
pub(crate) mod tests {
    pub const ENTRY_SIZE: usize = 8;
    pub const TEST_DATA: &[[u8; ENTRY_SIZE]] = &[
        [0u8; ENTRY_SIZE],
        [1u8; ENTRY_SIZE],
        [2u8; ENTRY_SIZE],
        [3u8; ENTRY_SIZE],
        [4u8; ENTRY_SIZE],
        [5u8; ENTRY_SIZE],
        [6u8; ENTRY_SIZE],
        [7u8; ENTRY_SIZE],
        [8u8; ENTRY_SIZE],
        [9u8; ENTRY_SIZE],
    ];
    pub type Journal = crate::Journal<ENTRY_SIZE>;

    /// Create a journal with an isolated temporary directory for the RocksDB database.
    /// Returns the journal and the temp dir guard (must be kept alive for the test duration).
    pub fn test_journal(capacity: usize) -> (Journal, tempfile::TempDir) {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        let config = crate::JournalConfig {
            db_path: tmp.path().join("journal_db"),
        };
        let journal =
            Journal::with_capacity_and_config(capacity, config).expect("failed to create journal");
        (journal, tmp)
    }

    #[tokio::test(flavor = "current_thread")]
    async fn try_reader_is_exclusive() -> eyre::Result<()> {
        let (journal, _tmp) = test_journal(2);

        let reader = journal.try_reader().unwrap();

        assert!(
            journal.try_reader().is_none(),
            "second reader acquisition should fail"
        );

        drop(reader);
        assert!(
            journal.try_reader().is_some(),
            "reader acquisition should succeed after drop"
        );

        journal.shutdown()?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn commit_and_read_round_trip() -> eyre::Result<()> {
        let (journal, _tmp) = test_journal(4);
        let mut reader = journal.reader();

        journal.commit(&TEST_DATA[0]);
        journal.commit(&TEST_DATA[1]);

        {
            let entries = reader.read(2);
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0], TEST_DATA[0]);
            assert_eq!(entries[1], TEST_DATA[1]);
        }

        reader.commit()?;
        assert_eq!(reader.available(), 0);
        journal.shutdown()?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn commit_returns_error_when_full() -> eyre::Result<()> {
        let (journal, _tmp) = test_journal(2);

        journal.commit(&TEST_DATA[1]);
        journal.commit(&TEST_DATA[2]);

        let err = journal
            .try_commit(&TEST_DATA[3])
            .expect_err("buffer should report full on third commit");
        assert!(
            matches!(err, crate::error::JournalUnavailable::Full),
            "expected Full, got {err:?}"
        );
        journal.shutdown()?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn reader_handles_sequential_reads() -> eyre::Result<()> {
        let (journal, _tmp) = test_journal(4);
        let mut reader = journal.reader();

        for entry in TEST_DATA.iter().take(4) {
            journal.commit(entry);
        }

        {
            let entries = reader.read(2);
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0], TEST_DATA[0]);
            assert_eq!(entries[1], TEST_DATA[1]);
        }
        reader.commit()?;

        for entry in TEST_DATA.iter().skip(4).take(2) {
            journal.commit(entry);
        }

        {
            let entries = reader.read(4);
            assert_eq!(entries.len(), 4);
            assert_eq!(entries[0], TEST_DATA[2]);
            assert_eq!(entries[1], TEST_DATA[3]);
            assert_eq!(entries[2], TEST_DATA[4]);
            assert_eq!(entries[3], TEST_DATA[5]);
        }

        reader.commit()?;
        assert_eq!(reader.available(), 0);
        journal.shutdown()?;
        Ok(())
    }
}

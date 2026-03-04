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

use crate::reader::JournalReader;
use rocksdb::{ColumnFamily, DB, Options, WriteBatch};
use std::{
    fmt,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    task::Waker,
};

/// Journal reader.
pub mod reader;

/// Error indicating that the journal buffer is not available now.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The journal is in a fatal state, caller should stop using it and drop it as soon as possible.
    #[error("fatal error happened")]
    Fatal,
    /// The journal buffer is full, new entries cannot be accepted until some entries are consumed
    /// and the buffer has space.
    #[error("journal buffer is full")]
    Full,
}

impl From<rocksdb::Error> for Error {
    fn from(e: rocksdb::Error) -> Self {
        error!("RocksDB error: {e}");
        Error::Fatal
    }
}

const CF_ENTRIES: &str = "entries";
const CF_META: &str = "meta";

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
}

impl<const ENTRY_SIZE: usize> fmt::Debug for Journal<ENTRY_SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Journal").finish()
    }
}

impl<const ENTRY_SIZE: usize> Journal<ENTRY_SIZE> {
    /// Create a new journal with the specified capacity and default configuration.
    pub fn with_capacity(capacity: usize) -> Result<Self, Error> {
        Self::with_capacity_and_config(capacity, JournalConfig::default())
    }

    /// Create a new journal with the specified capacity and configuration.
    pub fn with_capacity_and_config(capacity: usize, config: JournalConfig) -> Result<Self, Error> {
        let capacity = capacity as u64;

        let mut global_options = Options::default();
        global_options.create_if_missing(true);
        global_options.create_missing_column_families(true);

        let db = DB::open_cf(&global_options, &config.db_path, [CF_ENTRIES, CF_META])?;

        let inner = Arc::new(JournalInner {
            db,
            capacity,
            write_index: AtomicU64::new(0),
            consumed_index: AtomicU64::new(0),
            write_lock: Mutex::new(()),
            reader_taken: AtomicBool::new(false),
            consumer_wait: Mutex::new(None),
        });

        // Recover state
        let write_index = inner.read_write_index_from_db()?;
        let consumed_index = inner.read_consumed_index_from_db()?;
        if consumed_index > write_index {
            error!("Consumed index {consumed_index} is greater than write index {write_index}");
            return Err(Error::Fatal);
        }
        info!("Journal recovered: write_index={write_index}, consumed_index={consumed_index}");

        inner.set_write_index(write_index);
        inner.set_consumed_index(consumed_index);

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
    pub fn try_commit(&self, data: &[u8; ENTRY_SIZE]) -> Result<(), Error> {
        // Serialize writes so indices are strictly consecutive.
        let _guard = self.inner.write_lock.lock().unwrap();

        let write_idx = self.inner.write_index();
        let consumed = self.inner.consumed_index();

        if write_idx.wrapping_sub(consumed) >= self.inner.capacity {
            return Err(Error::Full);
        }

        let cf_entries = self.inner.cf_entries();
        // Write entry + updated write_index atomically via WriteBatch.
        let new_write_idx = write_idx.wrapping_add(1);
        let mut batch = WriteBatch::default();
        batch.put_cf(cf_entries, write_idx.to_be_bytes(), data);
        self.inner
            .write_write_index_batched(&mut batch, new_write_idx);
        self.inner.db.write(batch)?;

        self.inner.set_write_index(new_write_idx);

        // drop write_lock before notifying consumer
        drop(_guard);

        // Notify consumer if it is waiting for entries.
        self.inner.notify_consumer();

        Ok(())
    }
}

impl<const ENTRY_SIZE: usize> JournalInner<ENTRY_SIZE> {
    const META_WRITE_INDEX_KEY: &[u8] = &[0x00];
    const META_CONSUMED_INDEX_KEY: &[u8] = &[0x01];

    /// Wake the consumer waker if the write index has reached its target.
    fn notify_consumer(&self) {
        let mut guard = self.consumer_wait.lock().unwrap();
        if let Some(wait) = guard.as_ref()
            && self.write_index() >= wait.target_index
        {
            guard.take().unwrap().waker.wake();
        }
    }

    #[inline]
    pub(crate) fn consumed_index(&self) -> u64 {
        self.consumed_index.load(Ordering::Acquire)
    }

    #[inline]
    pub(crate) fn set_consumed_index(&self, idx: u64) {
        self.consumed_index.store(idx, Ordering::Release);
    }

    #[inline]
    pub(crate) fn write_index(&self) -> u64 {
        self.write_index.load(Ordering::Acquire)
    }

    #[inline]
    pub(crate) fn set_write_index(&self, idx: u64) {
        self.write_index.store(idx, Ordering::Release);
    }

    #[inline]
    pub(crate) fn cf_entries(&self) -> &ColumnFamily {
        self.db.cf_handle(CF_ENTRIES).expect("missing entries CF")
    }

    #[inline]
    pub(crate) fn cf_meta(&self) -> &ColumnFamily {
        self.db.cf_handle(CF_META).expect("missing meta CF")
    }

    #[inline]
    pub(crate) fn read_consumed_index_from_db(&self) -> Result<u64, Error> {
        self.read_meta(Self::META_CONSUMED_INDEX_KEY)
    }

    #[inline]
    pub(crate) fn read_write_index_from_db(&self) -> Result<u64, Error> {
        self.read_meta(Self::META_WRITE_INDEX_KEY)
    }

    #[inline]
    fn read_meta(&self, key: &[u8]) -> Result<u64, Error> {
        let cf = self.cf_meta();
        let Some(value) = self.db.get_cf(cf, key)? else {
            // If the key is missing, assume index 0 (fresh journal).
            return Ok(0);
        };
        if value.len() != 8 {
            error!(
                "Invalid meta value for key {:?}: expected 8 bytes, got {}",
                key,
                value.len()
            );
            return Err(Error::Fatal);
        }
        Ok(u64::from_le_bytes(value.as_slice().try_into().unwrap()))
    }

    #[inline]
    pub(crate) fn write_consumed_index_batched(&self, batch: &mut WriteBatch, new: u64) {
        self.write_meta_batched(Self::META_CONSUMED_INDEX_KEY, batch, new)
    }

    #[inline]
    pub(crate) fn write_write_index_batched(&self, batch: &mut WriteBatch, new: u64) {
        self.write_meta_batched(Self::META_WRITE_INDEX_KEY, batch, new)
    }

    #[inline]
    fn write_meta_batched(&self, key: &[u8], batch: &mut WriteBatch, new: u64) {
        batch.put_cf(self.cf_meta(), key, new.to_le_bytes())
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

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn commit_and_read_round_trip() -> eyre::Result<()> {
        let (journal, _tmp) = test_journal(4);
        let mut reader = journal.reader();

        journal.commit(&TEST_DATA[0]);
        journal.commit(&TEST_DATA[1]);

        {
            let entries = reader.read(2)?;
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0], TEST_DATA[0]);
            assert_eq!(entries[1], TEST_DATA[1]);
        }

        reader.commit()?;
        assert_eq!(reader.available(), 0);
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
            matches!(err, crate::Error::Full),
            "expected Full, got {err:?}"
        );
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
            let entries = reader.read(2)?;
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0], TEST_DATA[0]);
            assert_eq!(entries[1], TEST_DATA[1]);
        }
        reader.commit()?;

        for entry in TEST_DATA.iter().skip(4).take(2) {
            journal.commit(entry);
        }

        {
            let entries = reader.read(4)?;
            assert_eq!(entries.len(), 4);
            assert_eq!(entries[0], TEST_DATA[2]);
            assert_eq!(entries[1], TEST_DATA[3]);
            assert_eq!(entries[2], TEST_DATA[4]);
            assert_eq!(entries[3], TEST_DATA[5]);
        }

        reader.commit()?;
        assert_eq!(reader.available(), 0);
        Ok(())
    }
}

//! Journal implementation for UTS
#[macro_use]
extern crate tracing;

use crate::{
    checkpoint::{Checkpoint, CheckpointConfig},
    error::JournalUnavailable,
    reader::JournalReader,
    wal::Wal,
};
use std::{
    cell::UnsafeCell,
    fmt, io,
    ops::Deref,
    path::PathBuf,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    task::{Poll, Waker},
};

/// Checkpointing
pub mod checkpoint;
/// Error types.
pub mod error;
/// Journal reader.
pub mod reader;
/// Write-Ahead Log backend.
pub mod wal;

/// Configuration for the journal.
#[derive(Debug, Clone)]
pub struct JournalConfig {
    /// Configuration for the consumer checkpoint, which tracks the `consumed_index` of the journal.
    pub consumer_checkpoint: CheckpointConfig,
    /// Directory for the write-ahead log (WAL) backend, which is used for durability and recovery
    /// of the journal. The WAL will store committed entries that have not yet been persisted to the
    /// ring buffer, allowing the journal to recover from crashes without data loss.
    pub wal_dir: PathBuf,
}

impl Default for JournalConfig {
    fn default() -> Self {
        Self {
            consumer_checkpoint: CheckpointConfig::default(),
            wal_dir: PathBuf::from("wal"),
        }
    }
}

/// A `At-Least-Once` journal for storing fixed-size entries in a ring buffer.
///
/// All index here are monotonic u64, wrapping around on overflow.
///
/// Following invariants are maintained:
/// `consumed_index` <= `persisted_index` <= `write_index`.
#[derive(Clone)]
pub struct Journal<const ENTRY_SIZE: usize> {
    inner: Arc<JournalInner<{ ENTRY_SIZE }>>,
    /// Wal backend for recovery.
    wal: Wal<{ ENTRY_SIZE }>,
}

pub(crate) struct JournalInner<const ENTRY_SIZE: usize> {
    /// The ring buffer storing the entries.
    /// The capacity of the ring buffer, **MUST** be power of two.
    buffer: Box<[UnsafeCell<[u8; ENTRY_SIZE]>]>,
    /// The co-ring buffer storing the wakers.
    /// The capacity of the ring buffer, **MUST** be power of two.
    waker_buffer: Box<[WakerEntry]>,
    /// Mask for indexing into the ring buffer.
    index_mask: u64,
    /// Next Write Position, aka:
    /// - Total entries reserved count.
    /// - Position to write the next entry to.
    write_index: AtomicU64,
    /// Filled Boundary, aka:
    /// - Total entries that have been fully written to the ring buffer.
    /// - Advanced in order after each writer finishes copying data into its reserved slot.
    /// - The WAL worker uses this (not `write_index`) to determine how far it can safely read.
    filled_index: AtomicU64,
    /// WAL Committed Boundary, aka.:
    /// - Total committed entries count.
    /// - Position has not yet been persisted to durable storage.
    persisted_index: AtomicU64,
    /// Free Boundary, aka.:
    /// - Total consumed entries count.
    /// - Position that has not yet been consumed by readers.
    consumed_checkpoint: Checkpoint,
    /// Whether a reader has taken ownership of this journal.
    reader_taken: AtomicBool,
    /// Waker for the consumer to notify new persisted entries.
    consumer_wait: Mutex<Option<ConsumerWait>>,
    /// Shutdown flag
    shutdown: AtomicBool,
}

unsafe impl<const ENTRY_SIZE: usize> Sync for JournalInner<ENTRY_SIZE> {}
unsafe impl<const ENTRY_SIZE: usize> Send for JournalInner<ENTRY_SIZE> {}

impl<const ENTRY_SIZE: usize> fmt::Debug for Journal<ENTRY_SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Journal").finish()
    }
}

impl<const ENTRY_SIZE: usize> Journal<ENTRY_SIZE> {
    /// Create a new journal with the specified capacity.
    ///
    /// The capacity will be rounded up to the next power of two.
    pub fn with_capacity(capacity: usize) -> io::Result<Self> {
        Self::with_capacity_and_config(capacity, JournalConfig::default())
    }

    /// Create a new journal with the specified capacity.
    ///
    /// The capacity will be rounded up to the next power of two.
    pub fn with_capacity_and_config(capacity: usize, config: JournalConfig) -> io::Result<Self> {
        let capacity = capacity.next_power_of_two();
        let index_mask = capacity as u64 - 1;

        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize_with(capacity, || UnsafeCell::new([0u8; ENTRY_SIZE]));
        let buffer = buffer.into_boxed_slice();

        let mut waker_buffer = Vec::with_capacity(capacity);
        waker_buffer.resize_with(capacity, Default::default);
        let waker_buffer = waker_buffer.into_boxed_slice();

        let inner = Arc::new(JournalInner {
            buffer,
            waker_buffer,
            index_mask,
            write_index: AtomicU64::new(0),
            filled_index: AtomicU64::new(0),
            persisted_index: AtomicU64::new(0),
            consumed_checkpoint: Checkpoint::new(config.consumer_checkpoint)?,
            reader_taken: AtomicBool::new(false),
            consumer_wait: Mutex::new(None),
            shutdown: AtomicBool::new(false),
        });

        let wal = Wal::new(config.wal_dir, inner.clone())?;

        Ok(Self { inner, wal })
    }

    /// Get the capacity of the journal.
    #[inline]
    fn capacity(&self) -> usize {
        self.inner.capacity()
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
    /// # Panics
    ///
    /// Panics if:
    /// - the journal is full.
    /// - the journal is shut down.
    pub fn commit(&self, data: &[u8; ENTRY_SIZE]) -> CommitFuture<'_, ENTRY_SIZE> {
        self.try_commit(data).expect("Journal buffer is full")
    }

    /// Try commit a new entry to the journal.
    ///
    /// Returns a future that resolves when the entry has been safely persisted.
    /// Returns `BufferFull` error if the journal is full.
    pub fn try_commit(
        &self,
        data: &[u8; ENTRY_SIZE],
    ) -> Result<CommitFuture<'_, ENTRY_SIZE>, JournalUnavailable> {
        if self.inner.shutdown.load(Ordering::Acquire) {
            return Err(JournalUnavailable::Shutdown);
        }

        let mut current_written = self.inner.write_index.load(Ordering::Relaxed);
        loop {
            // 1. Check if there is space in the buffer.
            let consumed = self.inner.consumed_checkpoint.current_index();
            if current_written.wrapping_sub(consumed) >= self.capacity() as u64 {
                return Err(JournalUnavailable::Full);
            }

            // 2. Try to reserve a slot.
            match self.inner.write_index.compare_exchange_weak(
                current_written,
                current_written.wrapping_add(1),
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_written = actual,
            }
        }

        // 3. Write the data to the slot.
        let slot = unsafe { &mut *self.data_slot_ptr(current_written) };
        slot.copy_from_slice(data);

        // 4. Publish the filled slot.
        //    Spin-wait until all prior slots are filled, then advance `filled_index`.
        //    The Release ordering ensures the slot write above is visible to the WAL worker
        //    before it reads `filled_index`.
        while self
            .inner
            .filled_index
            .compare_exchange_weak(
                current_written,
                current_written.wrapping_add(1),
                Ordering::Release,
                Ordering::Relaxed,
            )
            .is_err()
        {
            std::hint::spin_loop();
        }

        // 5. Notify WAL worker if needed.
        let committed = self.inner.persisted_index.load(Ordering::Relaxed);
        // Explain: If there is no pending committed entry before ours,
        // the WAL worker may be sleeping, so we need to wake it up.
        if current_written == committed {
            // Notify the WAL worker thread to persist new entries.
            self.wal.unpark();
        }

        Ok(CommitFuture {
            journal: self,
            slot: current_written,
            active_waker: None,
        })
    }

    /// Shut down the journal, flushing all checkpoints and shutting down the WAL.
    pub fn shutdown(&self) -> io::Result<()> {
        self.inner.shutdown.store(true, Ordering::SeqCst);

        self.inner.consumed_checkpoint.flush()?;
        self.wal.shutdown();
        Ok(())
    }

    /// Get a mut ptr to the slot at the given index.
    #[inline]
    fn data_slot_ptr(&self, index: u64) -> *mut [u8; ENTRY_SIZE] {
        self.inner.data_slot_ptr(index)
    }

    /// Get a ref to the waker entry at the given index.
    #[inline]
    fn waker_slot(&self, index: u64) -> &WakerEntry {
        self.inner.waker_slot(index)
    }
}

impl<const ENTRY_SIZE: usize> JournalInner<ENTRY_SIZE> {
    /// Get the capacity of the journal.
    #[inline]
    fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Get a mut ptr to the slot at the given index.
    #[inline]
    const fn data_slot_ptr(&self, index: u64) -> *mut [u8; ENTRY_SIZE] {
        let slot_idx = index & self.index_mask;
        self.buffer[slot_idx as usize].get()
    }

    /// Get a ref to the waker entry at the given index.
    #[inline]
    const fn waker_slot(&self, index: u64) -> &WakerEntry {
        let slot_idx = index & self.index_mask;
        &self.waker_buffer[slot_idx as usize]
    }
}

/// Future returned by `Journal::commit` representing the commit operation.
/// The future resolves when the entry has been safely persisted.
#[derive(Debug)]
pub struct CommitFuture<'a, const ENTRY_SIZE: usize> {
    journal: &'a Journal<ENTRY_SIZE>,
    slot: u64,
    /// Whether the waker has been registered.
    active_waker: Option<Waker>,
}

impl<const ENTRY_SIZE: usize> Future for CommitFuture<'_, ENTRY_SIZE> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if self.journal.inner.persisted_index.load(Ordering::Acquire) > self.slot {
            return Poll::Ready(());
        }

        let should_register = match &self.active_waker {
            None => true,
            // waker changed, need to update, rare case
            Some(w) => !w.will_wake(cx.waker()),
        };

        if should_register {
            let entry = self.journal.waker_slot(self.slot);
            let mut guard = entry.lock().expect("Mutex poisoned");

            if self.journal.inner.persisted_index.load(Ordering::Acquire) > self.slot {
                return Poll::Ready(());
            }

            *guard = Some(cx.waker().clone());
            self.active_waker = Some(cx.waker().clone());
        }

        if self.journal.inner.persisted_index.load(Ordering::Acquire) > self.slot {
            return Poll::Ready(());
        }

        Poll::Pending
    }
}

/// A consumer wait entry.
struct ConsumerWait {
    waker: Waker,
    target_index: u64,
}

/// A waker entry in the co-ring buffer.
///
/// Aligned to cache line size to prevent false sharing.
#[derive(Default)]
#[repr(C, align(64))]
struct WakerEntry(Mutex<Option<Waker>>);

impl Deref for WakerEntry {
    type Target = Mutex<Option<Waker>>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::checkpoint::CheckpointConfig;

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

    /// Create a journal with an isolated temporary directory for WAL and checkpoint files.
    /// Returns the journal and the temp dir guard (must be kept alive for the test duration).
    pub fn test_journal(capacity: usize) -> (Journal, tempfile::TempDir) {
        let tmp = tempfile::tempdir().expect("failed to create temp dir");
        let config = crate::JournalConfig {
            consumer_checkpoint: CheckpointConfig {
                path: tmp.path().join("checkpoint.meta"),
                ..Default::default()
            },
            wal_dir: tmp.path().join("wal"),
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

        journal.commit(&TEST_DATA[0]).await;
        journal.commit(&TEST_DATA[1]).await;

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

        journal.commit(&TEST_DATA[1]).await;
        journal.commit(&TEST_DATA[2]).await;

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
    async fn reader_handles_wrap_around_reads() -> eyre::Result<()> {
        let (journal, _tmp) = test_journal(4);
        let mut reader = journal.reader();

        for entry in TEST_DATA.iter().take(4) {
            journal.commit(entry).await;
        }

        {
            let entries = reader.read(2);
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0], TEST_DATA[0]);
            assert_eq!(entries[1], TEST_DATA[1]);
        }
        reader.commit()?;

        for entry in TEST_DATA.iter().skip(4).take(2) {
            journal.commit(entry).await;
        }

        {
            let entries = reader.read(4);
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0], TEST_DATA[2]);
            assert_eq!(entries[1], TEST_DATA[3]);
        }

        {
            let entries = reader.read(4);
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0], TEST_DATA[4]);
            assert_eq!(entries[1], TEST_DATA[5]);
        }

        reader.commit()?;
        assert_eq!(reader.available(), 0);
        journal.shutdown()?;
        Ok(())
    }
}

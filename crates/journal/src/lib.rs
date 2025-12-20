//! Journal implementation for UTS

use crate::{
    error::BufferFull,
    reader::JournalReader,
    wal::{DummyWal, Wal},
};
use std::{
    cell::UnsafeCell,
    fmt,
    ops::Deref,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    task::{Poll, Waker},
};

/// Error types.
pub mod error;
/// Journal reader.
pub mod reader;
/// Write-Ahead Log backend.
pub mod wal;

/// A journal for storing fixed-size entries in a ring buffer.
///
/// All index here are monotonic u64, wrapping around on overflow.
///
/// Following invariants are maintained:
/// `consumed_index` <= `persisted_index` <= `write_index`
#[derive(Clone)]
pub struct Journal<const ENTRY_SIZE: usize> {
    inner: Arc<JournalInner<{ ENTRY_SIZE }>>,
    /// Wal backend for recovery.
    wal: Box<dyn Wal>,
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
    /// - Total entries written count.
    /// - Position to write the next entry to.
    write_index: AtomicU64,
    /// WAL Committed Boundary, aka.:
    /// - Total committed entries count.
    /// - Position has not yet been persisted to durable storage.
    persisted_index: AtomicU64,
    /// Free Boundary, aka.:
    /// - Total consumed entries count.
    /// - Position that has not yet been consumed by readers.
    consumed_index: AtomicU64,
    /// Whether a reader has taken ownership of this journal.
    reader_taken: AtomicBool,
    /// Waker for the consumer to notify new persisted entries.
    consumer_wait: Mutex<Option<ConsumerWait>>,
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
    pub fn with_capacity(capacity: usize) -> Self {
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
            persisted_index: AtomicU64::new(0),
            consumed_index: AtomicU64::new(0),
            reader_taken: AtomicBool::new(false),
            consumer_wait: Mutex::new(None),
        });

        let wal = Box::new(DummyWal::new(inner.clone()));

        Self { inner, wal }
    }

    /// Get the capacity of the journal.
    #[inline]
    fn capacity(&self) -> usize {
        self.inner.buffer.len()
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
    pub fn commit(
        &self,
        data: &[u8; ENTRY_SIZE],
    ) -> Result<CommitFuture<'_, ENTRY_SIZE>, BufferFull> {
        let mut current_written = self.inner.write_index.load(Ordering::Relaxed);
        loop {
            // 1. Check if there is space in the buffer.
            let consumed = self.inner.consumed_index.load(Ordering::Acquire);
            if current_written.wrapping_sub(consumed) >= self.capacity() as u64 {
                return Err(BufferFull);
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

        // 4. Notify WAL worker if needed.
        let committed = self.inner.persisted_index.load(Ordering::Relaxed);
        // Explain: Before we wrote to the slot, if there is no pending committed entry,
        // There's a chance the WAL worker is sleeping, we need to wake it up.
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

impl<'a, const ENTRY_SIZE: usize> Future for CommitFuture<'a, ENTRY_SIZE> {
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

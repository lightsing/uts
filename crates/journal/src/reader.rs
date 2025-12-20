use crate::{ConsumerWait, JournalInner};
use std::{
    fmt,
    pin::Pin,
    sync::{Arc, atomic::Ordering},
    task::{Context, Poll},
};

/// A reader for consuming settled entries from the journal.
pub struct JournalReader<const ENTRY_SIZE: usize> {
    journal: Arc<JournalInner<ENTRY_SIZE>>,
    consumed: u64,
}

impl<const ENTRY_SIZE: usize> fmt::Debug for JournalReader<ENTRY_SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JournalReader")
            .field("consumed", &self.consumed)
            .finish()
    }
}

impl<const ENTRY_SIZE: usize> Drop for JournalReader<ENTRY_SIZE> {
    fn drop(&mut self) {
        self.journal.reader_taken.store(false, Ordering::Release);
    }
}

impl<const ENTRY_SIZE: usize> JournalReader<ENTRY_SIZE> {
    pub(super) fn new(journal: Arc<JournalInner<ENTRY_SIZE>>) -> Self {
        let consumed = journal.consumed_index.load(Ordering::Acquire);
        Self { journal, consumed }
    }

    /// Returns the number of available entries are settled but not yet consumed by this reader.
    #[inline]
    pub fn available(&self) -> usize {
        let persisted = self.journal.persisted_index.load(Ordering::Acquire);
        persisted.wrapping_sub(self.consumed) as usize
    }

    /// Wait until at least `min` entries are available.
    pub async fn wait_at_least(&self, min: usize) {
        if self.available() >= min {
            return;
        }

        let target_index = self.consumed.wrapping_add(min as u64);

        // Slow path
        struct WaitForBatch<'a, const ENTRY_SIZE: usize> {
            reader: &'a JournalReader<ENTRY_SIZE>,
            target_index: u64,
        }

        impl<'a, const ENTRY_SIZE: usize> Future for WaitForBatch<'a, ENTRY_SIZE> {
            type Output = ();
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                if self.reader.journal.persisted_index.load(Ordering::Acquire) >= self.target_index
                {
                    return Poll::Ready(());
                }

                let mut guard = self
                    .reader
                    .journal
                    .consumer_wait
                    .lock()
                    .expect("Mutex poisoned");
                if self.reader.journal.persisted_index.load(Ordering::Acquire) >= self.target_index
                {
                    return Poll::Ready(());
                }
                *guard = Some(ConsumerWait {
                    waker: cx.waker().clone(),
                    target_index: self.target_index,
                });

                Poll::Pending
            }
        }

        WaitForBatch {
            reader: self,
            target_index,
        }
        .await;
    }

    /// Read available entries, up to `max`.
    /// Bumps the internal consumed index by the number of entries yielded.
    ///
    /// Caller is responsible for calling `commit()` after processing the entries.
    pub fn read(&mut self, max: usize) -> &[[u8; ENTRY_SIZE]] {
        let available = self.available();
        if available == 0 {
            return &[];
        }

        let count = available.min(max);
        let start_idx = self.consumed;

        // handle wrap-around
        let buffer_len = self.journal.buffer.len();
        let slot_idx = (start_idx & self.journal.index_mask) as usize;
        let continuous_len = count.min(buffer_len - slot_idx);

        // push local consumed index
        self.consumed += continuous_len as u64;

        // return slice
        let ptr = self.journal.buffer[slot_idx].get();
        // SAFETY: bounds checked above
        unsafe { std::slice::from_raw_parts(ptr, continuous_len) }
    }

    /// Commit current consumed index.
    pub fn commit(&self) {
        self.journal
            .consumed_index
            .store(self.consumed, Ordering::Release);
    }
}

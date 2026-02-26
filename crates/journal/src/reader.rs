use crate::{ConsumerWait, JournalInner};
use std::{
    fmt, io,
    pin::Pin,
    sync::{Arc, atomic::Ordering},
    task::{Context, Poll},
};

/// A reader for consuming settled entries from the journal.
///
/// Reader **WON'T** advance the shared consumed boundary until `commit()` is called.
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
        let consumed = journal.consumed_checkpoint.current_index();
        Self { journal, consumed }
    }

    /// Returns the number of available entries that are settled but not yet consumed by this reader.
    #[inline]
    pub fn available(&self) -> usize {
        if self.journal.shutdown.load(Ordering::Acquire) {
            return 0;
        }
        let persisted = self.journal.persisted_index.load(Ordering::Acquire);
        persisted.wrapping_sub(self.consumed) as usize
    }

    /// Wait until at least `min` entries are available.
    pub async fn wait_at_least(&mut self, min: usize) {
        if self.available() >= min {
            return;
        }

        let target_index = self.consumed.wrapping_add(min as u64);
        {
            // panics if the target_index exceeds buffer size, otherwise we might wait forever
            // this happens if:
            // - asks for more entries than the buffer can hold
            // - didn't commit previously read entries, then asks for more than new entries than the buffer can hold
            // this is considered a misuse of the API / design flaw in the caller, so we panics
            let journal_buffer_size = self.journal.buffer.len() as u64;
            let current_consumed = self.journal.consumed_checkpoint.current_index();
            let max_possible_target = current_consumed.wrapping_add(journal_buffer_size);
            if target_index > max_possible_target {
                panic!(
                    "requested ({target_index}) exceeds max possible ({max_possible_target}): journal.buffer.len()={journal_buffer_size}, journal.consumed_index={current_consumed}"
                );
            }
        }

        // Slow path
        struct WaitForBatch<'a, const ENTRY_SIZE: usize> {
            reader: &'a JournalReader<ENTRY_SIZE>,
            target_index: u64,
        }

        impl<const ENTRY_SIZE: usize> Future for WaitForBatch<'_, ENTRY_SIZE> {
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
    pub fn commit(&mut self) -> io::Result<()> {
        self.journal.consumed_checkpoint.update(self.consumed)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::*;
    use tokio::time::{Duration, sleep, timeout};

    #[tokio::test(flavor = "current_thread")]
    async fn available_tracks_persisted_entries() -> eyre::Result<()> {
        let journal = Journal::with_capacity(4)?;
        let mut reader = journal.reader();

        assert_eq!(reader.available(), 0);

        journal.commit(&TEST_DATA[0]).await;
        assert_eq!(reader.available(), 1);

        journal.commit(&TEST_DATA[1]).await;
        assert_eq!(reader.available(), 2);

        let slice = reader.read(1);
        assert_eq!(slice.len(), 1);
        assert_eq!(reader.available(), 1);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn commit_updates_shared_consumed_boundary() -> eyre::Result<()> {
        let journal = Journal::with_capacity(4)?;
        let mut reader = journal.reader();

        for entry in TEST_DATA.iter().take(3) {
            journal.commit(entry).await;
        }

        let slice = reader.read(2);
        assert_eq!(slice.len(), 2);
        assert_eq!(reader.available(), 1);
        assert_eq!(
            reader.journal.consumed_checkpoint.current_index(),
            0,
            "global consumed boundary should not advance before commit",
        );

        reader.commit()?;
        assert_eq!(reader.journal.consumed_checkpoint.current_index(), 2);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn wait_at_least_resumes_after_persistence() -> eyre::Result<()> {
        let journal = Journal::with_capacity(4)?;
        let mut reader = journal.reader();

        let journal_clone = journal.clone();
        let task = tokio::spawn(async move {
            sleep(Duration::from_millis(5)).await;
            journal_clone.commit(&TEST_DATA[0]).await;
        });

        reader.wait_at_least(1).await;
        assert_eq!(reader.available(), 1);

        task.await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn wait_at_least_waits_for_correct_count() -> eyre::Result<()> {
        let journal = Journal::with_capacity(4)?;
        let mut reader = journal.reader();

        let journal_clone = journal.clone();
        let task = tokio::spawn(async move {
            for entry in TEST_DATA.iter().take(4) {
                journal_clone.commit(entry).await;
                sleep(Duration::from_millis(5)).await;
            }
        });

        timeout(Duration::from_secs(1), reader.wait_at_least(3)).await?;
        assert!(reader.available() >= 3);

        task.await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[should_panic(
        expected = "requested (5) exceeds max possible (4): journal.buffer.len()=4, journal.consumed_index=0"
    )]
    async fn wait_at_least_exceeds_buffer_size() {
        let journal = Journal::with_capacity(4).unwrap();
        let mut reader = journal.reader();

        timeout(Duration::from_secs(1), reader.wait_at_least(5))
            .await
            .unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    #[should_panic(
        expected = "requested (5) exceeds max possible (4): journal.buffer.len()=4, journal.consumed_index=0"
    )]
    async fn wait_at_least_dirty_read_exceeds_available() {
        let journal = Journal::with_capacity(4).unwrap();
        journal.commit(&TEST_DATA[0]).await;

        let mut reader = journal.reader();
        reader.read(1);

        timeout(Duration::from_secs(1), reader.wait_at_least(4))
            .await
            .unwrap();
    }
}

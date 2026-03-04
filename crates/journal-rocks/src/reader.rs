use crate::{ConsumerWait, Error, JournalInner, helper::FatalErrorExt};
use rocksdb::{Direction, IteratorMode, ReadOptions, WriteBatch};
use std::{
    fmt,
    pin::Pin,
    sync::{Arc, atomic::Ordering},
    task::{Context, Poll},
};

/// A reader for consuming settled entries from the journal.
///
/// Reader **WON'T** advance the shared consumed boundary until `commit()` is
/// called.  Entries are fetched from RocksDB into an internal buffer on each
/// [`read`](JournalReader::read) call.
pub struct JournalReader<const ENTRY_SIZE: usize> {
    journal: Arc<JournalInner<ENTRY_SIZE>>,
    /// Local consumed cursor – how far this reader has read.
    consumed: u64,
    /// Internal buffer populated by [`read`](JournalReader::read).
    read_buf: Vec<[u8; ENTRY_SIZE]>,
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
        let consumed = journal.consumed_index();
        Self {
            journal,
            consumed,
            read_buf: Vec::new(),
        }
    }

    /// Returns the number of available entries that have been written but not
    /// yet consumed by this reader.
    #[inline]
    pub fn available(&self) -> usize {
        let write_idx = self.journal.write_index();
        write_idx.wrapping_sub(self.consumed) as usize
    }

    /// Wait until at least `min` entries are available.
    pub async fn wait_at_least(&mut self, min: usize) -> Result<(), Error> {
        if self.journal.fatal_error.load(Ordering::Acquire) {
            return Err(Error::Fatal);
        }
        if self.available() >= min {
            return Ok(());
        }

        let target_index = self.consumed.wrapping_add(min as u64);
        {
            let capacity = self.journal.capacity;
            let current_consumed = self.journal.consumed_index();
            let max_possible_target = current_consumed.wrapping_add(capacity);
            if target_index > max_possible_target {
                panic!(
                    "requested ({target_index}) exceeds max possible ({max_possible_target}): journal.capacity={capacity}, journal.consumed_index={current_consumed}"
                );
            }
        }

        // Slow path – register a waker and park until the writer catches up.
        struct WaitForBatch<'a, const ENTRY_SIZE: usize> {
            reader: &'a JournalReader<ENTRY_SIZE>,
            target_index: u64,
        }

        impl<const ENTRY_SIZE: usize> Future for WaitForBatch<'_, ENTRY_SIZE> {
            type Output = Result<(), Error>;
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                if self.reader.journal.fatal_error.load(Ordering::Acquire) {
                    return Poll::Ready(Err(Error::Fatal));
                }

                if self.reader.journal.write_index() >= self.target_index {
                    return Poll::Ready(Ok(()));
                }

                let mut guard = self
                    .reader
                    .journal
                    .consumer_wait
                    .lock()
                    .expect("Mutex poisoned");
                if self.reader.journal.write_index() >= self.target_index {
                    return Poll::Ready(Ok(()));
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
        .await
    }

    /// Read available entries from RocksDB, up to `max`.
    ///
    /// Bumps the internal consumed cursor by the number of entries yielded.
    /// Caller is responsible for calling [`commit`](JournalReader::commit)
    /// after processing the entries.
    pub fn read(&mut self, max: usize) -> Result<&[[u8; ENTRY_SIZE]], Error> {
        if self.journal.fatal_error.load(Ordering::Acquire) {
            return Err(Error::Fatal);
        }

        let available = self.available();
        if available == 0 {
            return Ok(&[]);
        }

        let count = available.min(max);
        self.read_buf.clear();

        let start_key = self.consumed.to_be_bytes();
        let end_key = (self.consumed + count as u64).to_be_bytes();

        let mut options = ReadOptions::default();
        options.set_iterate_lower_bound(start_key);
        options.set_iterate_upper_bound(end_key);
        options.set_auto_readahead_size(true);

        let iter = self.journal.db.iterator_cf_opt(
            self.journal.cf_entries(),
            options,
            IteratorMode::From(&start_key, Direction::Forward),
        );
        for (idx, data) in iter.enumerate() {
            let (_key, value) = data.stop_if_error(&self.journal)?;
            debug_assert_eq!((self.consumed + idx as u64).to_be_bytes(), _key.as_ref(),);
            if value.len() != ENTRY_SIZE {
                error!("entry at index {idx} has wrong size");
                return Err(Error::Fatal);
            }
            let mut entry = [0u8; ENTRY_SIZE];
            entry.copy_from_slice(&value);
            self.read_buf.push(entry);
        }

        self.consumed += count as u64;
        Ok(&self.read_buf)
    }

    /// Commit the current consumed index, persisting it to RocksDB and
    /// deleting consumed entries.
    pub fn commit(&mut self) -> Result<(), Error> {
        if self.journal.fatal_error.load(Ordering::Acquire) {
            return Err(Error::Fatal);
        }

        let old_consumed = self.journal.consumed_index();

        let mut batch = WriteBatch::default();
        self.journal
            .write_consumed_index_batched(&mut batch, self.consumed);
        // Garbage-collect consumed entries.
        batch.delete_range(old_consumed.to_be_bytes(), self.consumed.to_be_bytes());
        if let Err(e) = self.journal.db.write(batch) {
            error!("failed to commit consumed and run gc RocksDB: {e}");
            return Err(Error::Fatal);
        }

        self.journal
            .consumed_index
            .store(self.consumed, Ordering::Release);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::*;
    use tokio::time::{Duration, sleep, timeout};

    #[tokio::test(flavor = "current_thread")]
    async fn available_tracks_written_entries() -> eyre::Result<()> {
        let (journal, _tmp) = test_journal(4);
        let mut reader = journal.reader();

        assert_eq!(reader.available(), 0);

        journal.commit(&TEST_DATA[0]);
        assert_eq!(reader.available(), 1);

        journal.commit(&TEST_DATA[1]);
        assert_eq!(reader.available(), 2);

        let slice = reader.read(1)?;
        assert_eq!(slice.len(), 1);
        assert_eq!(reader.available(), 1);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn commit_updates_shared_consumed_boundary() -> eyre::Result<()> {
        let (journal, _tmp) = test_journal(4);
        let mut reader = journal.reader();

        for entry in TEST_DATA.iter().take(3) {
            journal.commit(entry);
        }

        let slice = reader.read(2)?;
        assert_eq!(slice.len(), 2);
        assert_eq!(reader.available(), 1);
        assert_eq!(
            reader
                .journal
                .consumed_index
                .load(std::sync::atomic::Ordering::Acquire),
            0,
            "global consumed boundary should not advance before commit",
        );

        reader.commit()?;
        assert_eq!(
            reader
                .journal
                .consumed_index
                .load(std::sync::atomic::Ordering::Acquire),
            2
        );
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn wait_at_least_resumes_after_write() -> eyre::Result<()> {
        let (journal, _tmp) = test_journal(4);
        let mut reader = journal.reader();

        let journal_clone = journal.clone();
        let task = tokio::spawn(async move {
            sleep(Duration::from_millis(5)).await;
            journal_clone.commit(&TEST_DATA[0]);
        });

        reader.wait_at_least(1).await;
        assert_eq!(reader.available(), 1);

        task.await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn wait_at_least_waits_for_correct_count() -> eyre::Result<()> {
        let (journal, _tmp) = test_journal(4);
        let mut reader = journal.reader();

        let journal_clone = journal.clone();
        let task = tokio::spawn(async move {
            for entry in TEST_DATA.iter().take(4) {
                journal_clone.commit(entry);
                sleep(Duration::from_millis(5)).await;
            }
        });

        timeout(Duration::from_secs(10), reader.wait_at_least(3)).await?;
        assert!(reader.available() >= 3);

        task.await?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[should_panic(
        expected = "requested (5) exceeds max possible (4): journal.capacity=4, journal.consumed_index=0"
    )]
    async fn wait_at_least_exceeds_capacity() {
        let (journal, _tmp) = test_journal(4);
        let mut reader = journal.reader();

        timeout(Duration::from_secs(1), reader.wait_at_least(5))
            .await
            .unwrap();
    }

    #[tokio::test(flavor = "current_thread")]
    #[should_panic(
        expected = "requested (5) exceeds max possible (4): journal.capacity=4, journal.consumed_index=0"
    )]
    async fn wait_at_least_dirty_read_exceeds_available() {
        let (journal, _tmp) = test_journal(4);
        journal.commit(&TEST_DATA[0]);

        let mut reader = journal.reader();
        reader.read(1).unwrap();

        timeout(Duration::from_secs(1), reader.wait_at_least(4))
            .await
            .unwrap();
    }
}

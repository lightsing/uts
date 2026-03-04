use crate::{ConsumerWait, JournalInner};
use rocksdb::WriteBatch;
use std::{
    fmt, io,
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
        let consumed = journal.consumed_index.load(Ordering::Acquire);
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
        if self.journal.shutdown.load(Ordering::Acquire) {
            return 0;
        }
        let write_idx = self.journal.write_index.load(Ordering::Acquire);
        write_idx.wrapping_sub(self.consumed) as usize
    }

    /// Wait until at least `min` entries are available.
    pub async fn wait_at_least(&mut self, min: usize) {
        if self.available() >= min {
            return;
        }

        let target_index = self.consumed.wrapping_add(min as u64);
        {
            let capacity = self.journal.capacity;
            let current_consumed = self.journal.consumed_index.load(Ordering::Acquire);
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
            type Output = ();
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
                if self.reader.journal.write_index.load(Ordering::Acquire) >= self.target_index {
                    return Poll::Ready(());
                }

                let mut guard = self
                    .reader
                    .journal
                    .consumer_wait
                    .lock()
                    .expect("Mutex poisoned");
                if self.reader.journal.write_index.load(Ordering::Acquire) >= self.target_index {
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

    /// Read available entries from RocksDB, up to `max`.
    ///
    /// Bumps the internal consumed cursor by the number of entries yielded.
    /// Caller is responsible for calling [`commit`](JournalReader::commit)
    /// after processing the entries.
    pub fn read(&mut self, max: usize) -> &[[u8; ENTRY_SIZE]] {
        let available = self.available();
        if available == 0 {
            return &[];
        }

        let count = available.min(max);
        self.read_buf.clear();

        for i in 0..count as u64 {
            let idx = self.consumed + i;
            let data = self
                .journal
                .db
                .get(idx.to_le_bytes())
                .expect("RocksDB read failed")
                .unwrap_or_else(|| panic!("missing journal entry at index {idx}"));
            assert_eq!(
                data.len(),
                ENTRY_SIZE,
                "entry at index {idx} has wrong size"
            );
            let mut entry = [0u8; ENTRY_SIZE];
            entry.copy_from_slice(&data);
            self.read_buf.push(entry);
        }

        self.consumed += count as u64;
        &self.read_buf
    }

    /// Commit the current consumed index, persisting it to RocksDB and
    /// deleting consumed entries.
    pub fn commit(&mut self) -> io::Result<()> {
        let old_consumed = self.journal.consumed_index.load(Ordering::Acquire);

        let mut batch = WriteBatch::default();
        batch.put(crate::META_CONSUMED_INDEX_KEY, self.consumed.to_le_bytes());
        // Garbage-collect consumed entries.
        for idx in old_consumed..self.consumed {
            batch.delete(idx.to_le_bytes());
        }
        self.journal
            .db
            .write(batch)
            .map_err(|e| io::Error::other(e.to_string()))?;

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

        let slice = reader.read(1);
        assert_eq!(slice.len(), 1);
        assert_eq!(reader.available(), 1);
        journal.shutdown()?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn commit_updates_shared_consumed_boundary() -> eyre::Result<()> {
        let (journal, _tmp) = test_journal(4);
        let mut reader = journal.reader();

        for entry in TEST_DATA.iter().take(3) {
            journal.commit(entry);
        }

        let slice = reader.read(2);
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
        journal.shutdown()?;
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
        journal.shutdown()?;
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
        journal.shutdown()?;
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
        reader.read(1);

        timeout(Duration::from_secs(1), reader.wait_at_least(4))
            .await
            .unwrap();
    }
}

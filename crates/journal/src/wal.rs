use crate::JournalInner;
use std::{
    fmt,
    fmt::Formatter,
    fs,
    fs::{File, OpenOptions},
    io,
    io::{BufWriter, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    thread::JoinHandle,
};

const MAX_SPIN: usize = 100;
const MAX_IO_BATCH: u64 = 128;

/// Write-Ahead Log
///
/// Busy-Wait + Parking when there's no work to do.
///
/// Using segmented log files named as `{base_dir}/{segment_id}.wal`, where `segment_id` is a
/// monotonically increasing integer.
///
/// Each segment file contains a fixed number of entries
/// (at least to be the size of the journal buffer) to simplify recovery and management.
#[derive(Clone)]
pub struct Wal<const ENTRY_SIZE: usize> {
    inner: Arc<WalInner<ENTRY_SIZE>>,
}

struct WalInner<const ENTRY_SIZE: usize> {
    worker: Mutex<Option<JoinHandle<()>>>,
    journal: Arc<JournalInner<ENTRY_SIZE>>,
    shutdown_flag: Arc<AtomicBool>,
}

impl<const ENTRY_SIZE: usize> Drop for WalInner<ENTRY_SIZE> {
    fn drop(&mut self) {
        // Signal the WAL worker to exit if it hasn't been shut down yet.
        // This prevents orphaned worker threads from spinning after the journal is dropped.
        if !self.shutdown_flag.swap(true, Ordering::AcqRel) {
            if let Some(worker) = self.worker.lock().unwrap().as_ref() {
                worker.thread().unpark();
            }
        }
    }
}

impl<const ENTRY_SIZE: usize> fmt::Debug for Wal<ENTRY_SIZE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Wal")
            .field(
                "write_index",
                &self.inner.journal.write_index.load(Ordering::Acquire),
            )
            .field(
                "persisted_index",
                &self.inner.journal.persisted_index.load(Ordering::Acquire),
            )
            .finish()
    }
}

impl<const ENTRY_SIZE: usize> Wal<ENTRY_SIZE> {
    /// Create a new WAL instance with the given base directory for storing log segments and
    /// a reference to the journal. This will recover existing segments from the base directory,
    /// and start a background worker thread to handle persistence of log entries.
    pub(crate) fn new<P: AsRef<Path>>(
        base_dir: P,
        journal: Arc<JournalInner<ENTRY_SIZE>>,
    ) -> io::Result<Self> {
        let base_dir = base_dir.as_ref();
        fs::create_dir_all(base_dir)?;
        if !base_dir.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Base path is not a directory",
            ));
        }

        let shutdown_flag = Arc::new(AtomicBool::new(false));

        let current_segment_id = recover(base_dir, &journal)?;
        let mut current_file = open_segment_file(base_dir, current_segment_id)?;
        current_file.seek(SeekFrom::End(0))?;

        let worker = WalWorker {
            current_segment_id,
            current_file: BufWriter::new(current_file),

            base_dir: base_dir.to_path_buf(),
            journal: journal.clone(),
            shutdown_flag: shutdown_flag.clone(),
        };

        let handle = thread::Builder::new()
            .name("wal-worker".to_string())
            .spawn(move || worker.run())
            .expect("Failed to spawn WAL worker thread");

        let inner = WalInner {
            worker: Mutex::new(Some(handle)),
            journal,
            shutdown_flag,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    /// Unpark the WAL worker thread to wake it up if it's parked.
    /// This should be called after new entries are written to the journal, to ensure that the
    /// worker thread can persist them in a timely manner.
    pub fn unpark(&self) {
        if self.inner.shutdown_flag.load(Ordering::Acquire) {
            return;
        }
        self.inner
            .worker
            .lock()
            .unwrap()
            .as_ref()
            .expect("WAL worker thread should be running")
            .thread()
            .unpark();
    }

    /// Shut down the WAL worker thread gracefully. This will set the shutdown flag, unpark the
    /// worker thread if it's parked, and wait for it to finish.
    pub fn shutdown(&self) {
        if self.inner.shutdown_flag.swap(true, Ordering::AcqRel) {
            // already shutdown
            return;
        }
        let worker = self
            .inner
            .worker
            .lock()
            .unwrap()
            .take()
            .expect("WAL worker thread should be running");
        worker.thread().unpark();
        worker.join().expect("Failed to join WAL worker thread");
    }
}

struct WalWorker<const ENTRY_SIZE: usize> {
    current_segment_id: u64,
    current_file: BufWriter<File>,

    base_dir: PathBuf,
    journal: Arc<JournalInner<ENTRY_SIZE>>,
    shutdown_flag: Arc<AtomicBool>,
}

impl<const ENTRY_SIZE: usize> WalWorker<ENTRY_SIZE> {
    fn run(self) {
        let Self {
            mut current_segment_id,
            mut current_file,
            base_dir,
            journal,
            shutdown_flag,
        } = self;

        let segment_size = journal.capacity() as u64;

        let mut persisted = journal.persisted_index.load(Ordering::Acquire);
        let mut spin_count = 0;

        while !shutdown_flag.load(Ordering::Acquire) {
            let filled = journal.filled_index.load(Ordering::Acquire);
            let available = filled.wrapping_sub(persisted);

            if available > 0 {
                spin_count = 0;

                let available = available.min(MAX_IO_BATCH);
                let target_index = persisted + available;

                // write ALL available entries to segment files, rotating files as needed
                for i in persisted..target_index {
                    let seg_id = i / segment_size;
                    if seg_id != current_segment_id {
                        // rotate to new segment file
                        current_segment_id = seg_id;
                        let new_file = open_segment_file(&base_dir, seg_id)
                            .expect("Failed to open WAL segment file");
                        current_file = BufWriter::new(new_file);
                        let base_dir = base_dir.clone();
                        thread::spawn(move || truncate_old_segments(base_dir, seg_id));
                    }

                    current_file
                        .write_all(unsafe { &*journal.data_slot_ptr(i) })
                        .expect("Failed to write to WAL segment file");
                }
                current_file
                    .flush()
                    .expect("Failed to flush WAL segment file");
                current_file
                    .get_ref()
                    .sync_all()
                    .expect("Failed to sync WAL segment file");

                // notify waiters only after data is persisted
                for i in persisted..target_index {
                    let entry = journal.waker_slot(i);
                    if let Some(waker) = entry.lock().unwrap().take() {
                        waker.wake();
                    }
                }

                // update persisted index
                persisted = target_index;
                journal.persisted_index.store(persisted, Ordering::Release);

                // notify consumer if needed
                let mut guard = journal.consumer_wait.lock().unwrap();
                if let Some(wait) = guard.as_ref() {
                    // Only wake if the target_index is reached
                    if persisted >= wait.target_index {
                        guard.take().unwrap().waker.wake();
                    }
                }

                continue;
            }

            // no new data to persist, spin for a while before parking
            if spin_count <= MAX_SPIN {
                spin_count += 1;
                std::hint::spin_loop();
                continue;
            }

            // park until unparked by a new commit
            thread::park();
        }
    }
}

fn recover<const ENTRY_SIZE: usize>(
    base_dir: &Path,
    journal: &JournalInner<ENTRY_SIZE>,
) -> io::Result<u64> {
    let mut segments = scan_segments(base_dir)?;
    if segments.is_empty() {
        info!("No WAL segments found, starting fresh");
        return Ok(0);
    }

    // remove the last segment for recovery, as it may be incomplete
    let last_segment_id = segments.pop().expect("segments is not empty");

    let segment_size = journal.capacity() as u64;
    let complete_segment_size = segment_size * ENTRY_SIZE as u64;
    for segment_id in segments.iter().copied() {
        let path = base_dir.join(format_segment_file_name(segment_id));
        let metadata = fs::metadata(&path)?;
        let file_size = metadata.len();

        if file_size != complete_segment_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Incomplete WAL segment: {}", path.display()),
            ));
        }
    }

    // handle last segment, which may be partially written
    let path = base_dir.join(format_segment_file_name(last_segment_id));
    let metadata = fs::metadata(&path)?;
    let file_size = metadata.len();
    let valid_count = file_size / ENTRY_SIZE as u64;
    if file_size % ENTRY_SIZE as u64 != 0 {
        warn!("Detected partial write in last segment #{last_segment_id}. Truncating.");
        let f = OpenOptions::new().write(true).open(&path)?;
        f.set_len(valid_count * ENTRY_SIZE as u64)?;
        f.sync_all()?;
    }

    let write_index = segments.len() as u64 * segment_size + valid_count;
    // consumed_checkpoint just recovered.
    let consumed_index = journal.consumed_checkpoint.persisted_index();
    // Data loss happens, don't continue to recover, as it may cause more damage.
    // User intervention is needed to fix the issue.
    if consumed_index > write_index {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Consumed index {consumed_index} is greater than recovered write index {write_index}"
            ),
        ));
    }

    journal.write_index.store(write_index, Ordering::Relaxed);
    journal.filled_index.store(write_index, Ordering::Relaxed);
    journal
        .persisted_index
        .store(write_index, Ordering::Relaxed);
    info!("WAL Recovered: write_index={write_index}, consumed_index={consumed_index}");

    // replay data to ring buffer
    replay_data(&base_dir, journal)?;
    Ok(last_segment_id)
}

fn scan_segments(base_dir: &Path) -> io::Result<Vec<u64>> {
    let mut segments: Vec<u64> = Vec::new();

    let entries = fs::read_dir(&base_dir)?;

    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_file() {
            continue;
        }

        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str().and_then(|n| n.strip_suffix(".wal")) else {
            warn!(
                "Skipping non-WAL file during recovery: {}",
                entry.path().display()
            );
            continue;
        };

        let Ok(id) = file_name.parse::<u64>() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid WAL file name: {}", entry.path().display()),
            ));
        };
        segments.push(id);
    }
    segments.sort_unstable();

    Ok(segments)
}

fn replay_data<const ENTRY_SIZE: usize>(
    base_dir: &Path,
    journal: &JournalInner<{ ENTRY_SIZE }>,
) -> io::Result<()> {
    let segment_size = journal.capacity() as u64;
    let entry_size = ENTRY_SIZE as u64;

    let write_index = journal.write_index.load(Ordering::Relaxed);
    let consumed_index = journal.consumed_checkpoint.persisted_index();

    let mut current_file: Option<File> = None;
    let mut current_seg_id: u64 = u64::MAX;

    for idx in consumed_index..write_index {
        let seg_id = idx / segment_size;

        if seg_id != current_seg_id {
            current_file = Some(open_segment_file(&base_dir, seg_id)?);
            current_seg_id = seg_id;
        }

        let offset = (idx % segment_size) * entry_size;
        let slot_ptr = journal.data_slot_ptr(idx);

        if let Some(ref mut f) = current_file {
            f.seek(SeekFrom::Start(offset))?;
            let buffer = unsafe { &mut *slot_ptr };
            f.read_exact(buffer)?;
        }
    }
    Ok(())
}

#[inline]
fn format_segment_file_name(segment_id: u64) -> String {
    format!("{segment_id:012}.wal")
}

fn open_segment_file(base_dir: &Path, segment_id: u64) -> io::Result<File> {
    let path = base_dir.join(format_segment_file_name(segment_id));
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path)
}

fn truncate_old_segments(base_dir: PathBuf, current_segment_id: u64) -> io::Result<()> {
    let Some(to_delete) = current_segment_id.checked_sub(2) else {
        // not segments to truncate
        return Ok(());
    };
    let path = base_dir.join(format_segment_file_name(to_delete));
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        JournalConfig,
        checkpoint::CheckpointConfig,
        tests::{ENTRY_SIZE, TEST_DATA, test_journal},
    };
    use std::sync::atomic::Ordering;

    type Journal = crate::Journal<ENTRY_SIZE>;

    /// Helper: create a journal config pointing at the given temp directory.
    fn test_config(tmp: &Path) -> JournalConfig {
        JournalConfig {
            consumer_checkpoint: CheckpointConfig {
                path: tmp.join("checkpoint.meta"),
                ..Default::default()
            },
            wal_dir: tmp.join("wal"),
        }
    }

    // ── Segment file helpers ─────────────────────────────────────────────

    #[test]
    fn format_segment_file_name_pads_to_twelve_digits() {
        assert_eq!(format_segment_file_name(0), "000000000000.wal");
        assert_eq!(format_segment_file_name(1), "000000000001.wal");
        assert_eq!(
            format_segment_file_name(999_999_999_999),
            "999999999999.wal"
        );
    }

    #[test]
    fn open_segment_file_creates_file() -> io::Result<()> {
        let tmp = tempfile::tempdir()?;
        let f = open_segment_file(tmp.path(), 0)?;
        assert!(tmp.path().join("000000000000.wal").exists());
        drop(f);
        Ok(())
    }

    // ── scan_segments ────────────────────────────────────────────────────

    #[test]
    fn scan_segments_empty_directory() -> io::Result<()> {
        let tmp = tempfile::tempdir()?;
        let segments = scan_segments(tmp.path())?;
        assert!(segments.is_empty());
        Ok(())
    }

    #[test]
    fn scan_segments_returns_sorted_ids() -> io::Result<()> {
        let tmp = tempfile::tempdir()?;
        // Create files out of order.
        File::create(tmp.path().join("000000000002.wal"))?;
        File::create(tmp.path().join("000000000000.wal"))?;
        File::create(tmp.path().join("000000000001.wal"))?;

        let segments = scan_segments(tmp.path())?;
        assert_eq!(segments, vec![0, 1, 2]);
        Ok(())
    }

    #[test]
    fn scan_segments_skips_non_wal_files() -> io::Result<()> {
        let tmp = tempfile::tempdir()?;
        File::create(tmp.path().join("000000000000.wal"))?;
        File::create(tmp.path().join("readme.txt"))?;

        let segments = scan_segments(tmp.path())?;
        assert_eq!(segments, vec![0]);
        Ok(())
    }

    #[test]
    fn scan_segments_rejects_invalid_wal_names() -> io::Result<()> {
        let tmp = tempfile::tempdir()?;
        File::create(tmp.path().join("not_a_number.wal"))?;

        let err = scan_segments(tmp.path()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        Ok(())
    }

    // ── truncate_old_segments ────────────────────────────────────────────

    #[test]
    fn truncate_old_segments_removes_two_behind() -> io::Result<()> {
        let tmp = tempfile::tempdir()?;
        File::create(tmp.path().join(format_segment_file_name(0)))?;
        File::create(tmp.path().join(format_segment_file_name(1)))?;
        File::create(tmp.path().join(format_segment_file_name(2)))?;

        truncate_old_segments(tmp.path().to_path_buf(), 2)?;
        assert!(
            !tmp.path().join(format_segment_file_name(0)).exists(),
            "segment 0 should be deleted when current is 2"
        );
        assert!(tmp.path().join(format_segment_file_name(1)).exists());
        assert!(tmp.path().join(format_segment_file_name(2)).exists());
        Ok(())
    }

    #[test]
    fn truncate_old_segments_noop_when_too_few() -> io::Result<()> {
        let tmp = tempfile::tempdir()?;
        File::create(tmp.path().join(format_segment_file_name(0)))?;
        File::create(tmp.path().join(format_segment_file_name(1)))?;

        // current_segment_id = 1, checked_sub(2) underflows → noop
        truncate_old_segments(tmp.path().to_path_buf(), 1)?;
        assert!(tmp.path().join(format_segment_file_name(0)).exists());
        assert!(tmp.path().join(format_segment_file_name(1)).exists());
        Ok(())
    }

    // ── WAL persistence end-to-end ───────────────────────────────────────

    #[tokio::test(flavor = "current_thread")]
    async fn wal_persists_entries_to_segment_file() -> eyre::Result<()> {
        let (journal, tmp) = test_journal(4);

        journal.commit(&TEST_DATA[0]).await;
        journal.commit(&TEST_DATA[1]).await;

        // WAL file should exist and contain the two entries.
        let wal_path = tmp.path().join("wal").join(format_segment_file_name(0));
        assert!(wal_path.exists(), "WAL segment file should exist");

        let data = fs::read(&wal_path)?;
        assert_eq!(
            data.len(),
            ENTRY_SIZE * 2,
            "segment should contain exactly 2 entries"
        );
        assert_eq!(&data[..ENTRY_SIZE], &TEST_DATA[0]);
        assert_eq!(&data[ENTRY_SIZE..ENTRY_SIZE * 2], &TEST_DATA[1]);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn wal_segment_rotation() -> eyre::Result<()> {
        // Capacity 4 → segment_size = 4 entries per segment.
        let (journal, tmp) = test_journal(4);
        let mut reader = journal.reader();

        // Fill first segment (4 entries).
        for entry in TEST_DATA.iter().take(4) {
            journal.commit(entry).await;
        }

        // Consume some entries to free ring buffer space.
        reader.read(4);
        reader.commit()?;

        // Write one more entry, causing rotation to segment 1.
        journal.commit(&TEST_DATA[4]).await;

        let seg0 = tmp.path().join("wal").join(format_segment_file_name(0));
        let seg1 = tmp.path().join("wal").join(format_segment_file_name(1));
        assert!(seg0.exists(), "segment 0 should exist");
        assert!(seg1.exists(), "segment 1 should exist after rotation");

        let seg1_data = fs::read(&seg1)?;
        assert_eq!(seg1_data.len(), ENTRY_SIZE);
        assert_eq!(&seg1_data[..ENTRY_SIZE], &TEST_DATA[4]);
        Ok(())
    }

    // ── Recovery ─────────────────────────────────────────────────────────

    #[tokio::test(flavor = "current_thread")]
    async fn recover_from_empty_dir() -> eyre::Result<()> {
        let tmp = tempfile::tempdir()?;
        let config = test_config(tmp.path());

        // First journal writes nothing.
        let journal = Journal::with_capacity_and_config(4, config.clone())?;
        journal.shutdown()?;
        drop(journal);

        // Second journal recovers with indices at 0.
        let journal = Journal::with_capacity_and_config(4, config)?;
        assert_eq!(journal.inner.write_index.load(Ordering::Acquire), 0);
        assert_eq!(journal.inner.persisted_index.load(Ordering::Acquire), 0);
        journal.shutdown()?;
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn recover_replays_data_into_ring_buffer() -> eyre::Result<()> {
        let tmp = tempfile::tempdir()?;
        let config = test_config(tmp.path());

        // Write entries and shut down.
        {
            let journal = Journal::with_capacity_and_config(4, config.clone())?;
            journal.commit(&TEST_DATA[0]).await;
            journal.commit(&TEST_DATA[1]).await;
            journal.commit(&TEST_DATA[2]).await;
            journal.shutdown()?;
        }

        // Recover and verify data via reader.
        {
            let journal = Journal::with_capacity_and_config(4, config)?;
            assert_eq!(journal.inner.write_index.load(Ordering::Acquire), 3);
            assert_eq!(journal.inner.persisted_index.load(Ordering::Acquire), 3);

            let mut reader = journal.reader();
            let entries = reader.read(3);
            assert_eq!(entries.len(), 3);
            assert_eq!(entries[0], TEST_DATA[0]);
            assert_eq!(entries[1], TEST_DATA[1]);
            assert_eq!(entries[2], TEST_DATA[2]);
            journal.shutdown()?;
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn recover_respects_consumed_checkpoint() -> eyre::Result<()> {
        let tmp = tempfile::tempdir()?;
        let config = test_config(tmp.path());

        // Write entries, consume some, then shut down.
        {
            let journal = Journal::with_capacity_and_config(4, config.clone())?;
            journal.commit(&TEST_DATA[0]).await;
            journal.commit(&TEST_DATA[1]).await;
            journal.commit(&TEST_DATA[2]).await;
            let mut reader = journal.reader();
            reader.read(2);
            reader.commit()?;
            journal.shutdown()?;
        }

        // Recover - reader should only see unconsumed entries.
        {
            let journal = Journal::with_capacity_and_config(4, config)?;
            assert_eq!(journal.inner.write_index.load(Ordering::Acquire), 3);
            let mut reader = journal.reader();
            let entries = reader.read(10);
            assert_eq!(entries.len(), 1, "only 1 unconsumed entry should remain");
            assert_eq!(entries[0], TEST_DATA[2]);
            journal.shutdown()?;
        }
        Ok(())
    }

    #[test]
    fn recover_truncates_partial_last_segment() -> io::Result<()> {
        let tmp = tempfile::tempdir()?;
        let wal_dir = tmp.path().join("wal");
        fs::create_dir_all(&wal_dir)?;

        // Write 2.5 entries worth of data (partial entry at end).
        let mut data = Vec::new();
        data.extend_from_slice(&TEST_DATA[0]);
        data.extend_from_slice(&TEST_DATA[1]);
        data.extend_from_slice(&[0xFF; ENTRY_SIZE / 2]); // partial write
        fs::write(wal_dir.join(format_segment_file_name(0)), &data)?;

        let config = test_config(tmp.path());
        let journal = Journal::with_capacity_and_config(4, config).unwrap();

        // Should recover 2 valid entries (truncated partial write).
        assert_eq!(journal.inner.write_index.load(Ordering::Acquire), 2);
        assert_eq!(journal.inner.persisted_index.load(Ordering::Acquire), 2);

        // Verify truncated file size.
        let file_size = fs::metadata(wal_dir.join(format_segment_file_name(0)))?.len();
        assert_eq!(file_size, (ENTRY_SIZE * 2) as u64);
        journal.shutdown().unwrap();
        Ok(())
    }

    #[test]
    fn recover_detects_incomplete_non_last_segment() {
        let tmp = tempfile::tempdir().unwrap();
        let wal_dir = tmp.path().join("wal");
        fs::create_dir_all(&wal_dir).unwrap();

        // Segment 0 is incomplete (not full), segment 1 exists.
        // For capacity 4, a full segment should be 4 * ENTRY_SIZE bytes.
        fs::write(
            wal_dir.join(format_segment_file_name(0)),
            &[0u8; ENTRY_SIZE * 2],
        )
        .unwrap();
        fs::write(wal_dir.join(format_segment_file_name(1)), &[0u8; 0]).unwrap();

        let config = test_config(tmp.path());
        let err = Journal::with_capacity_and_config(4, config).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn recover_detects_consumed_ahead_of_written() {
        let tmp = tempfile::tempdir().unwrap();
        let wal_dir = tmp.path().join("wal");
        fs::create_dir_all(&wal_dir).unwrap();

        // Write a checkpoint claiming index 10 consumed...
        let checkpoint_path = tmp.path().join("checkpoint.meta");
        fs::write(&checkpoint_path, &10u64.to_le_bytes()).unwrap();

        // ...but WAL only has 2 entries.
        let mut data = Vec::new();
        data.extend_from_slice(&TEST_DATA[0]);
        data.extend_from_slice(&TEST_DATA[1]);
        fs::write(wal_dir.join(format_segment_file_name(0)), &data).unwrap();

        let config = JournalConfig {
            consumer_checkpoint: CheckpointConfig {
                path: checkpoint_path,
                ..Default::default()
            },
            wal_dir,
        };
        let err = Journal::with_capacity_and_config(4, config).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    // ── Shutdown ─────────────────────────────────────────────────────────

    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_rejects_new_commits() -> eyre::Result<()> {
        let (journal, _tmp) = test_journal(4);

        journal.commit(&TEST_DATA[0]).await;
        journal.shutdown()?;

        let err = journal
            .try_commit(&TEST_DATA[1])
            .expect_err("commit after shutdown should fail");
        assert!(
            matches!(err, crate::error::JournalUnavailable::Shutdown),
            "expected Shutdown, got {err:?}"
        );
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_is_idempotent() -> eyre::Result<()> {
        let (journal, _tmp) = test_journal(4);
        journal.commit(&TEST_DATA[0]).await;
        journal.shutdown()?;
        journal.shutdown()?;
        Ok(())
    }

    // ── WAL worker wakes on commit ───────────────────────────────────────

    #[tokio::test(flavor = "current_thread")]
    async fn commit_future_resolves_after_wal_persistence() -> eyre::Result<()> {
        let (journal, _tmp) = test_journal(4);

        // The commit future should only resolve after the WAL worker persists.
        journal.commit(&TEST_DATA[0]).await;

        // If we got here, the persisted_index must have advanced.
        assert!(
            journal.inner.persisted_index.load(Ordering::Acquire) >= 1,
            "persisted_index should be >= 1 after commit future resolves"
        );
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn multiple_commits_advance_persisted_index() -> eyre::Result<()> {
        let (journal, _tmp) = test_journal(4);

        for i in 0..4 {
            journal.commit(&TEST_DATA[i]).await;
        }

        assert_eq!(journal.inner.persisted_index.load(Ordering::Acquire), 4);
        Ok(())
    }

    // ── Recovery across segments ─────────────────────────────────────────

    #[tokio::test(flavor = "current_thread")]
    async fn recover_across_segment_rotation() -> eyre::Result<()> {
        let tmp = tempfile::tempdir()?;
        let config = test_config(tmp.path());

        // Fill more than one segment. Capacity 4 → segment_size = 4.
        {
            let journal = Journal::with_capacity_and_config(4, config.clone())?;
            let mut reader = journal.reader();

            // Fill first segment.
            for entry in TEST_DATA.iter().take(4) {
                journal.commit(entry).await;
            }

            // Consume to free ring buffer.
            reader.read(4);
            reader.commit()?;

            // Write into second segment.
            journal.commit(&TEST_DATA[4]).await;
            journal.commit(&TEST_DATA[5]).await;

            // Consume second batch.
            reader.read(2);
            reader.commit()?;

            journal.shutdown()?;
        }

        // Recover and write more.
        {
            let journal = Journal::with_capacity_and_config(4, config)?;
            assert_eq!(journal.inner.write_index.load(Ordering::Acquire), 6);

            // Should be able to write new entries.
            journal.commit(&TEST_DATA[6]).await;
            assert_eq!(journal.inner.persisted_index.load(Ordering::Acquire), 7);
            journal.shutdown()?;
        }
        Ok(())
    }
}

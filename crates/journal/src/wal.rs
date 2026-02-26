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

const MAX_SPINS: usize = 10_000;
const IO_BATCH_LIMIT: u64 = 128;

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

        let mut persisted = 0;
        let mut spins = 0;

        while !shutdown_flag.load(Ordering::Acquire) {
            let written = journal.write_index.load(Ordering::Acquire);
            let available = written.wrapping_sub(persisted);

            if available > 0 {
                // reset spins counter
                spins = 0;

                let available = available.min(IO_BATCH_LIMIT);
                let target_index = persisted + available;

                // write entries to segment files, rotating files as needed
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

            // busy-wait + park
            if spins < MAX_SPINS {
                // busy-wait
                std::hint::spin_loop();
                spins += 1;
            } else {
                // park the thread
                thread::park();
                // reset spins counter on wake
                spins = 0;
            }
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

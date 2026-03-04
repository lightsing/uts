use std::{
    fs,
    fs::File,
    io,
    io::Read,
    path::{Path, PathBuf},
    sync::{Mutex, atomic::AtomicU64},
    time::{Duration, Instant},
};

/// Configuration for checkpointing mechanism.
#[derive(Debug, Clone)]
pub struct CheckpointConfig {
    /// Path to the checkpoint file where the last persisted index is stored. This file will be
    /// created if it does not exist, and updated atomically when a new checkpoint is flushed to
    /// disk.
    pub path: PathBuf,
    /// Flush interval for checkpointing, used to determine when to flush the persisted checkpoint
    /// to disk.
    pub min_interval: Duration,
    /// Flush threshold for checkpointing, used to determine when to flush the persisted checkpoint
    /// to disk based on the number of new entries since the last flush.
    pub min_advance: u64,
    /// Suffix for temporary checkpoint file when performing checkpointing. The checkpoint will be
    /// atomically renamed to the final checkpoint file after flush.
    pub temp_suffix: &'static str,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("checkpoint.meta"),
            min_interval: Duration::from_secs(1),
            min_advance: 128,
            temp_suffix: ".tmp",
        }
    }
}

/// Checkpointing for tracking `consumed_index`.
#[derive(Debug)]
pub struct Checkpoint {
    config: CheckpointConfig,
    current: AtomicU64,
    inner: Mutex<CheckpointInner>,
}

#[derive(Debug)]
struct CheckpointInner {
    temp_path: PathBuf,

    /// The index of the last persisted checkpoint. This is updated when a new checkpoint is
    /// flushed to disk.
    persisted_index: u64,
    last_flush_time: Instant,
}

impl Checkpoint {
    /// Creates a new checkpoint instance with the given configuration. This will attempt to recover
    /// the last persisted checkpoint index from disk, and initialize the internal state accordingly.
    #[instrument(skip_all, err)]
    pub fn new(config: CheckpointConfig) -> io::Result<Self> {
        let parent = config.path.parent().ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "parent directory does not exist",
        ))?;
        fs::create_dir_all(parent)?;

        let mut inner = CheckpointInner {
            temp_path: config.path.with_added_extension(config.temp_suffix),

            persisted_index: 0,
            last_flush_time: Instant::now(),
        };
        let recovered = inner.recover(&config)?;

        Ok(Self {
            config,
            current: AtomicU64::new(recovered),
            inner: Mutex::new(inner),
        })
    }

    /// Returns the last persisted checkpoint index, which is updated when a new checkpoint is
    /// flushed to disk. This requires acquiring the lock on the inner state, and may lag behind
    /// the current index until the next flush to disk.
    #[instrument(skip(self), ret)]
    pub fn persisted_index(&self) -> u64 {
        let inner = self.inner.lock().unwrap();
        inner.persisted_index
    }

    /// Returns the current checkpoint index, which may be ahead of the last persisted index.
    /// This is updated atomically when `update` is called, and can be read without acquiring the
    /// lock on the inner state.
    ///
    /// The persisted index may lag behind the current index until the next flush to disk.
    #[instrument(skip(self), ret)]
    pub fn current_index(&self) -> u64 {
        self.current.load(std::sync::atomic::Ordering::Acquire)
    }

    /// Updates the current checkpoint index. This will trigger a flush to disk if:
    /// - the new index has advanced by at least `min_advance` since the last persisted index
    /// - or, the time since the last flush has exceeded `min_interval`.
    #[instrument(skip(self), err)]
    pub fn update(&self, new_index: u64) -> io::Result<()> {
        let mut inner = self.inner.lock().unwrap();
        self.current
            .store(new_index, std::sync::atomic::Ordering::Release);
        inner.update(new_index, &self.config, false)
    }

    /// Flush the current checkpoint to disk immediately, regardless of the configured flush
    /// interval and flush threshold.
    #[instrument(skip(self), err)]
    pub fn flush(&self) -> io::Result<()> {
        let mut inner = self.inner.lock().unwrap();
        let new_index = self.current.load(std::sync::atomic::Ordering::Acquire);
        inner.update(new_index, &self.config, true)
    }
}

impl CheckpointInner {
    #[instrument(skip(self), err)]
    fn recover(&mut self, config: &CheckpointConfig) -> io::Result<u64> {
        // Try to recover from the temp checkpoint file first
        if let Ok(index) = recover_from_disk(&self.temp_path) {
            self.persisted_index = index;
            fs::rename(&self.temp_path, &config.path)?;
        } else {
            match recover_from_disk(&config.path) {
                Ok(index) => self.persisted_index = index,
                Err(e) if e.kind() == io::ErrorKind::NotFound => self.persisted_index = 0,
                Err(e) => return Err(e),
            }
        }
        Ok(self.persisted_index)
    }

    fn update(
        &mut self,
        new_index: u64,
        config: &CheckpointConfig,
        forced: bool,
    ) -> io::Result<()> {
        if new_index <= self.persisted_index {
            warn!(
                "New checkpoint index {} is not greater than persisted index {}, skipping update",
                new_index, self.persisted_index
            );
            return Ok(());
        }

        let now = Instant::now();
        let should_flush = new_index - self.persisted_index >= config.min_advance;
        let timeouts = now.duration_since(self.last_flush_time) >= config.min_interval;
        if forced || should_flush || timeouts {
            fs::write(&self.temp_path, new_index.to_le_bytes())?;
            fs::rename(&self.temp_path, &config.path)?;
            self.persisted_index = new_index;
            self.last_flush_time = now;
        }
        Ok(())
    }
}

fn recover_from_disk(path: &Path) -> io::Result<u64> {
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    if metadata.len() != 8 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid checkpoint file",
        ));
    }
    let mut buf = [0u8; 8];
    file.read_exact(&mut buf)?;
    let index = u64::from_le_bytes(buf);
    Ok(index)
}

//! Journal implementation for UTS

use std::cell::UnsafeCell;
use std::sync::Mutex;
use std::task::Waker;

/// Specialized writer for journal entries.
/// e.g. io-uring, O_DIRECT
pub trait JournalWriter {

}

#[derive(Copy, Clone, Debug)]
pub struct JournalConfig {
    /// Maximum number of entries per commit group.
    pub max_entries_per_group: usize,
    /// Maximum total size of entries per commit group, in bytes.
    /// Default is 64 KiB.
    pub max_group_size_bytes: usize,
    /// Maximum time to wait for more entries to fill a commit group, in milliseconds.
    pub max_wait_time_ms: u64,
}

impl Default for JournalConfig {
    /// Optimized for 32 byte entries (e.g., SHA-256 hashes).
    fn default() -> Self {
        Self {
            max_entries_per_group: 2048,
            max_group_size_bytes: 64 * 1024,
            max_wait_time_ms: 10,
        }
    }
}

/// Journal that writes entries to a writer.
pub struct Journal<W: JournalWriter> {
    writer: W,
    config: JournalConfig,
    group: RingCommitGroupBuffer,
}

/// A group of journal entries to be committed together.
pub struct RingCommitGroupBuffer {
   // ..
}


//! A module that maintains a globally accessible current time in seconds since the Unix epoch.
//!
//! This is for performance optimization to avoid frequent syscalls for time retrieval.
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

static CURRENT_TIME_SEC: AtomicU64 = AtomicU64::new(0);

/// Returns the current time in seconds since the Unix epoch.
#[inline]
pub fn current_time_sec() -> u64 {
    CURRENT_TIME_SEC.load(Ordering::Relaxed)
}

/// An asynchronous task that updates the current time every second.
pub async fn updater() {
    let mut sleep = tokio::time::interval(Duration::from_secs(1));
    loop {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        CURRENT_TIME_SEC.store(now, Ordering::Relaxed);
        sleep.tick().await;
    }
}

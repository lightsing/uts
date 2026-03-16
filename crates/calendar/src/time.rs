//! A module that maintains a globally accessible current time in seconds since the Unix epoch.
//!
//! This is for performance optimization to avoid frequent syscalls for time retrieval.
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::time::MissedTickBehavior;

static CURRENT_TIME_SEC: AtomicU64 = AtomicU64::new(0);
const UPDATE_PERIOD: Duration = Duration::from_secs(1);

/// Returns the current time in seconds since the Unix epoch.
#[inline]
pub fn current_time_sec() -> u64 {
    CURRENT_TIME_SEC.load(Ordering::Relaxed)
}

/// An asynchronous task that updates the current time every second.
pub async fn async_updater() {
    let mut interval = tokio::time::interval(UPDATE_PERIOD);
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        CURRENT_TIME_SEC.store(now, Ordering::Relaxed);
        interval.tick().await;
    }
}

/// A task that updates the current time every second.
pub fn updater() {
    let mut next_tick = Instant::now();

    loop {
        next_tick += UPDATE_PERIOD;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        CURRENT_TIME_SEC.store(now, Ordering::Relaxed);

        // Note: This behavior is different from the async version, which skips missed ticks.
        let now_instant = Instant::now();
        if next_tick > now_instant {
            let now = Instant::now();

            if let Some(delay) = next_tick.checked_duration_since(now) {
                std::thread::sleep(delay);
            }
        } else {
            // If we've fallen behind, resynchronize to avoid accumulating drift.
            next_tick = now_instant;
        }
    }
}

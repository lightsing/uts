use crate::JournalInner;
use dyn_clone::DynClone;
use std::{
    sync::{Arc, atomic::Ordering},
    thread,
    thread::JoinHandle,
    time::Duration,
};

const MAX_SPINS: usize = 10_000;
const IO_BATCH_LIMIT: u64 = 128;

/// Write-Ahead Log Trait
///
/// Busy-Wait + Parking when there's no work to do.
pub trait Wal: DynClone + Send + Sync + 'static {
    /// Unpark the WAL worker thread to notify new data is available.
    fn unpark(&self);
    /// Shutdown the WAL worker thread.
    fn shutdown(&self) {
        // Default implementation does nothing.
        // Specific implementations can override this method to provide shutdown logic.
    }
}

dyn_clone::clone_trait_object!(Wal);

#[derive(Clone)]
pub(crate) struct DummyWal<const ENTRY_SIZE: usize> {
    worker: Arc<JoinHandle<()>>,
}

struct WalWorker<const ENTRY_SIZE: usize> {
    journal: Arc<JournalInner<ENTRY_SIZE>>,
}

impl<const ENTRY_SIZE: usize> DummyWal<ENTRY_SIZE> {
    pub(crate) fn new(journal: Arc<JournalInner<ENTRY_SIZE>>) -> Self {
        let worker = WalWorker { journal };
        let worker = thread::spawn(move || {
            worker.run();
        });
        Self {
            worker: Arc::new(worker),
        }
    }
}

impl<const ENTRY_SIZE: usize> Wal for DummyWal<ENTRY_SIZE> {
    fn unpark(&self) {
        self.worker.thread().unpark();
    }
}

impl<const ENTRY_SIZE: usize> WalWorker<ENTRY_SIZE> {
    fn run(self) {
        let Self { journal } = self;

        let mut persisted = 0;
        let mut spins = 0;

        loop {
            let written = journal.write_index.load(Ordering::Acquire);
            let available = written.wrapping_sub(persisted);

            if available > 0 {
                // reset spins counter
                spins = 0;

                // take as much as we can, limited by IO_BATCH_LIMIT
                let batch_size = available.min(IO_BATCH_LIMIT);
                let target_index = persisted + batch_size;

                // simulate IO
                // TODO: replace with real IO
                thread::sleep(Duration::from_millis(1));

                // notify waiters only after data is "persisted"
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

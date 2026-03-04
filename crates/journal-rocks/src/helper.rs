use crate::{Error, JournalInner};

pub(crate) trait FatalErrorExt<T> {
    fn stop_if_error(self, journal: &JournalInner) -> Result<T, Error>;
}

impl<T> FatalErrorExt<T> for Result<T, rocksdb::Error> {
    fn stop_if_error(self, journal: &JournalInner) -> Result<T, Error> {
        match self {
            Ok(val) => Ok(val),
            Err(e) => {
                error!("RocksDB error: {e}");
                journal
                    .fatal_error
                    .store(true, std::sync::atomic::Ordering::Release);
                journal.notify_consumer();
                Err(Error::Fatal)
            }
        }
    }
}

/// Error indicating that the journal buffer is not available now.
#[derive(Debug, thiserror::Error)]
pub enum JournalUnavailable {
    /// The journal is shutting down, no new entries can be accepted.
    #[error("journal is shutting down")]
    Shutdown,
    /// The journal buffer is full, new entries cannot be accepted until some entries are consumed
    /// and the buffer has space.
    #[error("journal buffer is full")]
    Full,
}

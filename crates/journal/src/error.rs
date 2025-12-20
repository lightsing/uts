use std::fmt;

/// Error indicating that the journal buffer is full.
#[derive(Debug)]
pub struct BufferFull;

impl fmt::Display for BufferFull {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Journal buffer is full")
    }
}

impl std::error::Error for BufferFull {}

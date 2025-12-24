use core::fmt;

/// Zero-allocation wrapper that displays byte slices as lowercase hex.
pub struct Hexed<'a, T: ?Sized>(pub &'a T);

impl<'a, T: ?Sized + AsRef<[u8]>> fmt::Display for Hexed<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in self.0.as_ref() {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

impl<'a, T: ?Sized + AsRef<[u8]>> fmt::Debug for Hexed<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

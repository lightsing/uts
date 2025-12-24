#[derive(Default, Debug, Clone)]
#[repr(transparent)]
pub struct OnceLock<T>(std::sync::OnceLock<T>);

impl<T> OnceLock<T> {
    pub const fn new() -> OnceLock<T> {
        OnceLock(std::sync::OnceLock::new())
    }

    pub fn get(&self) -> Option<&T> {
        self.0.get()
    }

    pub fn get_or_init<F>(&self, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.0.get_or_init(init)
    }
}

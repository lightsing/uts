#[derive(Default, Debug, Clone)]
#[repr(transparent)]
pub struct OnceLock<T>(once_cell::race::OnceBox<T>);

impl<T> OnceLock<T> {
    pub const fn new() -> OnceLock<T> {
        OnceLock(once_cell::race::OnceBox::new())
    }

    pub fn get(&self) -> Option<&T> {
        self.0.get()
    }

    pub fn get_or_init<F>(&self, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.0.get_or_init(|| alloc::boxed::Box::new(init()))
    }
}

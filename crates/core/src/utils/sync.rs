#[cfg(feature = "std")]
mod std;
#[cfg(feature = "std")]
pub use self::std::OnceLock;

#[cfg(not(feature = "std"))]
mod race;
#[cfg(not(feature = "std"))]
pub use self::race::OnceLock;

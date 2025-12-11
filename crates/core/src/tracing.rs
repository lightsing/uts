#![allow(unused)]

#[cfg(not(feature = "tracing"))]
macro_rules! noop_macro {
    ($($t:tt)*) => {};
}

#[cfg(not(feature = "tracing"))]
pub(crate) use noop_macro as debug;
#[cfg(not(feature = "tracing"))]
pub(crate) use noop_macro as error;
#[cfg(not(feature = "tracing"))]
pub(crate) use noop_macro as info;
#[cfg(not(feature = "tracing"))]
pub(crate) use noop_macro as trace;
#[cfg(not(feature = "tracing"))]
pub(crate) use noop_macro as warn;

#[cfg(feature = "tracing")]
pub use ::tracing::{debug, error, info, trace, warn};

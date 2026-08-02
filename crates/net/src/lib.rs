//! # `async-wayland-net` - Async socket and networking primitives.

pub mod socket;
pub(crate) use socket::*;

mod path;
pub use path::{SocketPath, SocketPathError};

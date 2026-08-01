//! # `async-wayland-net` - Async socket and networking primitives.

mod socket;
pub use socket::WaylandUnixStream;

mod path;
pub use path::{SocketPath, SocketPathError};

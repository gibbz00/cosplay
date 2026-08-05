//! # `async-wayland-net` - Async socket and networking primitives.

pub mod socket;
pub(crate) use socket::*;

/// Type alias for [`UnixStream`] with the the ancillary buffer size preconfigured.
///
/// The Wayland reference implementation sets `MAX_FDS_OUT` to 28, so the same is done here.
pub type WaylandUnixStream = UnixStream<{ rustix::cmsg_space!(ScmRights(28)) }>;

mod path;
pub use path::{SocketPath, SocketPathError};

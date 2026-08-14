//! # `cosplay-net` - Async socket and networking primitives.
//!
//! ### Attribution
//!
//! Many parts in this crate originate from the [anchovy] crate.
//!
//! [anchovy]: https://github.com/tailwags/anchovy

mod socket;
pub(crate) use socket::UnixStreamSocket;

mod stream;
pub use stream::UnixStream;

mod split;
pub use split::{UnixStreamReadHalf, UnixStreamWriteHalf};

mod path;
pub use path::{SocketPath, SocketPathError};

/// The Wayland reference implementation sets `MAX_FDS_OUT` to 28, so the same is done here.
const FD_BUFFER_SIZE: usize = rustix::cmsg_space!(ScmRights(28));

/// Type alias for [`UnixStream`] with the the ancillary buffer size preconfigured.
pub type WaylandUnixStream = UnixStream<FD_BUFFER_SIZE>;

/// Read half returned from [`WaylandUnixStream::into_split`].
pub type WaylandUnixStreamReadHalf = UnixStreamReadHalf<FD_BUFFER_SIZE>;

/// Write half returned from [`WaylandUnixStream::into_split`].
pub type WaylandUnixStreamWriteHalf = UnixStreamWriteHalf<FD_BUFFER_SIZE>;

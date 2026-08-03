//! # `async-wayland-ancillary` - Ancillary data trait primitives complementing `AsyncRead` and `AsyncWrite`.

use std::{collections::VecDeque, os::fd::OwnedFd};

/// Implemented on a readers which buffer any file descriptors passed in control messages.
pub trait AncillaryRead {
    /// Retrieve any passed file descriptors *after* a data read has been performed.
    ///
    /// Call pop_front to remove file descriptors and push_back to append file descriptors.
    fn buffer(&mut self) -> &mut VecDeque<OwnedFd>;
}

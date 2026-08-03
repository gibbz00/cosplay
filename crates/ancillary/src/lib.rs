//! # `async-wayland-ancillary` - Ancillary data trait primitives complementing `AsyncRead` and `AsyncWrite`.

use std::{collections::VecDeque, os::fd::OwnedFd};

/// Implemented on a readers which buffer any file descriptors passed in control messages.
pub trait AncillaryRead {
    /// Retrieve any passed file descriptors *after* a data read has been performed.
    ///
    /// Users call pop_front to get the the file descriptors in the order they arrive.
    /// Likewise, implementors call push_back for each new file descriptor received.
    fn buffer(&mut self) -> &mut VecDeque<OwnedFd>;
}

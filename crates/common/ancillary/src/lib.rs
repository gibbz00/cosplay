//! # `cosplay-ancillary` - Ancillary data trait primitives complementing `AsyncRead` and `AsyncWrite`.

use std::{collections::VecDeque, os::fd::OwnedFd};

/// Implemented on a readers / writers which buffer any file descriptors passed in control messages.
pub trait AncillaryBuffer {
    /// For reading: Retrieve any passed file descriptors *after* a data read has been performed.
    ///
    /// For writing: Put any file descriptors to pass *before* the data write is performed.
    ///
    /// Call pop_front to remove file descriptors and push_back to append file descriptors.
    fn file_descriptors(&mut self) -> &mut VecDeque<OwnedFd>;
}

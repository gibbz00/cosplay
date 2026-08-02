//! # `async-wayland-ancillary` - Ancillary data trait primitives complementing `AsyncRead` and `AsyncWrite`.

/// Implemented on a readers which buffer any file descriptors passed in control messages.
pub trait AncillaryRead {
    /// Retrieve any passed file descriptors *after* a data read has been performed.
    fn drain(&mut self) -> Vec<std::os::fd::OwnedFd>;
}

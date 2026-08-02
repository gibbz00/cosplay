use std::{
    collections::VecDeque,
    io::IoSlice,
    os::fd::OwnedFd,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use crate::*;

/// Read half created from [`WaylandUnixStream::into_split`].
pub struct UnixStreamReadHalf<const S: usize> {
    pub(super) socket: Arc<UnixStreamSocket>,
    pub(super) inbound_fds: VecDeque<OwnedFd>,
}

impl<const S: usize> tokio::io::AsyncRead for UnixStreamReadHalf<S> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut tokio::io::ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        this.socket.poll_read::<S>(cx, buf, &mut this.inbound_fds)
    }
}

/// Write half created from [`WaylandUnixStream::into_split`].
///
/// Dropping the write half will also shut down the write half of the stream.
pub struct UnixStreamWriteHalf<const S: usize> {
    pub(super) shutdown_on_drop: bool,
    pub(super) socket: Arc<UnixStreamSocket>,
    pub(super) outbound_fds: Vec<OwnedFd>,
}

impl<const S: usize> tokio::io::AsyncWrite for UnixStreamWriteHalf<S> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        self.poll_write_vectored(cx, &[IoSlice::new(buf)])
    }

    fn poll_write_vectored(self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &[IoSlice<'_>]) -> Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        this.socket.poll_write_vectored::<S>(cx, bufs, &mut this.outbound_fds)
    }

    fn is_write_vectored(&self) -> bool {
        true
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.socket.poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let mut this = self.as_mut();

        this.socket
            .shutdown_write()
            .inspect(|_| {
                this.shutdown_on_drop = false;
            })
            .into()
    }
}

impl<const S: usize> Drop for UnixStreamWriteHalf<S> {
    fn drop(&mut self) {
        if self.shutdown_on_drop {
            let _ = self.socket.shutdown_write();
        }
    }
}

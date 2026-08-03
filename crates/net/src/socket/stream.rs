use std::{
    collections::VecDeque,
    io::IoSlice,
    os::fd::OwnedFd,
    path::Path,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use tokio::io::unix::AsyncFd;

use crate::*;

/// From the Wayland reference implementation (`MAX_FDS_OUT`).
const FD_LIMIT: usize = 28;

/// A Unix socket stream with support for passing file descriptors via `SCM_RIGHTS` ancillary
/// messages.
///
/// Implements [`AsyncRead`] and [`AsyncWrite`], with vectored write support.
pub type WaylandUnixStream = UnixStream<{ rustix::cmsg_space!(ScmRights(FD_LIMIT)) }>;

pub struct UnixStream<const S: usize> {
    socket: UnixStreamSocket,
    inbound_fds: VecDeque<OwnedFd>,
    outbound_fds: VecDeque<OwnedFd>,
}

impl<const S: usize> UnixStream<S> {
    // FIXME: document: panic if called outside the tokio runtime
    pub fn new(path: &Path) -> std::io::Result<Self> {
        let addr = rustix::net::SocketAddrUnix::new(path)?;

        let fd = rustix::net::socket_with(
            rustix::net::AddressFamily::UNIX,
            rustix::net::SocketType::STREAM,
            rustix::net::SocketFlags::NONBLOCK | rustix::net::SocketFlags::CLOEXEC,
            None,
        )?;

        rustix::net::connect(&fd, &addr)?;

        Self::new_impl(fd)
    }

    pub fn pop_inbound(&mut self) -> Option<OwnedFd> {
        self.inbound_fds.pop_front()
    }

    pub fn push_outbound(&mut self, fd: OwnedFd) {
        self.outbound_fds.push_back(fd);
    }

    pub fn into_split(self) -> (UnixStreamReadHalf<S>, UnixStreamWriteHalf<S>) {
        let Self { socket, inbound_fds, outbound_fds } = self;

        let shared_socket = Arc::new(socket);

        let read = UnixStreamReadHalf { socket: shared_socket.clone(), inbound_fds };

        let write = UnixStreamWriteHalf { shutdown_on_drop: true, socket: shared_socket, outbound_fds };

        (read, write)
    }

    /// Invariants: The file descriptor points to a *connected* unix domain
    /// socket stream in non-blocking mode and close on exec.
    fn new_impl(fd: OwnedFd) -> std::io::Result<Self> {
        AsyncFd::new(fd).map(|inner| Self {
            socket: UnixStreamSocket { inner },
            inbound_fds: Default::default(),
            outbound_fds: Default::default(),
        })
    }
}

impl<const S: usize> tokio::io::AsyncRead for UnixStream<S> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut tokio::io::ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        let this = self.get_mut();
        this.socket.poll_read::<S>(cx, buf, &mut this.inbound_fds)
    }
}

impl<const S: usize> tokio::io::AsyncWrite for UnixStream<S> {
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

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.socket.shutdown_write().into()
    }
}

impl<const S: usize> async_wayland_ancillary::AncillaryRead for UnixStream<S> {
    fn buffer(&mut self) -> &mut VecDeque<OwnedFd> {
        &mut self.inbound_fds
    }
}

#[cfg(test)]
mod tests {
    use std::os::fd::AsRawFd;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    fn mock_pair() -> (WaylandUnixStream, WaylandUnixStream) {
        let (left, right) = rustix::net::socketpair(
            rustix::net::AddressFamily::UNIX,
            rustix::net::SocketType::STREAM,
            rustix::net::SocketFlags::NONBLOCK | rustix::net::SocketFlags::CLOEXEC,
            None,
        )
        .unwrap();

        (
            WaylandUnixStream::new_impl(left).unwrap(),
            WaylandUnixStream::new_impl(right).unwrap(),
        )
    }

    #[tokio::test]
    async fn send_receive_bytes() {
        let mock_str = "hello";

        let (mut writer, mut reader) = mock_pair();

        writer.write_all(mock_str.as_bytes()).await.unwrap();
        writer.shutdown().await.unwrap();

        let mut receive_buffer = String::new();
        reader.read_to_string(&mut receive_buffer).await.unwrap();

        assert_eq!(mock_str, receive_buffer)
    }

    #[tokio::test]
    async fn send_receive_fd() {
        let mock_str = "channel hello";

        let (mut reader, mut writer) = mock_pair();

        tokio::task::spawn(async move {
            reader.read_u8().await.unwrap();
            let received_fd = reader.pop_inbound().unwrap();

            let mut received_writer = WaylandUnixStream::new_impl(received_fd).unwrap();
            received_writer.write_all(mock_str.as_bytes()).await.unwrap();
            received_writer.shutdown().await.unwrap();
        });

        let (mut channel_reader, channel_writer) = mock_pair();

        writer.push_outbound(channel_writer.socket.inner.into_inner());
        writer.write_u8(1).await.unwrap();

        let mut received_string = String::new();
        channel_reader.read_to_string(&mut received_string).await.unwrap();

        assert_eq!(mock_str, received_string);
    }

    #[tokio::test]
    async fn preserve_fd_order() {
        let (first, second) = mock_pair();

        let sent_order = [first.socket.inner.as_raw_fd(), second.socket.inner.as_raw_fd()];

        let (mut reader, mut writer) = mock_pair();

        writer.push_outbound(first.socket.inner.into_inner());
        writer.push_outbound(second.socket.inner.into_inner());
        writer.write_u8(1).await.unwrap();

        reader.read_u8().await.unwrap();
        let received_first = reader.pop_inbound().unwrap();
        let received_second = reader.pop_inbound().unwrap();

        let received_order = [received_first.as_raw_fd(), received_second.as_raw_fd()];

        assert_eq!(sent_order, received_order);
    }
}

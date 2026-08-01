use std::{
    collections::VecDeque,
    io::{IoSlice, IoSliceMut},
    mem::MaybeUninit,
    os::fd::{AsFd, OwnedFd},
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};

use tokio::io::unix::AsyncFd;

/// From the Wayland reference implementation (`MAX_FDS_OUT`).
const FD_LIMIT: usize = 28;

/// A Unix socket stream with support for passing file descriptors via `SCM_RIGHTS` ancillary
/// messages.
///
/// Implements [`AsyncRead`] and [`AsyncWrite`], with vectored write support.
pub type WaylandUnixStream = UnixStreamImpl<{ rustix::cmsg_space!(ScmRights(FD_LIMIT)) }>;

pub struct UnixStreamImpl<const S: usize> {
    socket: AsyncFd<OwnedFd>,
    inbound_fds: VecDeque<OwnedFd>,
    outbound_fds: Vec<OwnedFd>,
}

impl<const S: usize> UnixStreamImpl<S> {
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
        self.outbound_fds.push(fd);
    }

    /// Invariants: The file descriptor points to a *connected* unix domain
    /// socket stream in non-blocking mode and close on exec.
    fn new_impl(fd: OwnedFd) -> std::io::Result<Self> {
        AsyncFd::new(fd).map(|socket| Self {
            socket,
            inbound_fds: Default::default(),
            outbound_fds: Default::default(),
        })
    }
}

impl<const S: usize> tokio::io::AsyncRead for UnixStreamImpl<S> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut tokio::io::ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        let mut cmsg_space = [MaybeUninit::uninit(); S];
        let mut ancillary = rustix::net::RecvAncillaryBuffer::new(&mut cmsg_space);

        loop {
            let mut guard = std::task::ready!(self.socket.poll_read_ready(cx))?;

            let unfilled = buf.initialize_unfilled();

            let recv_result = guard.try_io(|inner| {
                rustix::net::recvmsg(
                    inner,
                    &mut [IoSliceMut::new(unfilled)],
                    &mut ancillary,
                    rustix::net::RecvFlags::CMSG_CLOEXEC,
                )
                .map_err(rustix_to_io_err)
            });

            match recv_result {
                Err(_would_block) => continue,
                Ok(result) => {
                    let result = result.map(|msg| {
                        buf.advance(msg.bytes);

                        for message in ancillary.drain() {
                            if let rustix::net::RecvAncillaryMessage::ScmRights(fds) = message {
                                for fd in fds {
                                    self.inbound_fds.push_back(fd);
                                }
                            }
                        }
                    });

                    return Poll::Ready(result);
                }
            }
        }
    }
}

impl<const S: usize> tokio::io::AsyncWrite for UnixStreamImpl<S> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        self.poll_write_vectored(cx, &[IoSlice::new(buf)])
    }

    fn poll_write_vectored(mut self: Pin<&mut Self>, cx: &mut Context<'_>, bufs: &[IoSlice<'_>]) -> Poll<std::io::Result<usize>> {
        let outbound_fds = self.outbound_fds.iter().map(OwnedFd::as_fd).collect::<Vec<_>>();

        loop {
            let mut guard = std::task::ready!(self.socket.poll_write_ready(cx))?;

            let mut cmsg_space = [MaybeUninit::uninit(); S];
            let mut ancillary = rustix::net::SendAncillaryBuffer::new(&mut cmsg_space);

            if !outbound_fds.is_empty() {
                ancillary.push(rustix::net::SendAncillaryMessage::ScmRights(&outbound_fds));
            }

            let send_result = guard.try_io(|inner| {
                rustix::net::sendmsg(inner, bufs, &mut ancillary, rustix::net::SendFlags::NOSIGNAL).map_err(rustix_to_io_err)
            });

            match send_result {
                Err(_would_block) => continue,
                Ok(result) => {
                    if result.is_ok() {
                        self.outbound_fds.clear();
                    }

                    return Poll::Ready(result);
                }
            }
        }
    }

    fn is_write_vectored(&self) -> bool {
        true
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        rustix::net::shutdown(self.get_mut().socket.as_fd(), rustix::net::Shutdown::Write)?;

        Poll::Ready(Ok(()))
    }
}

fn rustix_to_io_err(rustix_err: rustix::io::Errno) -> std::io::Error {
    std::io::Error::from_raw_os_error(rustix_err.raw_os_error())
}

#[cfg(test)]
mod tests {
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

        writer.push_outbound(channel_writer.socket.into_inner());
        writer.write_u8(1).await.unwrap();

        let mut received_string = String::new();
        channel_reader.read_to_string(&mut received_string).await.unwrap();

        assert_eq!(mock_str, received_string);
    }
}

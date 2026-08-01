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
    outbound_fds: VecDeque<OwnedFd>,
}

impl<const S: usize> UnixStreamImpl<S> {
    // TODO: document: panic if called outside the tokio runtime
    pub fn new(path: &Path) -> std::io::Result<Self> {
        let addr = rustix::net::SocketAddrUnix::new(path)?;

        let fd = rustix::net::socket_with(
            rustix::net::AddressFamily::UNIX,
            rustix::net::SocketType::STREAM,
            rustix::net::SocketFlags::NONBLOCK | rustix::net::SocketFlags::CLOEXEC,
            None,
        )?;

        rustix::net::connect(&fd, &addr)?;

        let socket = AsyncFd::new(fd)?;

        Ok(Self { socket, inbound_fds: VecDeque::new(), outbound_fds: VecDeque::new() })
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

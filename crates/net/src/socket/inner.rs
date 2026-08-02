use std::{
    collections::VecDeque,
    io::{IoSlice, IoSliceMut},
    mem::MaybeUninit,
    os::fd::{AsFd, OwnedFd},
    task::{Context, Poll},
};

use tokio::io::unix::AsyncFd;

pub(crate) struct UnixStreamSocket {
    pub(crate) inner: AsyncFd<OwnedFd>,
}

// AsyncRead / AsyncWrite poll functions that take self by reference and the
// fd_buffer as a separate parameter. `UnixStreamSocket` can then be wrapped in
// a Arc for split read and write halves.

impl UnixStreamSocket {
    pub(crate) fn poll_read<const S: usize>(
        &self,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
        fd_buffer: &mut VecDeque<OwnedFd>,
    ) -> Poll<std::io::Result<()>> {
        let mut cmsg_space = [MaybeUninit::uninit(); S];
        let mut ancillary = rustix::net::RecvAncillaryBuffer::new(&mut cmsg_space);

        loop {
            let mut guard = std::task::ready!(self.inner.poll_read_ready(cx))?;

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
                                    fd_buffer.push_back(fd);
                                }
                            }
                        }
                    });

                    return Poll::Ready(result);
                }
            }
        }
    }

    pub(crate) fn poll_write_vectored<const S: usize>(
        &self,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
        fd_buffer: &mut Vec<OwnedFd>,
    ) -> Poll<std::io::Result<usize>> {
        let outbound_fds = fd_buffer.iter().map(OwnedFd::as_fd).collect::<Vec<_>>();

        loop {
            let mut guard = std::task::ready!(self.inner.poll_write_ready(cx))?;

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
                        fd_buffer.clear();
                    }

                    return Poll::Ready(result);
                }
            }
        }
    }

    pub(crate) fn poll_flush(&self, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    pub(crate) fn shutdown_write(&self) -> std::io::Result<()> {
        rustix::net::shutdown(self.inner.as_fd(), rustix::net::Shutdown::Write).map_err(Into::into)
    }
}

fn rustix_to_io_err(rustix_err: rustix::io::Errno) -> std::io::Error {
    std::io::Error::from_raw_os_error(rustix_err.raw_os_error())
}

use std::{ffi::c_void, num::NonZeroUsize, os::fd::OwnedFd, ptr::NonNull};

use rustix::mm::{MapFlags, ProtFlags};

/// Memory mapped pointer to an in-memory file that has
/// been sent to the server with `wl_shm::create_pool`.
///
/// Wrapped to avoid accidental copies etc.
#[derive(Debug)]
pub(crate) struct ShmPtr {
    inner: NonNull<c_void>,
    len: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateShmPtrError {
    #[error("Failed to invoke `memfd_create`: {0}")]
    FdCreate(std::io::Error),
    #[error("Unable resize in-memory file with `ftruncate`: {0}")]
    Resize(std::io::Error),
    #[error("Failed to invoke `mmap` on in-memory file: {0}")]
    Mmap(std::io::Error),
}

impl ShmPtr {
    pub(crate) fn new(len: NonZeroUsize) -> Result<(Self, OwnedFd), CreateShmPtrError> {
        // `mmap()` will fail if len is zero.
        let len = len.get();

        // Fine to reuse name as file descriptor number is prefixed to final path.
        const MEMFD_NAME: &str = "cosplay-memfd";

        // Create an in-memory file.
        let fd = rustix::fs::memfd_create(MEMFD_NAME, rustix::fs::MemfdFlags::CLOEXEC)
            .map_err(into_io_error)
            .map_err(CreateShmPtrError::FdCreate)?;

        // Size it to the appropriate buffer size.
        rustix::fs::ftruncate(&fd, len as u64)
            .map_err(into_io_error)
            .map_err(CreateShmPtrError::Resize)?;

        // SAFETY: Passed pointer is null and therefore guaranteed to be aligned.
        let inner = unsafe {
            rustix::mm::mmap(
                std::ptr::null_mut(),
                len,
                ProtFlags::WRITE | ProtFlags::READ,
                MapFlags::SHARED,
                &fd,
                0,
            )
        }
        .map(|ptr| NonNull::new(ptr).expect("`rustix::mm::mmap` returned nullptr even when the function succeeded."))
        .map_err(into_io_error)
        .map_err(CreateShmPtrError::Mmap)?;

        let this = Self { inner, len };

        return Ok((this, fd));

        fn into_io_error(errno: rustix::io::Errno) -> std::io::Error {
            std::io::Error::from_raw_os_error(errno.raw_os_error())
        }
    }

    pub(super) fn len(&self) -> usize {
        self.len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_multiple() {
        let len = NonZeroUsize::new(10).unwrap();

        // Mostly to assert that file names don't clash.
        let _pair_0 = ShmPtr::new(len).unwrap();
        let _pair_1 = ShmPtr::new(len).unwrap();
    }
}

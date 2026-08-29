use std::{ffi::c_void, num::NonZeroUsize, os::fd::OwnedFd};

use rustix::mm::{MapFlags, ProtFlags};

/// Shared memory mapped region to an in-memory file that has
/// been sent to the server with `wl_shm::create_pool`.
#[derive(Debug)]
pub struct ShmRegion {
    ptr: *mut c_void,
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

impl ShmRegion {
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
        let ptr = unsafe {
            rustix::mm::mmap(
                std::ptr::null_mut(),
                len,
                ProtFlags::WRITE | ProtFlags::READ,
                MapFlags::SHARED,
                &fd,
                0,
            )
        }
        .map_err(into_io_error)
        .map_err(CreateShmPtrError::Mmap)?;

        assert!(
            !ptr.is_null(),
            "`rustix::mm::mmap` returned nullptr even when the function succeeded."
        );

        let this = Self { ptr, len };

        return Ok((this, fd));

        fn into_io_error(errno: rustix::io::Errno) -> std::io::Error {
            std::io::Error::from_raw_os_error(errno.raw_os_error())
        }
    }

    /// # Panics
    ///
    /// Panics if `src.len()` > `self.len()`.
    ///
    /// # Safety
    ///
    /// Caller must unsure that the server will not read or write from the memory region.
    pub unsafe fn write(&mut self, src: &[u8]) {
        let src_ptr = src.as_ptr();

        assert!(src.len() <= self.len());

        // SAFETY:
        // - `src_ptr` is valid for reads of size `src.len()`.
        // - `self.ptr` is assumed to be valid for writes, namely:
        //   - Aligned as returned by mmap.
        //   - Not null as asserted in Self::new().
        //   - Contained within one continuous allocation.
        //   - `self.len() <= src.len()`
        //   - Caller ensures that the backing memory region is not being accessed concurrently.
        unsafe { std::ptr::copy_nonoverlapping(src_ptr, self.ptr.cast(), src.len()) };
    }

    // Never empty since new requires len of NonZeroUsize.
    #[expect(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.len
    }
}

impl Drop for ShmRegion {
    fn drop(&mut self) {
        // SAFETY: `self.ptr` is aligned, exclusively owned, and has a page size of `self.len()`.
        let munmap_result = unsafe { rustix::mm::munmap(self.ptr, self.len) };

        if let Err(error_number) = munmap_result {
            tracing::error!(kind = %error_number.kind(), "`rustix::mm::munmap returned an unhandled error.`")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use rustix::mm::MsyncFlags;

    use super::*;

    #[test]
    fn create_multiple() {
        let len = NonZeroUsize::new(10).unwrap();

        // Mostly to assert that file names don't clash.
        let _pair_0 = ShmRegion::new(len).unwrap();
        let _pair_1 = ShmRegion::new(len).unwrap();
    }

    #[test]
    fn write_read() {
        let msg = b"hello shared memory!";

        let len = NonZeroUsize::new(msg.len()).unwrap();

        let (mut region, fd) = ShmRegion::new(len).unwrap();

        unsafe { region.write(msg) }

        let mut recv_buffer = Vec::with_capacity(msg.len());
        std::fs::File::from(fd).read_to_end(&mut recv_buffer).unwrap();

        assert_eq!(msg, recv_buffer.as_slice());
    }

    #[test]
    #[should_panic]
    fn write_overflow() {
        let len = NonZeroUsize::new(1).unwrap();

        let (mut region, _fd) = ShmRegion::new(len).unwrap();

        unsafe { region.write(&[0, 0]) }
    }

    #[test]
    fn unmap_on_drop() {
        let len = NonZeroUsize::new(1).unwrap();

        let (region, _fd) = ShmRegion::new(len).unwrap();

        let ptr = region.ptr;
        let len = region.len;

        // SAFETY: ptr and length are valid.
        let msync = || unsafe { rustix::mm::msync(ptr, len, MsyncFlags::empty()) };

        assert!(msync().is_ok());

        drop(region);

        assert!(msync().is_err());
    }
}

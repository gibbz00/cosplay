use std::collections::HashSet;

use cosplay_codec::{ArgumentDecodeError, Enumeration};
use cosplay_core_client::*;
use cosplay_protocols_wayland::wl_shm::{self, PixelFormat, WlShm};
use rustix::mm::{MapFlags, ProtFlags};

use crate::*;

/// Handle to a `wl_shm` instance.
///
/// Drop implementation automatically queue a [`wl_shm::Release`] request.
pub struct WlShmHandle {
    handle: ScopedObjectHandle<WlShm>,
    supported_formats: HashSet<PixelFormat>,
}

impl GlobalHandle for WlShmHandle {
    type Interface = WlShm;

    fn from_raw(handle: ObjectHandle<Self::Interface>) -> Self {
        Self { handle: handle.into(), supported_formats: Default::default() }
    }
}

/// Returned from [`WlShmHandle::sync_supported_formats`].
#[derive(Debug, thiserror::Error)]
pub enum SyncSupportedFormatsError {
    #[error("Failed to deserialize wl_shm::Format: {0}")]
    Deserialize(#[from] ArgumentDecodeError),
    #[error("Unknown event of opcode '{0}' forwarded to wl_shm.")]
    UnknownEvent(u16),
    #[error("Unhandled error event forwarded to wl_shm. {code:?}. {message}")]
    UnhandledError { code: wl_shm::Error, message: String },
    #[error("Failed to retrieve object events: {0}")]
    Events(#[from] ObjectEventsError),
}

impl WlShmHandle {
    /// Shorthand for calling [`Self::sync_supported_formats`] and then
    /// [`Self::get_supported_formats`].
    pub fn supported_formats(&mut self) -> Result<&HashSet<PixelFormat>, SyncSupportedFormatsError> {
        self.sync_supported_formats()?;
        Ok(self.get_supported_formats())
    }

    /// Get the internally buffered set of supported formats as announced by the server.
    ///
    /// Note that the internal buffer may be out of sync with queued inbound events. Most
    /// users will want to first call [`Self::sync_supported_formats`].
    pub fn get_supported_formats(&self) -> &HashSet<PixelFormat> {
        &self.supported_formats
    }

    /// Checks if any new supported pixel formats have been announced by the
    /// server and updates the internal set accordingly. The set can then be
    /// inspected with [`Self::supported_formats`].
    pub fn sync_supported_formats(&mut self) -> Result<(), SyncSupportedFormatsError> {
        for inbound_result in self.handle.event().iter() {
            match inbound_result? {
                ObjectEvent::Event(event) => match event {
                    wl_shm::WlShmEvent::Format(format) => {
                        let format = format.format.inner();
                        self.supported_formats.insert(format);
                    }
                },
                ObjectEvent::Error { code, message } => {
                    let code = wl_shm::Error::from_repr(code);
                    return Err(SyncSupportedFormatsError::UnhandledError { code, message });
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WlShmPoolError {
    #[error("Requested size does not fit into protocol message argument.")]
    SizeOverflow,
    #[error("Failed to invoke `memfd_create`: {0}")]
    FdCreate(std::io::Error),
    #[error("Unable resize in-memory file with `ftruncate`: {0}")]
    Resize(std::io::Error),
    #[error("Failed to invoke `mmap` on in-memory file: {0}")]
    Mmap(std::io::Error),
    #[error("Failed to request `wl_shm::create_pool`.")]
    Request(#[from] RequestError),
}

impl WlShmHandle {
    pub fn create_pool(&self, size: u32) -> Result<WlShmPoolHandle<Available>, WlShmPoolError> {
        // i32 used used in the wire protocol but doesn't make
        // sense to be negative from a user's standpoint.
        //
        // Intentionally done before any syscalls are made.
        let arg_size = i32::try_from(size).map_err(|_| WlShmPoolError::SizeOverflow)?;

        // Fine to reuse name as file descriptor number is prefixed to final path.
        const MEMFD_NAME: &str = "cosplay-memfd";

        // Create an in-memory file.
        let fd = rustix::fs::memfd_create(MEMFD_NAME, rustix::fs::MemfdFlags::CLOEXEC)
            .map_err(into_io_error)
            .map_err(WlShmPoolError::FdCreate)?;

        // Size it to the appropriate buffer size.
        rustix::fs::ftruncate(&fd, size as u64)
            .map_err(into_io_error)
            .map_err(WlShmPoolError::FdCreate)?;

        // SAFETY: Passed pointer is null and therefore guaranteed to be aligned.
        let shm_ptr = unsafe {
            rustix::mm::mmap(
                std::ptr::null_mut(),
                size as usize,
                ProtFlags::WRITE | ProtFlags::READ,
                MapFlags::SHARED,
                &fd,
                0,
            )
        }
        .map_err(into_io_error)
        .map_err(WlShmPoolError::Mmap)?;

        let raw_handle = self
            .handle
            .request()
            .init_subobject(|id| wl_shm::CreatePool { id, fd, size: arg_size })?;

        let shm_pool = WlShmPoolHandle::new(shm_ptr, arg_size, raw_handle);

        Ok(shm_pool)
    }
}

fn into_io_error(errno: rustix::io::Errno) -> std::io::Error {
    std::io::Error::from_raw_os_error(errno.raw_os_error())
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn sync_supported_formats() {
        let (test_driver, object_handle) = TestDriver::new();

        let mut shm_handle = WlShmHandle::from_raw(object_handle);

        assert!(shm_handle.supported_formats.is_empty());

        shm_handle.sync_supported_formats().unwrap();

        assert!(shm_handle.supported_formats.is_empty());

        test_driver.send_event(shm_handle.handle.id(), wl_shm::Format { format: PixelFormat::C8.into() });
        test_driver.send_event(shm_handle.handle.id(), wl_shm::Format { format: PixelFormat::Xrgb4444.into() });

        shm_handle.sync_supported_formats().unwrap();

        assert!(shm_handle.supported_formats.contains(&PixelFormat::C8));
        assert!(shm_handle.supported_formats.contains(&PixelFormat::Xrgb4444));
    }

    #[test]
    fn drop_sends_release() {
        let (mut test_driver, object_handle) = TestDriver::new();

        let shm_handle = WlShmHandle::from_raw(object_handle);

        test_driver.assert_queued_destructor_on_drop::<wl_shm::Release, _>(shm_handle.handle.id(), shm_handle);
    }

    #[test]
    fn arg_size_overflow() {
        let (_, object_handle) = TestDriver::new();

        let shm_handle = WlShmHandle::from_raw(object_handle);

        assert_matches!(shm_handle.create_pool(u32::MAX), Err(WlShmPoolError::SizeOverflow));
    }

    #[test]
    fn multiple_pools() {
        let (_driver, object_handle) = TestDriver::new();

        let shm_handle = WlShmHandle::from_raw(object_handle);

        let _pool_result_0 = shm_handle.create_pool(100).unwrap();

        let _pool_result_1 = shm_handle.create_pool(100).unwrap();
    }
}

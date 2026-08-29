use std::{collections::HashSet, num::NonZeroUsize};

use cosplay_codec::{ArgumentDecodeError, Enumeration};
use cosplay_core_client::*;
use cosplay_protocols_wayland::{
    wl_shm::{self, PixelFormat, WlShm},
    wl_shm_pool,
};

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
pub enum CreateCombinedBuffer {
    #[error("Bytes per pixel for format not known.")]
    UnknownPixelDensity,
    #[error("Requested size does not fit into protocol message argument.")]
    SizeOverflow,
    #[error("Requested size can not be zero.")]
    ZeroSized,
    #[error("Failed to create shared memory region: {0}")]
    CreateShmPtr(#[from] CreateShmPtrError),
    #[error("Failed to request `wl_shm::create_pool`.")]
    CreatePool(RequestError),
    #[error("Failed to request `wl_shm_pool::create_buffer`: {0}")]
    CreateBuffer(RequestError),
}

impl WlShmHandle {
    /// Create a buffer without creating a standalone pool handle. (The wl_shm_pool is held within
    /// wl_buffer.)
    ///
    /// This should cover the majority of simpler pooling uses-cases without becoming full-blown
    /// memory allocators. Thus avoiding fragmentation handling whilst still supporting mmap
    /// reuse; just reuse the buffer.
    ///
    /// Additional benefits from both the implementation and user perspective include:
    ///
    /// - Size changes need not be coordinated between the pool and the buffer.
    ///
    /// - No error handling for trying to take a buffer from a pool with too few bytes remaining.
    ///
    /// - No overhead of managing how buffers are returned to the parent pool.
    ///
    /// - Pool size is often a function of expected pixel format, width, and height. Passing them
    ///   upfront removes the possibility of sizing error from being returned in
    ///   `wl_shm_pool::create_buffer` .
    ///
    /// - Pixel format support is communicated to `wl_shm`, but passed as as argument to
    ///   `wl_shm_pool::create_buffer`. Combining both removes the need sync the supported formats
    ///   between the handles.
    pub fn create_combined_buffer(
        &self,
        width: u16,
        height: u16,
        format: PixelFormat,
    ) -> Result<WlCombinedBufferHandle, CreateCombinedBuffer> {
        // i32 used used in the wire protocol but doesn't make
        // sense to be negative from a user's standpoint.
        let height = height as i32;
        let width = width as i32;
        let bytes_per_pixel = cosplay_agent::pixel_format::bytes_per_pixel(format).ok_or(CreateCombinedBuffer::UnknownPixelDensity)?;
        // u16 * u8 can't overflow a i32.
        let stride = width * (bytes_per_pixel as i32);
        let size = stride.checked_mul(height).ok_or(CreateCombinedBuffer::SizeOverflow)?;

        let len = NonZeroUsize::new(size as usize).ok_or(CreateCombinedBuffer::ZeroSized)?;

        let (shm_ptr, fd) = ShmRegion::new(len)?;

        let raw_pool_handle = self
            .handle
            .request()
            .init_subobject(|id| wl_shm::CreatePool { id, fd, size })
            .map_err(CreateCombinedBuffer::CreatePool)?;

        let raw_buffer_handle = raw_pool_handle
            .request()
            .init_subobject(|id| wl_shm_pool::CreateBuffer { id, offset: 0, width, height, stride, format: format.into() })
            .map_err(CreateCombinedBuffer::CreateBuffer)?;

        Ok(WlCombinedBufferHandle::new(raw_pool_handle, raw_buffer_handle, shm_ptr))
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn sync_supported_formats() {
        let (driver, object_handle) = TestDriver::new();

        let mut shm_handle = WlShmHandle::from_raw(object_handle);

        assert!(shm_handle.supported_formats.is_empty());

        shm_handle.sync_supported_formats().unwrap();

        assert!(shm_handle.supported_formats.is_empty());

        driver.send_event(shm_handle.handle.id(), wl_shm::Format { format: PixelFormat::C8.into() });
        driver.send_event(shm_handle.handle.id(), wl_shm::Format { format: PixelFormat::Xrgb4444.into() });

        shm_handle.sync_supported_formats().unwrap();

        assert!(shm_handle.supported_formats.contains(&PixelFormat::C8));
        assert!(shm_handle.supported_formats.contains(&PixelFormat::Xrgb4444));
    }

    #[test]
    fn send_release_on_drop() {
        let (mut driver, object_handle) = TestDriver::new();

        let shm_handle = WlShmHandle::from_raw(object_handle);

        driver.assert_queued_destructor_on_drop::<wl_shm::Release, _>(shm_handle.handle.id(), shm_handle);
    }

    #[test]
    fn create_combined_buffer_unknown_density_err() {
        let (_driver, object_handle) = TestDriver::new();
        let shm_handle = WlShmHandle::from_raw(object_handle);

        let error = shm_handle.create_combined_buffer(0, 0, PixelFormat::Other(0)).unwrap_err();
        assert_matches!(error, CreateCombinedBuffer::UnknownPixelDensity);
    }

    #[test]
    fn create_combined_buffer_size_overflow() {
        let (_driver, object_handle) = TestDriver::new();
        let shm_handle = WlShmHandle::from_raw(object_handle);

        assert_matches!(
            shm_handle.create_combined_buffer(u16::MAX, u16::MAX, PixelFormat::Y8),
            Err(CreateCombinedBuffer::SizeOverflow)
        );
    }

    #[test]
    fn create_combined_buffer_with_stride() {
        let (_driver, object_handle) = TestDriver::new();
        let shm_handle = WlShmHandle::from_raw(object_handle);

        let buffer = shm_handle.create_combined_buffer(2, 2, PixelFormat::Argb8888).unwrap();
        assert_eq!(2 * 2 * 4, buffer.region().len());

        let buffer = shm_handle.create_combined_buffer(2, 2, PixelFormat::Y8).unwrap();
        assert_eq!(2 * 2, buffer.region().len());
    }
}

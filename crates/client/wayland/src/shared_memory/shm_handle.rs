use std::{collections::HashSet, num::NonZeroUsize};

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

impl WlShmHandle {
    /// Get the internally buffered set of supported formats as announced by the server.
    ///
    /// Note that the internal buffer may be out of sync with queued inbound events. Most
    /// users will want to first call [`Self::sync_supported_formats`].
    pub fn get_supported_formats(&self) -> &HashSet<PixelFormat> {
        &self.supported_formats
    }

    /// Checks if any new supported pixel formats have been announced by the
    /// server and updates the internal set accordingly. The set can then be
    /// inspected with [`Self::get_supported_formats`].
    pub fn sync_supported_formats(&mut self) -> Result<(), ObjectEventsError> {
        for inbound_result in self.handle.event().iter() {
            match inbound_result? {
                wl_shm::WlShmEvent::Format(format) => {
                    let format = format.format.inner();
                    self.supported_formats.insert(format);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateBufferError {
    #[error("Format not processed from a `wl_shm::format` event.")]
    UnsupportedPixelFormat,
    #[error("Bytes per pixel for format not known.")]
    UnknownPixelDensity,
    #[error("Requested size does not fit into protocol message argument.")]
    SizeOverflow,
    #[error("Requested size can not be zero.")]
    ZeroSized,
    #[error("Failed to create shared memory region: {0}")]
    CreateShmPtr(#[from] CreateShmRegionError),
    #[error("Failed to request `wl_shm::create_pool`.")]
    CreatePool(RequestError),
    #[error("Failed to request `wl_shm_pool::create_buffer`: {0}")]
    CreateBuffer(RequestError),
}

impl WlShmHandle {
    /// Create a buffer without creating a standalone pool handle. (The `wl_shm_pool` is held within
    /// the wrapped `wl_buffer`)
    ///
    /// This should cover the majority of simpler pooling uses-cases without turning into the pool a
    /// full-blown memory allocator. Thus avoiding fragmentation handling whilst still
    /// supporting mmap reuse; "just" reuse the buffer.
    ///
    /// Additional benefits from both the implementation and user perspective include:
    ///
    /// - Size changes need not be coordinated between pool and buffer handles.
    ///
    /// - No error handling for trying to take a buffer from a pool with too few bytes remaining.
    ///
    /// - No overhead of managing how buffers are returned to the parent pool.
    ///
    /// - Pool size is often a function of expected pixel format, width, and height. Passing them
    ///   upfront removes the possibility of sizing errors from being returned in a
    ///   `wl_shm_pool::create_buffer` .
    ///
    /// - Pixel format support is held in `wl_shm`, but passed as as argument to
    ///   `wl_shm_pool::create_buffer`. Avoiding the intermediary handle removes the need sync
    ///   supported formats between handles.
    ///
    /// # Pixel Format Support
    ///
    /// First thing `create_shm_buffer()` does is to check if the requested pixel format exists
    /// in the internal set of supported formats. This set is in turn only populated by calling
    /// [`Self::sync_supported_formats`]. As such, the user is expected to have synced the supported
    /// formats at startup, or risk receiving [`CreateBufferError::UnsupportedPixelFormat`]
    /// indefinitely.
    ///
    /// ```
    /// use cosplay_core_client::{RegistryHandle, SyncHandle};
    /// use cosplay_wayland_client::shared_memory::WlShmHandle;
    /// use cosplay_protocols_wayland::wl_shm::PixelFormat;
    ///
    /// async fn run(
    ///     registry_handle: &mut RegistryHandle,
    ///     sync_handle: &SyncHandle,
    /// ) -> Result<(), Box<dyn std::error::Error>> {
    ///     let mut shm_handle = registry_handle.bind::<WlShmHandle>()?;
    ///
    ///     // Sync roundtrip to ensure that the server has finished its announcement of
    ///     // all supported pixel formats over `wl_shm::format` events.
    ///     sync_handle.sync().await?;
    ///
    ///     shm_handle.sync_supported_formats()?;
    ///
    ///     // It can now be assumed that `shm_handle` knows about all formats currently supported by the compositor.
    ///     let _buffer = shm_handle.create_shm_buffer(128, 128, PixelFormat::Argb8888)?;
    ///
    ///     Ok(())
    /// }
    /// ````
    pub fn create_shm_buffer(&self, width: u16, height: u16, format: PixelFormat) -> Result<ShmBuffer<Available>, CreateBufferError> {
        if !self.supported_formats.contains(&format) {
            return Err(CreateBufferError::UnsupportedPixelFormat);
        }

        // i32 used used in the wire protocol but doesn't make
        // sense to be negative from a user's standpoint.
        let height = height as i32;
        let width = width as i32;
        let bytes_per_pixel = cosplay_agent::pixel_format::bytes_per_pixel(format).ok_or(CreateBufferError::UnknownPixelDensity)?;
        // u16 * u8 can't overflow a i32.
        let stride = width * (bytes_per_pixel as i32);
        let size = stride.checked_mul(height).ok_or(CreateBufferError::SizeOverflow)?;

        let len = NonZeroUsize::new(size as usize).ok_or(CreateBufferError::ZeroSized)?;

        let (shm_ptr, fd) = ShmRegion::new(len)?;

        let raw_pool_handle = self
            .handle
            .request()
            .init_subobject(|id| wl_shm::CreatePool { id, fd, size })
            .map_err(CreateBufferError::CreatePool)?;

        let raw_buffer_handle = raw_pool_handle
            .request()
            .init_subobject(|id| wl_shm_pool::CreateBuffer { id, offset: 0, width, height, stride, format: format.into() })
            .map_err(CreateBufferError::CreateBuffer)?;

        Ok(ShmBuffer::new(raw_pool_handle, raw_buffer_handle, shm_ptr))
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn sync_supported_formats() {
        let (driver, mut shm_handle) = TestDriver::new_global::<WlShmHandle>();

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
        let (mut driver, shm_handle) = TestDriver::new_global::<WlShmHandle>();

        driver.assert_queued_destructor_on_drop::<wl_shm::Release, _>(shm_handle.handle.id(), shm_handle);
    }

    #[test]
    fn create_buffer_unknown_format_err() {
        let (_driver, shm_handle) = TestDriver::new_global::<WlShmHandle>();

        let error = shm_handle.create_shm_buffer(0, 0, PixelFormat::Argb8888).unwrap_err();
        assert_matches!(error, CreateBufferError::UnsupportedPixelFormat);
    }

    #[test]
    fn create_buffer_unknown_density_err() {
        let (_driver, mut shm_handle) = TestDriver::new_global::<WlShmHandle>();
        shm_handle.supported_formats.insert(PixelFormat::Other(0));

        let error = shm_handle.create_shm_buffer(0, 0, PixelFormat::Other(0)).unwrap_err();
        assert_matches!(error, CreateBufferError::UnknownPixelDensity);
    }

    #[test]
    fn create_buffer_size_overflow() {
        let (_driver, mut shm_handle) = TestDriver::new_global::<WlShmHandle>();
        shm_handle.supported_formats.insert(PixelFormat::Y8);

        assert_matches!(
            shm_handle.create_shm_buffer(u16::MAX, u16::MAX, PixelFormat::Y8),
            Err(CreateBufferError::SizeOverflow)
        );
    }

    #[test]
    fn create_buffer_with_stride() {
        let (_driver, mut shm_handle) = TestDriver::new_global::<WlShmHandle>();
        shm_handle.supported_formats.insert(PixelFormat::Argb8888);
        shm_handle.supported_formats.insert(PixelFormat::Y8);

        let buffer = shm_handle.create_shm_buffer(2, 2, PixelFormat::Argb8888).unwrap();
        assert_eq!(2 * 2 * 4, buffer.len());

        let buffer = shm_handle.create_shm_buffer(2, 2, PixelFormat::Y8).unwrap();
        assert_eq!(2 * 2, buffer.len());
    }
}

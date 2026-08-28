use std::{ffi::c_void, marker::PhantomData};

use cosplay_core_client::{ObjectHandle, RequestError};
use cosplay_protocols_wayland::{
    wl_shm::PixelFormat,
    wl_shm_pool::{self, WlShmPool},
};

use crate::*;

pub struct Available {
    _priv: (),
}
pub struct Reserved {
    _priv: (),
}

/// Simple wrapper for a `wl_shm_pool` which covers the simpler and more common
/// use-cases without becoming a full-blown memory allocator.
///
/// It does so by only allowing one buffer to be reserved and then returned
/// at time. Thus avoiding fragmentation handling whilst still supporting mmap
/// reuse.
#[impl_tools::autoimpl(Debug)]
pub struct WlShmPoolHandle<S> {
    state_marker: PhantomData<S>,
    handle: ObjectHandle<WlShmPool>,
    /// Memory mapped pointer to an in-memory file that has
    // been sent to the server.
    shm_ptr: *mut c_void,
    /// Original length of memory mapped pointer.
    size: i32,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateBufferError {
    #[error("Provided region overflows the max pointer size.")]
    PointerOverflow,
    #[error("Bytes per pixel for format not known.")]
    UnknownPixelDensity,
    #[error("Not enough bytes left in pool.")]
    InsufficientBytes,
    #[error("Failed to request `wl_shm_pool::create_buffer`: {0}")]
    Request(#[from] RequestError),
}

impl WlShmPoolHandle<Available> {
    pub(super) fn new(shm_ptr: *mut c_void, size: i32, handle: ObjectHandle<WlShmPool>) -> Self {
        Self { handle, shm_ptr, size, state_marker: PhantomData }
    }

    // FIXME: pixel format must have been advertised by shm
    pub fn create_buffer(
        self,
        width: u16,
        height: u16,
        format: PixelFormat,
    ) -> Result<(WlShmPoolHandle<Reserved>, WlBufferHandle), (WlShmPoolHandle<Available>, CreateBufferError)> {
        // i32 used used in the wire protocol but doesn't make
        // sense to be negative from a user's standpoint.
        let height = height as i32;
        let width = width as i32;

        let stride = match Self::checked_stride(self.size, height, width, format) {
            Ok(stride) => stride,
            Err(err) => return Err((self, err)),
        };

        let wl_buffer_handle_result = self.handle.request().init_subobject(|id| wl_shm_pool::CreateBuffer {
            id,
            offset: 0,
            width,
            height,
            stride,
            format: format.into(),
        });

        match wl_buffer_handle_result {
            Ok(buffer_handle) => {
                let Self { handle, shm_ptr, size, .. } = self;

                let reserved = WlShmPoolHandle { handle, shm_ptr, size, state_marker: PhantomData };

                let buffer = WlBufferHandle::new(buffer_handle);

                Ok((reserved, buffer))
            }
            Err(err) => Err((self, err.into())),
        }
    }

    fn checked_stride(buffer_size: i32, height: i32, width: i32, format: PixelFormat) -> Result<i32, CreateBufferError> {
        let bytes_per_pixel = cosplay_agent::pixel_format::bytes_per_pixel(format).ok_or(CreateBufferError::UnknownPixelDensity)?;

        // u16 * u8 can't overflow a i32.
        let stride = width * (bytes_per_pixel as i32);

        let new_size = stride.checked_mul(height).ok_or(CreateBufferError::PointerOverflow)?;

        if new_size > buffer_size {
            return Err(CreateBufferError::InsufficientBytes);
        }

        Ok(stride)
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use cosplay_core_client::TestDriver;

    use super::*;

    #[test]
    fn non_zero_size_ok() {
        let (_driver, object_handle) = TestDriver::new();
        let pool_handle = WlShmPoolHandle::new(std::ptr::null_mut(), 256, object_handle);

        let result = pool_handle.create_buffer(8, 8, PixelFormat::Argb8888);
        assert!(result.is_ok());
    }

    #[test]
    fn zero_size_ok() {
        let (_driver, object_handle) = TestDriver::new();
        let pool_handle = WlShmPoolHandle::new(std::ptr::null_mut(), 0, object_handle);

        let result = pool_handle.create_buffer(0, 0, PixelFormat::Argb8888);
        assert!(result.is_ok());
    }

    #[test]
    fn size_uses_stride() {
        let (_driver, object_handle) = TestDriver::new();
        let pool_handle = WlShmPoolHandle::new(std::ptr::null_mut(), 4, object_handle);

        let (pool_handle, error) = pool_handle.create_buffer(2, 2, PixelFormat::Argb8888).unwrap_err();
        assert_matches!(error, CreateBufferError::InsufficientBytes);

        let result = pool_handle.create_buffer(2, 2, PixelFormat::Y8);
        assert!(result.is_ok());
    }

    #[test]
    fn insufficient_bytes_error() {
        let (_driver, object_handle) = TestDriver::new();
        let pool_handle = WlShmPoolHandle::new(std::ptr::null_mut(), 9, object_handle);

        let (pool_handle, error) = pool_handle.create_buffer(2, 5, PixelFormat::Y8).unwrap_err();
        assert_matches!(error, CreateBufferError::InsufficientBytes);

        let result = pool_handle.create_buffer(3, 3, PixelFormat::Y8);
        assert!(result.is_ok());
    }

    #[test]
    fn unknown_density_err() {
        let (_driver, object_handle) = TestDriver::new();
        let pool_handle = WlShmPoolHandle::new(std::ptr::null_mut(), 0, object_handle);

        let error = pool_handle.create_buffer(0, 0, PixelFormat::Other(0)).unwrap_err().1;
        assert_matches!(error, CreateBufferError::UnknownPixelDensity);
    }

    #[test]
    fn pointer_overflow_err() {
        let (_driver, object_handle) = TestDriver::new();
        let pool_handle = WlShmPoolHandle::new(std::ptr::null_mut(), 0, object_handle);

        let error = pool_handle.create_buffer(u16::MAX, u16::MAX, PixelFormat::Argb8888).unwrap_err().1;
        assert_matches!(error, CreateBufferError::PointerOverflow);
    }
}

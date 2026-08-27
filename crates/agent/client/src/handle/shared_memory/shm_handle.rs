use std::collections::HashSet;

use cosplay_codec::{ArgumentDecodeError, Enumeration};
use cosplay_protocols_wayland::wl_shm::{self, PixelFormat, WlShm};

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
        for inbound_result in self.handle.event.iter() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_supported_formats() {
        let (test_driver, object_handle) = TestDriver::new();

        let mut shm_handle = WlShmHandle::from_raw(object_handle);

        assert!(shm_handle.supported_formats.is_empty());

        shm_handle.sync_supported_formats().unwrap();

        assert!(shm_handle.supported_formats.is_empty());

        test_driver.send_event(shm_handle.handle.request.id, wl_shm::Format { format: PixelFormat::C8.into() });
        test_driver.send_event(
            shm_handle.handle.request.id,
            wl_shm::Format { format: PixelFormat::Xrgb4444.into() },
        );

        shm_handle.sync_supported_formats().unwrap();

        assert!(shm_handle.supported_formats.contains(&PixelFormat::C8));
        assert!(shm_handle.supported_formats.contains(&PixelFormat::Xrgb4444));
    }

    #[test]
    fn drop_sends_release() {
        let (mut test_driver, object_handle) = TestDriver::new();

        let shm_handle = WlShmHandle::from_raw(object_handle);

        test_driver.assert_queued_destructor_on_drop::<wl_shm::Release, _>(shm_handle.handle.request.id, shm_handle);
    }
}

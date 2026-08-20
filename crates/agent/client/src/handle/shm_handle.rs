use std::collections::HashSet;

use cosplay_codec::{ArgumentDecodeError, Enumeration, Message};
use cosplay_protocols_wayland::wl_shm::{self, Format, PixelFormat, WlShm};
use tokio::sync::mpsc::error::TryRecvError;

use crate::*;

pub struct WlShmHandle {
    object_handle: ObjectHandle<WlShm>,
    supported_formats: HashSet<PixelFormat>,
}

impl Handle for WlShmHandle {
    type Interface = WlShm;

    fn from_raw(object_handle: ObjectHandle<Self::Interface>) -> Self {
        Self { object_handle, supported_formats: Default::default() }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SyncSupportedFormatsError {
    #[error("Failed to deserialize wl_shm::Format: {0}")]
    Deserialize(#[from] ArgumentDecodeError),
    #[error("Unknown event of opcode '{0}' forwarded to wl_shm.")]
    UnknownEvent(u16),
    #[error("Unhandled error event forwarded to wl_shm. {code:?}. {message}")]
    UnhandledError { code: wl_shm::Error, message: String },
    #[error("Mediator down, unable to receive supported any more format updates.")]
    Disconnected,
}

impl WlShmHandle {
    /// Get the internally buffered set of supported formats as announced by the server.
    ///
    /// Note that the internal buffer may be out of sync with queued inbound events. Most
    /// users will want to first call [`Self::sync_supported_formats`].
    pub fn supported_formats(&self) -> &HashSet<PixelFormat> {
        &self.supported_formats
    }

    /// Checks if any new supported pixel formats have been announced by the
    /// server and updates the internal set accordingly. The set can then be
    /// inspected with [`Self::supported_formats`].
    pub fn sync_supported_formats(&mut self) -> Result<(), SyncSupportedFormatsError> {
        loop {
            match self.object_handle.inbound_rx.try_recv() {
                Ok(event) => match event {
                    ObjectHandleMessage::Event(opaque_message) => {
                        let opcode = opaque_message.opcode();

                        match opcode == Format::OP_CODE {
                            true => {
                                let format = opaque_message.into_concrete::<Format>()?;
                                let format = format.format.inner();
                                self.supported_formats.insert(format);
                            }
                            false => return Err(SyncSupportedFormatsError::UnknownEvent(opcode)),
                        }
                    }
                    ObjectHandleMessage::Error { code, message } => {
                        let code = wl_shm::Error::from_repr(code);
                        return Err(SyncSupportedFormatsError::UnhandledError { code, message });
                    }
                },
                Err(error) => match error {
                    TryRecvError::Empty => {
                        return Ok(());
                    }
                    TryRecvError::Disconnected => {
                        return Err(SyncSupportedFormatsError::Disconnected);
                    }
                },
            }
        }
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

        test_driver.send_event(shm_handle.object_handle.id, wl_shm::Format { format: PixelFormat::C8.into() });
        test_driver.send_event(shm_handle.object_handle.id, wl_shm::Format { format: PixelFormat::Xrgb4444.into() });

        shm_handle.sync_supported_formats().unwrap();

        assert!(shm_handle.supported_formats.contains(&PixelFormat::C8));
        assert!(shm_handle.supported_formats.contains(&PixelFormat::Xrgb4444));
    }
}

use cosplay_codec::Enumeration;
use cosplay_protocols_wayland::wl_seat::{self, Capability, WlSeat, WlSeatEvent};

use crate::*;

pub struct WlSeatHandle {
    object_handle: ObjectHandle<WlSeat>,
    name: Option<String>,
    capability: Capability,
}

impl GlobalHandle for WlSeatHandle {
    type Interface = WlSeat;

    fn from_raw(object_handle: ObjectHandle<Self::Interface>) -> Self {
        Self { object_handle, name: None, capability: Capability::empty() }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WlSeatGetInputError {
    #[error("Failed to initialize input object: {0}")]
    Init(#[from] RequestError),
    #[error("Input device capability not announced by server.")]
    MissingCapability,
    #[error("Failed to sync state from inbound object events: {0}")]
    Events(#[from] ObjectEventsError),
}

impl WlSeatHandle {
    pub fn get_pointer(&mut self) -> Result<WlPointerHandle, WlSeatGetInputError> {
        self.sync_metadata()?;

        if !self.capability.contains(Capability::POINTER) {
            return Err(WlSeatGetInputError::MissingCapability);
        }

        self.object_handle.init_subobject().map(WlPointerHandle::new).map_err(Into::into)
    }

    fn sync_metadata(&mut self) -> Result<(), ObjectEventsError> {
        for inbound_result in self.object_handle.events_iter() {
            match inbound_result? {
                ObjectEvent::Event(event) => match event {
                    WlSeatEvent::Name(event) => {
                        self.name = Some(event.name);
                    }
                    WlSeatEvent::Capabilities(event) => {
                        self.capability = event.capabilities.inner();
                    }
                },
                ObjectEvent::Error { code, message } => {
                    // Considered "unreachable". Implementation should have
                    // encapsulated the respective safeguards.
                    //
                    // May however be triggered if there's a data race between
                    // sending get_<input> and receiving a capability removal
                    // event for the same input. The trailing capability
                    // removal would in that case close the corresponding input
                    // channels, so the resulting handle state would still be
                    // considered valid.
                    let code = wl_seat::Error::from_repr(code);
                    tracing::warn!(?code, message, "Received error.");
                }
            }
        }

        Ok(())
    }
}

impl Drop for WlSeatHandle {
    fn drop(&mut self) {
        // Unclear from spec how this affects pointer, keyboards etc.
        let _ = self.object_handle.queue_request(wl_seat::Release);
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn drop_sends_release() {
        let (mut test_driver, object_handle) = TestDriver::new();

        let seat_handle = WlSeatHandle::from_raw(object_handle);

        test_driver.assert_queued_destructor_on_drop::<wl_seat::Release, _>(seat_handle.object_handle.id, seat_handle);
    }

    #[test]
    fn get_pointer_requires_capability() {
        let (test_driver, object_handle) = TestDriver::new();

        let mut seat_handle = WlSeatHandle::from_raw(object_handle);

        let result = seat_handle.get_pointer().unwrap_err();
        assert_matches!(result, WlSeatGetInputError::MissingCapability);

        test_driver.send_event(
            seat_handle.object_handle.id,
            wl_seat::Capabilities { capabilities: Capability::KEYBOARD.into() },
        );

        let result = seat_handle.get_pointer().unwrap_err();
        assert_matches!(result, WlSeatGetInputError::MissingCapability);

        test_driver.send_event(
            seat_handle.object_handle.id,
            wl_seat::Capabilities { capabilities: Capability::POINTER.into() },
        );

        assert!(seat_handle.get_pointer().is_ok())
    }
}

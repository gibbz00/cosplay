use cosplay_codec::Enumeration;
use cosplay_protocols_wayland::wl_seat::{self, Capability, WlSeat, WlSeatEvent};

use crate::*;

pub struct WlSeatHandle {
    object_handle: ObjectHandle<WlSeat>,
    name: Option<String>,
    capability: Capability,
}

impl Handle for WlSeatHandle {
    type Interface = WlSeat;

    fn from_raw(object_handle: ObjectHandle<Self::Interface>) -> Self {
        Self { object_handle, name: None, capability: Capability::empty() }
    }
}

impl WlSeatHandle {
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
                    // Considered unreachable, implementation should have
                    // encapsulated the respective safeguards.
                    let code = wl_seat::Error::from_repr(code);
                    tracing::error!(?code, message, "Unhandled error received.");
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
    use super::*;

    #[test]
    fn drop_sends_release() {
        let (mut test_driver, object_handle) = TestDriver::new();

        let seat_handle = WlSeatHandle::from_raw(object_handle);

        test_driver.assert_queued_destructor_on_drop::<wl_seat::Release, _>(seat_handle.object_handle.id, seat_handle);
    }
}

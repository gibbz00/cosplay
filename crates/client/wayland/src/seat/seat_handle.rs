use cosplay_codec::Enumeration;
use cosplay_core_client::*;
use cosplay_protocols_wayland::wl_seat::{self, Capability, WlSeat, WlSeatEvent};

use crate::*;

/// Note, droppig `WlSeatHandle` closes the broadcast channels for announcing
/// capability removals. Message passing will in turn cease to work for device
/// handles created during the corresponding capability window.
pub struct WlSeatHandle {
    handle: ScopedObjectHandle<WlSeat>,
    name: Option<String>,
    pointer_broadcast: Option<CapabilityBroadcast<Pointer>>,
    keyboard_broadcast: Option<CapabilityBroadcast<Keyboard>>,
    touch_broadcast: Option<CapabilityBroadcast<Touch>>,
}

impl GlobalHandle for WlSeatHandle {
    type Interface = WlSeat;

    fn from_raw(handle: ObjectHandle<Self::Interface>) -> Self {
        Self {
            handle: handle.into(),
            name: None,
            pointer_broadcast: None,
            keyboard_broadcast: None,
            touch_broadcast: None,
        }
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
        self.get_device_impl()
    }

    pub fn get_keyboard(&mut self) -> Result<WlKeyboardHandle, WlSeatGetInputError> {
        self.get_device_impl()
    }

    pub fn get_touch(&mut self) -> Result<WlTouchHandle, WlSeatGetInputError> {
        self.get_device_impl()
    }

    fn get_device_impl<S, T>(&mut self) -> Result<S, WlSeatGetInputError>
    where
        Self: DeviceBroadcast<T>,
        S: DeviceHandle<Device = T>,
    {
        self.sync_metadata()?;

        let capability_rx = self
            .get_device_broadcast()
            .as_ref()
            .map(CapabilityBroadcast::subscribe)
            .ok_or(WlSeatGetInputError::MissingCapability)?;

        let object_handle = self.handle.request().init_subobject(S::request)?;

        Ok(S::new(object_handle, capability_rx))
    }

    fn sync_metadata(&mut self) -> Result<(), ObjectEventsError> {
        let events = self.handle.event().iter().collect::<Vec<_>>();

        for inbound_result in events {
            match inbound_result? {
                ObjectEvent::Event(event) => match event {
                    WlSeatEvent::Name(event) => {
                        self.name = Some(event.name);
                    }
                    WlSeatEvent::Capabilities(event) => {
                        self.handle_capability_change(event.capabilities.inner());
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

    fn handle_capability_change(&mut self, new_capability: Capability) {
        handle_capability_impl::<Pointer>(self.get_device_broadcast(), new_capability);
        handle_capability_impl::<Keyboard>(self.get_device_broadcast(), new_capability);
        handle_capability_impl::<Touch>(self.get_device_broadcast(), new_capability);

        fn handle_capability_impl<T: CapabilityBit>(broadcast_tx: &mut Option<CapabilityBroadcast<T>>, new_capability: Capability) {
            let has_capability = new_capability.contains(T::BIT);
            let has_broadcast = broadcast_tx.is_some();

            match (has_broadcast, has_capability) {
                (false, true) => {
                    *broadcast_tx = Some(CapabilityBroadcast::new());
                }
                (true, false) => {
                    // Broadcast close.
                    std::mem::take(broadcast_tx);
                }
                _ => {
                    // Nothing to do.
                }
            }
        }
    }
}

trait DeviceBroadcast<T> {
    fn get_device_broadcast(&mut self) -> &mut Option<CapabilityBroadcast<T>>;
}

impl DeviceBroadcast<Pointer> for WlSeatHandle {
    fn get_device_broadcast(&mut self) -> &mut Option<CapabilityBroadcast<Pointer>> {
        &mut self.pointer_broadcast
    }
}

impl DeviceBroadcast<Keyboard> for WlSeatHandle {
    fn get_device_broadcast(&mut self) -> &mut Option<CapabilityBroadcast<Keyboard>> {
        &mut self.keyboard_broadcast
    }
}

impl DeviceBroadcast<Touch> for WlSeatHandle {
    fn get_device_broadcast(&mut self) -> &mut Option<CapabilityBroadcast<Touch>> {
        &mut self.touch_broadcast
    }
}

trait CapabilityBit {
    const BIT: Capability;
}

impl CapabilityBit for Pointer {
    const BIT: Capability = Capability::POINTER;
}

impl CapabilityBit for Keyboard {
    const BIT: Capability = Capability::KEYBOARD;
}

impl CapabilityBit for Touch {
    const BIT: Capability = Capability::TOUCH;
}

#[cfg(test)]
mod tests {
    use std::{assert_matches, marker::PhantomData};

    use super::*;

    #[test]
    fn drop_sends_release() {
        let (mut test_driver, object_handle) = TestDriver::new();

        let seat_handle = WlSeatHandle::from_raw(object_handle);

        test_driver.assert_queued_destructor_on_drop::<wl_seat::Release, _>(seat_handle.handle.id(), seat_handle);
    }

    #[test]
    fn get_pointer_requires_capability() {
        get_device_requires_capability_impl::<WlPointerHandle, Pointer>(Capability::KEYBOARD, Capability::POINTER);
    }

    #[test]
    fn get_keyboard_requires_capability() {
        get_device_requires_capability_impl::<WlKeyboardHandle, Keyboard>(Capability::TOUCH, Capability::KEYBOARD);
    }

    #[test]
    fn get_touch_requires_capability() {
        get_device_requires_capability_impl::<WlTouchHandle, Touch>(Capability::POINTER, Capability::TOUCH);
    }

    #[test]
    fn pointer_removal_sends_broadcast() {
        removal_sends_broadcast_impl::<WlPointerHandle, Pointer>(Capability::POINTER);
    }

    #[test]
    fn keyboard_removal_sends_broadcast() {
        removal_sends_broadcast_impl::<WlKeyboardHandle, Keyboard>(Capability::KEYBOARD);
    }

    #[test]
    fn touch_removal_sends_broadcast() {
        removal_sends_broadcast_impl::<WlTouchHandle, Touch>(Capability::TOUCH);
    }

    fn get_device_requires_capability_impl<S, T>(missing_capability: Capability, contains_capability: Capability)
    where
        S: std::fmt::Debug + DeviceHandle<Device = T>,
        WlSeatHandle: DeviceBroadcast<T>,
    {
        let (test_driver, object_handle) = TestDriver::new();

        let mut seat_handle = WlSeatHandle::from_raw(object_handle);

        let id = seat_handle.handle.id();

        let mut get_device = || seat_handle.get_device_impl::<S, T>();

        assert_matches!(get_device(), Err(WlSeatGetInputError::MissingCapability));

        test_driver.send_event(id, wl_seat::Capabilities { capabilities: missing_capability.into() });

        assert_matches!(get_device(), Err(WlSeatGetInputError::MissingCapability));

        test_driver.send_event(id, wl_seat::Capabilities { capabilities: contains_capability.into() });

        // IMPROVEMENT: check that the correct request is being sent
        assert!(test_driver.request_queue_rx.is_empty());

        assert!(get_device().is_ok());

        assert!(!test_driver.request_queue_rx.is_empty());
    }

    fn removal_sends_broadcast_impl<S, T>(contains_capability: Capability)
    where
        S: DeviceHandle<Device = T>,
        WlSeatHandle: DeviceBroadcast<T>,
    {
        let (test_driver, object_handle) = TestDriver::new();

        let mut seat_handle = WlSeatHandle::from_raw(object_handle);

        let id = seat_handle.handle.id();

        assert!(seat_handle.get_device_broadcast().is_none());

        test_driver.send_event(id, wl_seat::Capabilities { capabilities: contains_capability.into() });
        seat_handle.sync_metadata().unwrap();

        let mut rx = seat_handle.get_device_broadcast().as_ref().unwrap().subscribe();

        assert!(rx.is_empty());
        assert!(!rx.is_closed());

        test_driver.send_event(id, wl_seat::Capabilities { capabilities: Capability::empty().into() });
        seat_handle.sync_metadata().unwrap();

        assert_eq!(PhantomData, rx.try_recv().unwrap());
        assert!(rx.is_closed());
    }
}

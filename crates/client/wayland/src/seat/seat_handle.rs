use cosplay_codec::Interface;
use cosplay_core_client::*;
use cosplay_protocols_wayland::wl_seat::{Capability, WlSeat, WlSeatEvent};

use crate::*;

/// Note, droppig `WlSeatHandle` closes the broadcast channels for announcing
/// capability removals. Message passing will in turn cease to work for device
/// handles created during the corresponding capability window.
pub struct SeatHandle {
    inner: ScopedObjectHandle<WlSeat>,
    name: Option<String>,
    pointer_broadcast: Option<CapabilityBroadcast<Pointer>>,
    keyboard_broadcast: Option<CapabilityBroadcast<Keyboard>>,
    touch_broadcast: Option<CapabilityBroadcast<Touch>>,
}

impl GlobalHandle for SeatHandle {
    type Interface = WlSeat;

    fn from_raw(handle: ObjectHandle<Self::Interface>) -> Self {
        Self {
            inner: handle.into(),
            name: None,
            pointer_broadcast: None,
            keyboard_broadcast: None,
            touch_broadcast: None,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SeatGetInputError {
    #[error("Failed to initialize input object: {0}")]
    Init(#[from] RequestError),
    #[error("Input device capability not announced by server.")]
    MissingCapability,
    #[error("Failed to sync state from inbound object events: {0}")]
    Events(#[from] ObjectEventsError),
}

impl SeatHandle {
    pub fn get_pointer(&mut self) -> Result<PointerHandle, SeatGetInputError> {
        self.get_device_impl()
    }

    pub fn get_keyboard(&mut self) -> Result<KeyboardHandle, SeatGetInputError> {
        self.get_device_impl()
    }

    pub fn get_touch(&mut self) -> Result<TouchHandle, SeatGetInputError> {
        self.get_device_impl()
    }

    fn get_device_impl<S, T>(&mut self) -> Result<S, SeatGetInputError>
    where
        Self: DeviceBroadcast<T>,
        S: DeviceHandle<Device = T>,
        S::Inner: Interface,
    {
        self.sync_metadata()?;

        let capability_rx = self
            .get_device_broadcast()
            .as_ref()
            .map(CapabilityBroadcast::subscribe)
            .ok_or(SeatGetInputError::MissingCapability)?;

        let object_handle = self.inner.request().init_subobject(S::request)?;

        Ok(S::new(object_handle, capability_rx))
    }

    fn sync_metadata(&mut self) -> Result<(), ObjectEventsError> {
        let events = self.inner.event().iter().collect::<Vec<_>>();

        for inbound_result in events {
            match inbound_result? {
                WlSeatEvent::Name(event) => {
                    self.name = Some(event.name);
                }
                WlSeatEvent::Capabilities(event) => {
                    self.handle_capability_change(event.capabilities.inner());
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

impl DeviceBroadcast<Pointer> for SeatHandle {
    fn get_device_broadcast(&mut self) -> &mut Option<CapabilityBroadcast<Pointer>> {
        &mut self.pointer_broadcast
    }
}

impl DeviceBroadcast<Keyboard> for SeatHandle {
    fn get_device_broadcast(&mut self) -> &mut Option<CapabilityBroadcast<Keyboard>> {
        &mut self.keyboard_broadcast
    }
}

impl DeviceBroadcast<Touch> for SeatHandle {
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

    use cosplay_protocols_wayland::wl_seat;

    use super::*;

    #[test]
    fn drop_sends_release() {
        let (mut test_driver, seat_handle) = TestDriver::new_global::<SeatHandle>();

        test_driver.assert_queued_destructor_on_drop::<wl_seat::Release, _>(seat_handle.inner.id(), seat_handle);
    }

    #[test]
    fn get_pointer_requires_capability() {
        get_device_requires_capability_impl::<PointerHandle, Pointer>(Capability::KEYBOARD, Capability::POINTER);
    }

    #[test]
    fn get_keyboard_requires_capability() {
        get_device_requires_capability_impl::<KeyboardHandle, Keyboard>(Capability::TOUCH, Capability::KEYBOARD);
    }

    #[test]
    fn get_touch_requires_capability() {
        get_device_requires_capability_impl::<TouchHandle, Touch>(Capability::POINTER, Capability::TOUCH);
    }

    #[test]
    fn pointer_removal_sends_broadcast() {
        removal_sends_broadcast_impl::<PointerHandle, Pointer>(Capability::POINTER);
    }

    #[test]
    fn keyboard_removal_sends_broadcast() {
        removal_sends_broadcast_impl::<KeyboardHandle, Keyboard>(Capability::KEYBOARD);
    }

    #[test]
    fn touch_removal_sends_broadcast() {
        removal_sends_broadcast_impl::<TouchHandle, Touch>(Capability::TOUCH);
    }

    fn get_device_requires_capability_impl<S, T>(missing_capability: Capability, contains_capability: Capability)
    where
        S: std::fmt::Debug + DeviceHandle<Device = T>,
        SeatHandle: DeviceBroadcast<T>,
        S::Inner: Interface,
    {
        let (driver, mut seat_handle) = TestDriver::new_global::<SeatHandle>();

        let id = seat_handle.inner.id();

        let mut get_device = || seat_handle.get_device_impl::<S, T>();

        assert_matches!(get_device(), Err(SeatGetInputError::MissingCapability));

        driver.send_event(id, wl_seat::Capabilities { capabilities: missing_capability.into() });

        assert_matches!(get_device(), Err(SeatGetInputError::MissingCapability));

        driver.send_event(id, wl_seat::Capabilities { capabilities: contains_capability.into() });

        // IMPROVEMENT: check that the correct request is being sent
        assert!(driver.request_queue_rx.is_empty());

        assert!(get_device().is_ok());

        assert!(!driver.request_queue_rx.is_empty());
    }

    fn removal_sends_broadcast_impl<S, T>(contains_capability: Capability)
    where
        S: DeviceHandle<Device = T>,
        SeatHandle: DeviceBroadcast<T>,
    {
        let (driver, mut seat_handle) = TestDriver::new_global::<SeatHandle>();

        let id = seat_handle.inner.id();

        assert!(seat_handle.get_device_broadcast().is_none());

        driver.send_event(id, wl_seat::Capabilities { capabilities: contains_capability.into() });
        seat_handle.sync_metadata().unwrap();

        let mut rx = seat_handle.get_device_broadcast().as_ref().unwrap().subscribe();

        assert!(rx.is_empty());
        assert!(!rx.is_closed());

        driver.send_event(id, wl_seat::Capabilities { capabilities: Capability::empty().into() });
        seat_handle.sync_metadata().unwrap();

        assert_eq!(PhantomData, rx.try_recv().unwrap());
        assert!(rx.is_closed());
    }
}

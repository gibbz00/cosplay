use cosplay_protocols_wayland::wl_keyboard::{self, WlKeyboard};

use crate::*;

#[derive(Debug)]
pub struct WlKeyboardHandle {
    object_handle: ObjectHandle<WlKeyboard>,
    capability_removed_rx: CapabilityRemovedRx<Keyboard>,
}

impl DeviceHandle for WlKeyboardHandle {
    type Device = Keyboard;
    type Inner = WlKeyboard;

    fn new(object_handle: ObjectHandle<Self::Inner>, capability_removed_rx: CapabilityRemovedRx<Keyboard>) -> Self {
        Self { object_handle, capability_removed_rx }
    }
}

impl Drop for WlKeyboardHandle {
    fn drop(&mut self) {
        let _ = self.object_handle.queue_request(wl_keyboard::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_sends_release() {
        let capability_broadcast = CapabilityBroadcast::new();

        let (mut test_driver, object_handle) = TestDriver::new();

        let keyboard_handle = WlKeyboardHandle::new(object_handle, capability_broadcast.subscribe());

        test_driver.assert_queued_destructor_on_drop::<wl_keyboard::Release, _>(keyboard_handle.object_handle.id, keyboard_handle);
    }
}

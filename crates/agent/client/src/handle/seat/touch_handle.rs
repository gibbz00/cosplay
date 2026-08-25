use cosplay_protocols_wayland::wl_touch::{self, WlTouch};

use crate::*;

#[derive(Debug)]
pub struct WlTouchHandle {
    object_handle: ObjectHandle<WlTouch>,
    capability_removed_rx: CapabilityRemovedRx<Touch>,
}

impl DeviceHandle for WlTouchHandle {
    type Device = Touch;
    type Inner = WlTouch;

    fn new(object_handle: ObjectHandle<Self::Inner>, capability_removed_rx: CapabilityRemovedRx<Touch>) -> Self {
        Self { object_handle, capability_removed_rx }
    }
}

impl Drop for WlTouchHandle {
    fn drop(&mut self) {
        let _ = self.object_handle.queue_request(wl_touch::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_sends_release() {
        let capability_broadcast = CapabilityBroadcast::new();

        let (mut test_driver, object_handle) = TestDriver::new();

        let touch_handle = WlTouchHandle::new(object_handle, capability_broadcast.subscribe());

        test_driver.assert_queued_destructor_on_drop::<wl_touch::Release, _>(touch_handle.object_handle.id, touch_handle);
    }
}

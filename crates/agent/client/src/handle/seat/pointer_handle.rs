use cosplay_protocols_wayland::wl_pointer::{self, WlPointer};

use crate::*;

#[derive(Debug)]
pub struct WlPointerHandle {
    object_handle: ObjectHandle<WlPointer>,
    capability_removed_rx: CapabilityRemovedRx<Pointer>,
}

impl DeviceHandle for WlPointerHandle {
    type Device = Pointer;
    type Inner = WlPointer;

    fn new(object_handle: ObjectHandle<Self::Inner>, capability_removed_rx: CapabilityRemovedRx<Pointer>) -> Self {
        Self { object_handle, capability_removed_rx }
    }
}

impl Drop for WlPointerHandle {
    fn drop(&mut self) {
        let _ = self.object_handle.queue_request(wl_pointer::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_sends_release() {
        let capability_broadcast = CapabilityBroadcast::new();

        let (mut test_driver, object_handle) = TestDriver::new();

        let pointer_handle = WlPointerHandle::new(object_handle, capability_broadcast.subscribe());

        test_driver.assert_queued_destructor_on_drop::<wl_pointer::Release, _>(pointer_handle.object_handle.id, pointer_handle);
    }
}

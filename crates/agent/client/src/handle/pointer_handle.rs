use cosplay_protocols_wayland::wl_pointer::{self, WlPointer};

use crate::*;

#[derive(Debug)]
pub struct WlPointerHandle {
    object_handle: ObjectHandle<WlPointer>,
}

impl WlPointerHandle {
    pub(super) fn new(object_handle: ObjectHandle<WlPointer>) -> Self {
        Self { object_handle }
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
        let (mut test_driver, object_handle) = TestDriver::new();

        let pointer_handle = WlPointerHandle::new(object_handle);

        test_driver.assert_queued_destructor_on_drop::<wl_pointer::Release, _>(pointer_handle.object_handle.id, pointer_handle);
    }
}

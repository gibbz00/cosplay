use cosplay_codec::{EncodeMessage, Message, NewObjectId};
use cosplay_core_client::*;
use cosplay_protocols_wayland::{
    wl_seat::{self, WlSeat},
    wl_touch::WlTouch,
};

use crate::*;

#[derive(Debug)]
pub struct TouchHandle {
    inner: ScopedObjectHandle<WlTouch>,
    capability_removed_rx: CapabilityRemovedRx<Touch>,
}

impl DeviceHandle for TouchHandle {
    type Device = Touch;
    type Inner = WlTouch;

    fn new(handle: ObjectHandle<Self::Inner>, capability_removed_rx: CapabilityRemovedRx<Touch>) -> Self {
        Self { inner: handle.into(), capability_removed_rx }
    }

    fn request(new_id: NewObjectId<Self::Inner>) -> impl Message<Interface = WlSeat> + EncodeMessage {
        wl_seat::GetTouch { id: new_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_sends_release() {
        let capability_broadcast = CapabilityBroadcast::new();

        let (mut test_driver, object_handle) = TestDriver::new_raw();

        let handle = TouchHandle::new(object_handle, capability_broadcast.subscribe());

        test_driver.assert_queued_destructor_on_drop::<cosplay_protocols_wayland::wl_touch::Release, _>(handle.inner.id(), handle);
    }
}

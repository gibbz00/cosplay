use cosplay_codec::{EncodeMessage, Message, NewObjectId};
use cosplay_core_client::*;
use cosplay_protocols_wayland::{
    wl_pointer::WlPointer,
    wl_seat::{self, WlSeat},
};

use crate::*;

#[derive(Debug)]
pub struct PointerHandle {
    inner: ScopedObjectHandle<WlPointer>,
    capability_removed_rx: CapabilityRemovedRx<Pointer>,
}

impl DeviceHandle for PointerHandle {
    type Device = Pointer;
    type Inner = WlPointer;

    fn new(handle: ObjectHandle<Self::Inner>, capability_removed_rx: CapabilityRemovedRx<Pointer>) -> Self {
        Self { inner: handle.into(), capability_removed_rx }
    }

    fn request(new_id: NewObjectId<Self::Inner>) -> impl Message<Interface = WlSeat> + EncodeMessage {
        wl_seat::GetPointer { id: new_id }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_sends_release() {
        let capability_broadcast = CapabilityBroadcast::new();

        let (mut test_driver, object_handle) = TestDriver::new_raw();

        let pointer_handle = PointerHandle::new(object_handle, capability_broadcast.subscribe());

        test_driver.assert_queued_destructor_on_drop::<cosplay_protocols_wayland::wl_pointer::Release, _>(
            pointer_handle.inner.id(),
            pointer_handle,
        );
    }
}

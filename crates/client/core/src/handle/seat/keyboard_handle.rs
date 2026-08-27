use cosplay_codec::{EncodeMessage, Message, NewObjectId};
use cosplay_protocols_wayland::{
    wl_keyboard::WlKeyboard,
    wl_seat::{self, WlSeat},
};

use crate::*;

#[derive(Debug)]
pub struct WlKeyboardHandle {
    handle: ScopedObjectHandle<WlKeyboard>,
    capability_removed_rx: CapabilityRemovedRx<Keyboard>,
}

impl DeviceHandle for WlKeyboardHandle {
    type Device = Keyboard;
    type Inner = WlKeyboard;

    fn new(handle: ObjectHandle<Self::Inner>, capability_removed_rx: CapabilityRemovedRx<Keyboard>) -> Self {
        Self { handle: handle.into(), capability_removed_rx }
    }

    fn request(new_id: NewObjectId<Self::Inner>) -> impl Message<Interface = WlSeat> + EncodeMessage {
        wl_seat::GetKeyboard { id: new_id }
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

        test_driver.assert_queued_destructor_on_drop::<cosplay_protocols_wayland::wl_keyboard::Release, _>(
            keyboard_handle.handle.request.id,
            keyboard_handle,
        );
    }
}

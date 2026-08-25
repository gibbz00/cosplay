use cosplay_codec::{EncodeMessage, Message, NewObjectId};
use cosplay_protocols_wayland::wl_seat::WlSeat;

use crate::*;

pub struct Pointer;

pub struct Keyboard;

pub struct Touch;

pub(crate) trait DeviceHandle {
    type Inner;
    type Device;

    fn request(new_id: NewObjectId<Self::Inner>) -> impl Message<Interface = WlSeat> + EncodeMessage;

    fn new(object_handle: ObjectHandle<Self::Inner>, capability_removed_rx: CapabilityRemovedRx<Self::Device>) -> Self;
}

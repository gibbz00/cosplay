use crate::*;

pub struct Pointer;

pub struct Keyboard;

pub struct Touch;

pub(crate) trait DeviceHandle {
    type Inner;
    type Device;

    fn new(object_handle: ObjectHandle<Self::Inner>, capability_removed_rx: CapabilityRemovedRx<Self::Device>) -> Self;
}

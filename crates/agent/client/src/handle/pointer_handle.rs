use cosplay_protocols_wayland::wl_pointer::WlPointer;

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

use cosplay_core_client::ObjectHandle;
use cosplay_protocols_wayland::wl_buffer::WlBuffer;

use crate::*;

// A `wl_buffer` created by a `WlShmPoolHandle`.
#[derive(Debug)]
pub struct WlBufferHandle {
    handle: ObjectHandle<WlBuffer>,
}

impl WlBufferHandle {
    pub(super) fn new(handle: ObjectHandle<WlBuffer>) -> Self {
        Self { handle }
    }
}

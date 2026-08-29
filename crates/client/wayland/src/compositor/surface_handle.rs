use cosplay_codec::ObjectId;
use cosplay_core_client::*;
use cosplay_protocols_wayland::wl_surface::{self, WlSurface};

use crate::WlCombinedBufferHandle;

/// Handle to a `wl_surface` instance.
///
/// It is up the the creator of WlSurface to ensure its roles do not change.
///
/// Drop implementation queues a [`wl_surface::Destroy`] request.
pub struct WlSurfaceHandle {
    handle: ScopedObjectHandle<WlSurface>,
    pending_buffer: Option<WlCombinedBufferHandle>,
}

impl WlSurfaceHandle {
    pub fn id(&self) -> ObjectId<WlSurface> {
        self.handle.id()
    }

    // FIXME: Return buffer if operation failed? pass &mut and to mem::take?
    pub fn attach(&mut self, buffer: Option<WlCombinedBufferHandle>) -> Result<(), RequestError> {
        self.pending_buffer = buffer;

        self.handle.request().enqueue(wl_surface::Attach {
            buffer: self.pending_buffer.as_ref().map(WlCombinedBufferHandle::id),
            // See official `wl_surface::attach` for why x and y should be set
            // to zero. (Deprecated in favor of wl_surface::offset.)
            x: 0,
            y: 0,
        })
    }

    pub fn commit(&self) -> Result<(), RequestError> {
        self.handle.request().enqueue(wl_surface::Commit)
    }

    pub(crate) fn new(handle: ObjectHandle<WlSurface>) -> Self {
        Self { handle: handle.into(), pending_buffer: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_sends_destroy() {
        let (mut test_driver, object_handle) = TestDriver::new();

        let handle = WlSurfaceHandle::new(object_handle);

        test_driver.assert_queued_destructor_on_drop::<cosplay_protocols_wayland::wl_surface::Destroy, _>(handle.id(), handle);
    }
}

use std::collections::VecDeque;

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
    /// Important to store a handle which is equal to or exceeds the lifetime of the buffer.
    ///
    /// "If a pending wl_buffer has been destroyed, the result is not specified... Clients seeking
    /// to maximise compatibility should not destroy pending buffers...
    pending_buffer: Option<WlCombinedBufferHandle>,
    applied_buffers: VecDeque<WlCombinedBufferHandle>,
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

    /// No-op if there are no pending buffers.
    pub fn commit(&mut self) -> Result<(), RequestError> {
        if let Some(pending_buffer) = std::mem::take(&mut self.pending_buffer) {
            self.handle.request().enqueue(wl_surface::Commit)?;
            self.applied_buffers.push_back(pending_buffer);
        }

        Ok(())
    }

    pub(crate) fn new(handle: ObjectHandle<WlSurface>) -> Self {
        Self {
            handle: handle.into(),
            pending_buffer: None,
            applied_buffers: VecDeque::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_sends_destroy() {
        let (mut driver, raw_handle) = TestDriver::new_raw();
        let handle = WlSurfaceHandle::new(raw_handle);

        driver.assert_queued_destructor_on_drop::<cosplay_protocols_wayland::wl_surface::Destroy, _>(handle.id(), handle);
    }
}

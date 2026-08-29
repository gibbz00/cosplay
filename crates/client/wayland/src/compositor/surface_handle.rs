use cosplay_codec::ObjectId;
use cosplay_core_client::*;
use cosplay_protocols_wayland::wl_surface::{self, WlSurface};

use crate::*;

pub struct Empty {
    _priv: (),
}

pub struct Pending {
    /// Important to store a handle which is equal to or exceeds the lifetime of
    /// the buffer in order to avoid a destroy request on drop.
    ///
    /// "If a pending wl_buffer has been destroyed, the result is not
    /// specified... Clients seeking to maximise compatibility should not
    /// destroy pending buffers..."
    buffer: Option<ShmBuffer<Available>>,
}

/// Handle to a `wl_surface` instance.
///
/// It is up the the creator of WlSurface to ensure its roles do not change.
///
/// Drop implementation queues a [`wl_surface::Destroy`] request.
pub struct WlSurfaceHandle<S> {
    state: S,

    handle: ScopedObjectHandle<WlSurface>,
}

impl<S> WlSurfaceHandle<S> {
    pub fn id(&self) -> ObjectId<WlSurface> {
        self.handle.id()
    }
}

impl WlSurfaceHandle<Empty> {
    pub(crate) fn empty(handle: ObjectHandle<WlSurface>) -> Self {
        Self { state: Empty { _priv: () }, handle: handle.into() }
    }

    /// Committing an empty surface.
    ///
    /// Mostly used as way to trigger a surface initialization procedure (say for an `xdg_surface`).
    ///
    /// See [`WlSurfaceHandle:<Pending>::commit`] for the more common commit use-case.
    pub fn commit(&self) -> Result<(), RequestError> {
        self.handle.request().enqueue(wl_surface::Commit)
    }

    // FIXME: Return buffer if operation failed?
    pub fn attach(self, buffer: Option<ShmBuffer<Available>>) -> Result<WlSurfaceHandle<Pending>, RequestError> {
        self.handle.request().enqueue(wl_surface::Attach {
            buffer: buffer.as_ref().map(ShmBuffer::id),
            // See official `wl_surface::attach` for why x and y should be set
            // to zero. (Deprecated in favor of wl_surface::offset.)
            x: 0,
            y: 0,
        })?;

        Ok(WlSurfaceHandle { state: Pending { buffer }, handle: self.handle })
    }
}

impl WlSurfaceHandle<Pending> {
    // FIXME: Return buffer if operation failed?
    pub fn commit(self) -> Result<(WlSurfaceHandle<Empty>, Option<ShmBuffer<Committed>>), RequestError> {
        self.handle.request().enqueue(wl_surface::Commit)?;

        let this = WlSurfaceHandle { state: Empty { _priv: () }, handle: self.handle };

        let buffer = self.state.buffer.map(ShmBuffer::committed);

        Ok((this, buffer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_sends_destroy() {
        let (mut driver, raw_handle) = TestDriver::new_raw();
        let handle = WlSurfaceHandle::empty(raw_handle);

        driver.assert_queued_destructor_on_drop::<cosplay_protocols_wayland::wl_surface::Destroy, _>(handle.id(), handle);
    }
}

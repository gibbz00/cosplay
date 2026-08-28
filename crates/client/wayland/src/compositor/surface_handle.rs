use std::marker::PhantomData;

use cosplay_codec::ObjectId;
use cosplay_core_client::*;
use cosplay_protocols_wayland::wl_surface::{self, WlSurface};

/// Handle to a `wl_surface` instance.
///
/// Drop implementation queues a [`wl_surface::Destroy`] request.
pub struct WlSurfaceHandle<R> {
    handle: ScopedObjectHandle<WlSurface>,
    role_marker: PhantomData<R>,
}

pub struct UnassignedRole {
    _priv: (),
}

impl<R> WlSurfaceHandle<R> {
    pub fn id(&self) -> ObjectId<WlSurface> {
        self.handle.id()
    }

    pub fn commit(&self) -> Result<(), RequestError> {
        self.handle.request().enqueue(wl_surface::Commit)
    }
}

impl WlSurfaceHandle<UnassignedRole> {
    pub(crate) fn new(handle: ObjectHandle<WlSurface>) -> Self {
        Self { handle: handle.into(), role_marker: PhantomData }
    }

    pub fn with_role<R>(self) -> WlSurfaceHandle<R> {
        WlSurfaceHandle { handle: self.handle, role_marker: PhantomData }
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

use std::marker::PhantomData;

use cosplay_codec::ObjectId;
use cosplay_core_client::*;
use cosplay_protocols_wayland::wl_surface::WlSurface;

use crate::*;

pub struct UnassignedRole;

/// Handle to a `wl_surface` instance.
///
/// Drop implementation queues a [`wl_surface::Destroy`] request.
pub struct WlSurfaceHandle<R> {
    handle: ScopedObjectHandle<WlSurface>,
    role_marker: PhantomData<R>,
}

impl GlobalHandle for WlSurfaceHandle<UnassignedRole> {
    type Interface = WlSurface;

    fn from_raw(handle: ObjectHandle<Self::Interface>) -> Self {
        Self { handle: handle.into(), role_marker: PhantomData }
    }
}

impl<R> WlSurfaceHandle<R> {
    pub fn id(&self) -> ObjectId<WlSurface> {
        self.handle.id()
    }
}

impl WlSurfaceHandle<UnassignedRole> {
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

        let handle = WlSurfaceHandle::from_raw(object_handle);

        test_driver.assert_queued_destructor_on_drop::<cosplay_protocols_wayland::wl_surface::Destroy, _>(handle.id(), handle);
    }
}

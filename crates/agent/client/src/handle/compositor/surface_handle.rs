use cosplay_protocols_wayland::wl_surface::WlSurface;

use crate::*;

/// Handle to a `wl_surface` instance.
///
/// Drop implementation queues a [`wl_surface::Destroy`] request.
pub struct WlSurfaceHandle {
    object_handle: ScopedObjectHandle<WlSurface>,
}

impl GlobalHandle for WlSurfaceHandle {
    type Interface = WlSurface;

    fn from_raw(object_handle: ObjectHandle<Self::Interface>) -> Self {
        Self { object_handle: object_handle.into() }
    }
}

impl WlSurfaceHandle {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_sends_destroy() {
        let (mut test_driver, object_handle) = TestDriver::new();

        let handle = WlSurfaceHandle::from_raw(object_handle);

        test_driver.assert_queued_destructor_on_drop::<cosplay_protocols_wayland::wl_surface::Destroy, _>(handle.object_handle.id, handle);
    }
}

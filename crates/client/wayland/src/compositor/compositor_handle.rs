use cosplay_core_client::*;
use cosplay_protocols_wayland::wl_compositor::{self, WlCompositor};

use crate::*;

/// Handle to a `wl_compositor` instance.
///
/// Drop implementation automatically queue a [`wl_compositor::Release`] request.
pub struct CompositorHandle {
    handle: ScopedObjectHandle<WlCompositor>,
}

impl GlobalHandle for CompositorHandle {
    type Interface = WlCompositor;

    fn from_raw(handle: ObjectHandle<Self::Interface>) -> Self {
        Self { handle: handle.into() }
    }
}

impl CompositorHandle {
    // Wrapper for sending [`wl_compositor::CreateSurface`].
    pub fn create_surface(&self) -> Result<SurfaceHandle<Empty>, RequestError> {
        self.handle
            .request()
            .init_subobject(|id| wl_compositor::CreateSurface { id })
            .map(SurfaceHandle::empty)
    }

    // Wrapper for sending [`wl_compositor::CreateRegion`].
    pub fn create_region(&self) -> Result<RegionHandle, RequestError> {
        self.handle
            .request()
            .init_subobject(|id| wl_compositor::CreateRegion { id })
            .map(RegionHandle::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_sends_release() {
        let (mut driver, handle) = TestDriver::new_global::<CompositorHandle>();

        driver.assert_queued_destructor_on_drop::<wl_compositor::Release, _>(handle.handle.id(), handle);
    }
}

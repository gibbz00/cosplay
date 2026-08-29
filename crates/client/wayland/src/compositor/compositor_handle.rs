use cosplay_core_client::*;
use cosplay_protocols_wayland::wl_compositor::{self, WlCompositor};

use crate::*;

/// Handle to a `wl_compositor` instance.
///
/// Drop implementation automatically queue a [`wl_compositor::Release`] request.
pub struct WlCompositorHandle {
    handle: ScopedObjectHandle<WlCompositor>,
}

impl GlobalHandle for WlCompositorHandle {
    type Interface = WlCompositor;

    fn from_raw(handle: ObjectHandle<Self::Interface>) -> Self {
        Self { handle: handle.into() }
    }
}

impl WlCompositorHandle {
    // Wrapper for sending [`wl_compositor::CreateSurface`].
    pub fn create_surface(&self) -> Result<WlSurfaceHandle, RequestError> {
        self.handle
            .request()
            .init_subobject(|id| wl_compositor::CreateSurface { id })
            .map(WlSurfaceHandle::new)
    }

    // Wrapper for sending [`wl_compositor::CreateRegion`].
    pub fn create_region(&self) -> Result<WlRegionHandle, RequestError> {
        self.handle
            .request()
            .init_subobject(|id| wl_compositor::CreateRegion { id })
            .map(WlRegionHandle::new)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_sends_release() {
        let (mut driver, handle) = TestDriver::new_global::<WlCompositorHandle>();

        driver.assert_queued_destructor_on_drop::<wl_compositor::Release, _>(handle.handle.id(), handle);
    }
}

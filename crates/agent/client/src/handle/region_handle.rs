use cosplay_agent::geometry::Rectangle;
use cosplay_protocols_wayland::wl_region::{self, WlRegion};

use crate::*;

/// Handle to a `wl_region` instance.
///
/// Drop implementation automatically queue a [`wl_region::Destroy`] request.
pub struct WlRegionHandle {
    object_handle: ScopedObjectHandle<WlRegion>,
}

impl GlobalHandle for WlRegionHandle {
    type Interface = WlRegion;

    fn from_raw(object_handle: ObjectHandle<Self::Interface>) -> Self {
        Self { object_handle: object_handle.into() }
    }
}

impl WlRegionHandle {
    // Wrapper for sending [`wl_region::Add`].
    pub fn add(&self, rectangle: &Rectangle) -> Result<(), RequestError> {
        let Rectangle { x, y, width, height } = rectangle.clone();
        self.object_handle.queue_request(wl_region::Add { x, y, width, height })
    }

    // Wrapper for sending [`wl_region::Subtract`].
    pub fn subtract(&self, rectangle: &Rectangle) -> Result<(), RequestError> {
        let Rectangle { x, y, width, height } = rectangle.clone();
        self.object_handle.queue_request(wl_region::Subtract { x, y, width, height })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_sends_destroy() {
        let (mut test_driver, object_handle) = TestDriver::new();

        let handle = WlRegionHandle::from_raw(object_handle);

        test_driver.assert_queued_destructor_on_drop::<cosplay_protocols_wayland::wl_region::Destroy, _>(handle.object_handle.id, handle);
    }

    #[test]
    fn add() {
        let (mut test_driver, object_handle) = TestDriver::new();

        let handle = WlRegionHandle::from_raw(object_handle);

        handle.add(&Rectangle { x: 1, y: 2, width: 3, height: 4 }).unwrap();

        let outbound = test_driver.assert_outbound_request(handle.object_handle.id);

        let wl_region::Add { x, y, width, height } = outbound;

        assert_eq!(x, 1);
        assert_eq!(y, 2);
        assert_eq!(width, 3);
        assert_eq!(height, 4);
    }

    #[test]
    fn subtract() {
        let (mut test_driver, object_handle) = TestDriver::new();

        let handle = WlRegionHandle::from_raw(object_handle);

        handle.subtract(&Rectangle { x: 4, y: 3, width: 2, height: 1 }).unwrap();

        let outbound = test_driver.assert_outbound_request(handle.object_handle.id);

        let wl_region::Subtract { x, y, width, height } = outbound;

        assert_eq!(x, 4);
        assert_eq!(y, 3);
        assert_eq!(width, 2);
        assert_eq!(height, 1);
    }
}

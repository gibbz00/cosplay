use cosplay_agent::geometry::Rectangle;
use cosplay_core_client::*;
use cosplay_protocols_wayland::wl_region::{self, WlRegion};

/// Handle to a `wl_region` instance.
///
/// Drop implementation queues a [`wl_region::Destroy`] request.
pub struct WlRegionHandle {
    handle: ScopedObjectHandle<WlRegion>,
}

impl GlobalHandle for WlRegionHandle {
    type Interface = WlRegion;

    fn from_raw(handle: ObjectHandle<Self::Interface>) -> Self {
        Self { handle: handle.into() }
    }
}

impl WlRegionHandle {
    // Wrapper for sending [`wl_region::Add`].
    pub fn add(&self, rectangle: &Rectangle) -> Result<(), RequestError> {
        let Rectangle { x, y, width, height } = rectangle.clone();
        self.handle.request().enqueue(wl_region::Add { x, y, width, height })
    }

    // Wrapper for sending [`wl_region::Subtract`].
    pub fn subtract(&self, rectangle: &Rectangle) -> Result<(), RequestError> {
        let Rectangle { x, y, width, height } = rectangle.clone();
        self.handle.request().enqueue(wl_region::Subtract { x, y, width, height })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_sends_destroy() {
        let (mut test_driver, object_handle) = TestDriver::new();

        let wl_region = WlRegionHandle::from_raw(object_handle);

        test_driver.assert_queued_destructor_on_drop::<cosplay_protocols_wayland::wl_region::Destroy, _>(wl_region.handle.id(), wl_region);
    }

    #[test]
    fn add() {
        let (mut test_driver, object_handle) = TestDriver::new();

        let handle = WlRegionHandle::from_raw(object_handle);

        handle.add(&Rectangle { x: 1, y: 2, width: 3, height: 4 }).unwrap();

        let outbound = test_driver.assert_outbound_request(handle.handle.id());

        let expected = wl_region::Add { x: 1, y: 2, width: 3, height: 4 };

        assert_eq!(expected, outbound);
    }

    #[test]
    fn subtract() {
        let (mut test_driver, object_handle) = TestDriver::new();

        let wl_region = WlRegionHandle::from_raw(object_handle);

        wl_region.subtract(&Rectangle { x: 4, y: 3, width: 2, height: 1 }).unwrap();

        let outbound = test_driver.assert_outbound_request(wl_region.handle.id());

        let expected = wl_region::Subtract { x: 4, y: 3, width: 2, height: 1 };

        assert_eq!(expected, outbound);
    }
}

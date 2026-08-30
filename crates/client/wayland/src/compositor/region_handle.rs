use cosplay_agent::geometry::Rectangle;
use cosplay_core_client::*;
use cosplay_protocols_wayland::wl_region::{self, WlRegion};

/// Handle to a `wl_region` instance.
///
/// Drop implementation queues a [`wl_region::Destroy`] request.
pub struct RegionHandle {
    inner: ScopedObjectHandle<WlRegion>,
}

impl RegionHandle {
    pub(crate) fn new(handle: ObjectHandle<WlRegion>) -> Self {
        Self { inner: handle.into() }
    }

    // Wrapper for sending [`wl_region::Add`].
    pub fn add(&self, rectangle: &Rectangle) -> Result<(), RequestError> {
        let Rectangle { x, y, width, height } = rectangle.clone();

        self.inner
            .request()
            .enqueue(wl_region::Add { x, y, width: width as i32, height: height as i32 })
    }

    // Wrapper for sending [`wl_region::Subtract`].
    pub fn subtract(&self, rectangle: &Rectangle) -> Result<(), RequestError> {
        let Rectangle { x, y, width, height } = rectangle.clone();

        self.inner
            .request()
            .enqueue(wl_region::Subtract { x, y, width: width as i32, height: height as i32 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drop_sends_destroy() {
        let (mut driver, object_handle) = TestDriver::new_raw();

        let region = RegionHandle::new(object_handle);

        driver.assert_queued_destructor_on_drop::<cosplay_protocols_wayland::wl_region::Destroy, _>(region.inner.id(), region);
    }

    #[test]
    fn add() {
        let (mut driver, object_handle) = TestDriver::new_raw();
        let handle = RegionHandle::new(object_handle);

        handle.add(&Rectangle { x: 1, y: 2, width: 3, height: 4 }).unwrap();

        let outbound = driver.assert_outbound_request(handle.inner.id());

        let expected = wl_region::Add { x: 1, y: 2, width: 3, height: 4 };

        assert_eq!(expected, outbound);
    }

    #[test]
    fn subtract() {
        let (mut driver, object_handle) = TestDriver::new_raw();
        let region = RegionHandle::new(object_handle);

        region.subtract(&Rectangle { x: 4, y: 3, width: 2, height: 1 }).unwrap();

        let outbound = driver.assert_outbound_request(region.inner.id());

        let expected = wl_region::Subtract { x: 4, y: 3, width: 2, height: 1 };

        assert_eq!(expected, outbound);
    }
}

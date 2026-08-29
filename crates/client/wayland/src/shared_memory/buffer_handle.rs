use cosplay_codec::ObjectId;
use cosplay_core_client::{ObjectHandle, ScopedObjectHandle};
use cosplay_protocols_wayland::{wl_buffer::WlBuffer, wl_shm_pool::WlShmPool};

use crate::*;

// A `wl_buffer` created by a `WlShmPoolHandle`.
#[derive(Debug)]
pub struct WlCombinedBufferHandle {
    pool_handle: ScopedObjectHandle<WlShmPool>,
    buffer_handle: ScopedObjectHandle<WlBuffer>,
    region: ShmRegion,
}

impl WlCombinedBufferHandle {
    pub(crate) fn id(&self) -> ObjectId<WlBuffer> {
        self.buffer_handle.id()
    }

    pub(super) fn new(pool_handle: ObjectHandle<WlShmPool>, buffer_handle: ObjectHandle<WlBuffer>, region: ShmRegion) -> Self {
        Self {
            pool_handle: pool_handle.into(),
            buffer_handle: buffer_handle.into(),
            region,
        }
    }

    pub fn region(&self) -> &ShmRegion {
        &self.region
    }

    pub fn region_mut(&mut self) -> &mut ShmRegion {
        &mut self.region
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use cosplay_core_client::TestDriver;
    use cosplay_protocols_wayland::{
        wl_buffer,
        wl_shm::PixelFormat,
        wl_shm_pool::{self, CreateBuffer},
    };

    use super::*;

    #[test]
    fn destroy_handles_on_drop() {
        let (mut driver, wl_shm_pool) = TestDriver::new_raw::<WlShmPool>();

        let wl_buffer = wl_shm_pool
            .request()
            .init_subobject(|id| CreateBuffer {
                id,
                offset: 0,
                width: 0,
                height: 0,
                stride: 0,
                format: PixelFormat::Argb8888.into(),
            })
            .unwrap();

        let pool_id = wl_shm_pool.id();
        let buffer_id = wl_buffer.id();

        let (shm_ptr, _fd) = ShmRegion::new(NonZeroUsize::new(1).unwrap()).unwrap();

        let combined_buffer = WlCombinedBufferHandle::new(wl_shm_pool, wl_buffer, shm_ptr);

        drop(combined_buffer);

        driver.assert_outbound_request::<CreateBuffer>(pool_id);
        driver.assert_outbound_request::<wl_shm_pool::Destroy>(pool_id);
        driver.assert_outbound_request::<wl_buffer::Destroy>(buffer_id);
    }
}

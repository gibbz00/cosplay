use std::marker::PhantomData;

use cosplay_codec::ObjectId;
use cosplay_core_client::{ObjectHandle, ScopedObjectHandle};
use cosplay_protocols_wayland::{wl_buffer::WlBuffer, wl_shm_pool::WlShmPool};

use crate::*;

pub struct Committed {
    _priv: (),
}

pub struct Available {
    _priv: (),
}

#[impl_tools::autoimpl(Debug)]
pub struct ShmBuffer<S> {
    state_marker: PhantomData<S>,

    pool_handle: ScopedObjectHandle<WlShmPool>,
    /// Buffer can safely be destroyed before compositor is done with processing the a surface
    /// commit as long as ShmRegion isn't taken and used for something else.
    buffer_handle: ScopedObjectHandle<WlBuffer>,
    /// # Safety
    ///
    /// Do not, in any circumstance, support taking out the Shm on a committed buffer,
    /// before the receival of a `wl_buffer::release` event. (See the official
    /// `wl_surface::attach` documentation for more.)
    region: ShmRegion,
}

impl ShmBuffer<Available> {
    /// # Panics
    ///
    /// Panics if `src.len()` > `self.len()`.
    pub fn write(&mut self, src: &[u8]) {
        // SAFETY: Server should only read from buffer if the buffer is committed. This invariant is ensured
        // by returning said buffer as `Shmbuffer<Committed>` from `WlSurfaceHandle::commit`.
        unsafe { self.region.write(src) };
    }

    pub(super) fn new(pool_handle: ObjectHandle<WlShmPool>, buffer_handle: ObjectHandle<WlBuffer>, region: ShmRegion) -> Self {
        Self {
            state_marker: PhantomData,
            pool_handle: pool_handle.into(),
            buffer_handle: buffer_handle.into(),
            region,
        }
    }

    /// Expected to only be called in [`WlSurfaceHandle::<Pending>::commit`].
    pub(crate) fn committed(self) -> ShmBuffer<Committed> {
        let Self { pool_handle, buffer_handle, region, .. } = self;
        ShmBuffer { state_marker: PhantomData, pool_handle, buffer_handle, region }
    }
}

impl<S> ShmBuffer<S> {
    pub(crate) fn id(&self) -> ObjectId<WlBuffer> {
        self.buffer_handle.id()
    }

    // Never empty since ShmRegion::new requires len of NonZeroUsize.
    #[expect(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.region.len()
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

        let shm_buffer = ShmBuffer::new(wl_shm_pool, wl_buffer, shm_ptr);

        drop(shm_buffer);

        driver.assert_outbound_request::<CreateBuffer>(pool_id);
        driver.assert_outbound_request::<wl_shm_pool::Destroy>(pool_id);
        driver.assert_outbound_request::<wl_buffer::Destroy>(buffer_id);
    }
}

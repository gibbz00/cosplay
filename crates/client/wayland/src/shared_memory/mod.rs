mod ptr {
    use std::{ffi::c_void, ptr::NonNull};

    /// Memory mapped pointer to an in-memory file that has
    /// been sent to the server with `wl_shm::create_pool`.
    ///
    /// Wrapped to avoid accidental copies etc.
    #[derive(Debug)]
    pub(crate) struct ShmPtr {
        inner: NonNull<c_void>,
        len: usize,
    }

    impl ShmPtr {
        pub(super) fn new(inner: NonNull<c_void>, len: i32) -> Self {
            Self { inner, len: len as usize }
        }

        pub(super) fn len(&self) -> usize {
            self.len
        }
    }
}
pub(crate) use ptr::ShmPtr;

mod shm_handle;
pub use shm_handle::{SyncSupportedFormatsError, WlShmHandle};

mod buffer_handle;
pub use buffer_handle::WlCombinedBufferHandle;

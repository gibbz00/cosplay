mod object_handle;
pub(crate) use object_handle::ObjectHandleTx;
pub use object_handle::{ObjectHandle, ObjectHandleMessage};

mod sync_handle;
pub(crate) use sync_handle::SyncDoneTx;
pub use sync_handle::{SyncError, SyncHandle};

mod registry_handle;
pub use registry_handle::RegistryHandle;

mod wrapper;
pub(crate) use wrapper::Handle;

mod shm_handle;
pub use shm_handle::WlShmHandle;

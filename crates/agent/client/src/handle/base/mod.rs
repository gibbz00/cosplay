mod sync_handle;
pub(crate) use sync_handle::SyncDoneTx;
pub use sync_handle::{SyncError, SyncHandle};

mod registry_handle;
pub use registry_handle::{GlobalHandle, RegistryHandle};

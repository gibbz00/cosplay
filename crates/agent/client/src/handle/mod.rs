mod object_handle;
pub(crate) use object_handle::ObjectHandleTx;
pub use object_handle::{ObjectEvent, ObjectEventsError, ObjectEventsIter, ObjectHandle, ObjectHandleMessage};

mod sync_handle;
pub(crate) use sync_handle::SyncDoneTx;
pub use sync_handle::{SyncError, SyncHandle};

mod registry_handle;
pub use registry_handle::{GlobalHandle, RegistryHandle};

mod shm_handle;
pub use shm_handle::{SyncSupportedFormatsError, WlShmHandle};

mod compositor_handle;
pub use compositor_handle::WlCompositorHandle;

mod seat;
pub(crate) use seat::*;
pub use seat::{WlKeyboardHandle, WlPointerHandle, WlSeatGetInputError, WlSeatHandle, WlTouchHandle};

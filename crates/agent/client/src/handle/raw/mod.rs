mod object_handle;
pub use object_handle::{ObjectHandle, ObjectHandleMessage};
pub(crate) use object_handle::{ObjectHandleRx, ObjectHandleTx};

mod split_handle;
pub(crate) use split_handle::{EventHandle, RequestHandle};

mod iterator;
pub use iterator::{ObjectEvent, ObjectEventsError, ObjectEventsIter};

mod scoped_object_handle;
pub(crate) use scoped_object_handle::ScopedObjectHandle;

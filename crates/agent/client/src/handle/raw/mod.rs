mod object_handle;
pub(crate) use object_handle::ObjectHandleTx;
pub use object_handle::{ObjectEvent, ObjectEventsError, ObjectEventsIter, ObjectHandle, ObjectHandleMessage};

mod scoped_object_handle;
pub(crate) use scoped_object_handle::ScopedObjectHandle;

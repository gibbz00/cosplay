// TEMP:
#![allow(missing_docs)]

//! # `cosplay-core-client`

mod setup;
pub use setup::{ClientSetup, ClientSetupError};

mod path;
pub use path::{SocketPath, SocketPathError};

mod request_queue;
pub use request_queue::RequestQueue;
pub(crate) use request_queue::RequestQueueTx;

mod event_mediator;
pub use event_mediator::EventMediator;
pub(crate) use event_mediator::{MediatorMessage, MediatorTx};

mod request_handle;
pub use request_handle::{RequestError, RequestHandle};

mod event_handle;
pub use event_handle::{EventHandle, ObjectEvent, ObjectEventsError, ObjectEventsIter};

mod object_handle;
pub use object_handle::{ObjectHandle, ObjectHandleMessage};
pub(crate) use object_handle::{ObjectHandleRx, ObjectHandleTx};

mod scoped_handle;
pub use scoped_handle::ScopedObjectHandle;

mod sync_handle;
pub(crate) use sync_handle::SyncDoneTx;
pub use sync_handle::{SyncError, SyncHandle};

mod registry_handle;
pub use registry_handle::{GlobalHandle, RegistryHandle};

#[cfg(feature = "test-driver")]
mod test_driver;
#[cfg(feature = "test-driver")]
pub use test_driver::TestDriver;

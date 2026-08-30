// TEMP:
#![allow(missing_docs)]

//! # `cosplay-core-client`

mod setup;
pub use setup::{ClientSetup, ClientSetupError};

mod path;
pub use path::{SocketPath, SocketPathError};

mod actors {
    mod request_queue;
    pub use request_queue::RequestQueue;
    pub(crate) use request_queue::RequestQueueTx;

    mod event_mediator;
    pub use event_mediator::EventMediator;
    pub(crate) use event_mediator::{MediatorMessage, MediatorTx};

    #[cfg(feature = "test-driver")]
    mod test_driver;
    #[cfg(feature = "test-driver")]
    pub use test_driver::TestDriver;
}
pub use actors::*;

mod raw_handles {
    mod request;
    pub use request::{RequestError, RequestHandle};

    mod event;
    pub use event::{EventHandle, ObjectEventsError, ObjectEventsIter};

    mod object;
    pub use object::{ObjectHandle, ObjectHandleMessage};
    pub(crate) use object::{ObjectHandleRx, ObjectHandleTx};

    mod scoped;
    pub use scoped::ScopedObjectHandle;
}
pub use raw_handles::*;

mod base_handles {
    mod callback;
    pub use callback::CallbackHandle;

    mod sync;
    pub use sync::{SyncError, SyncHandle};

    mod registry;
    pub use registry::{GlobalHandle, RegistryHandle};
}
pub use base_handles::*;

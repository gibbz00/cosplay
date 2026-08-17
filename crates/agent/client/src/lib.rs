// TEMP:
#![allow(missing_docs)]

//! # `cosplay-client`

mod setup;
pub use setup::{ClientSetup, ClientSetupError};

mod path;
pub use path::{SocketPath, SocketPathError};

mod request_queue;
pub use request_queue::RequestQueue;
pub(crate) use request_queue::RequestQueueTx;

mod request_error;
pub use request_error::RequestError;

mod event_mediator;
pub use event_mediator::EventMediator;
pub(crate) use event_mediator::{MediatorMessage, MediatorTx};

mod object_handle;
pub(crate) use object_handle::{ObjectHandle, ObjectHandleMessage, ObjectHandleTx};

mod sync_handle;
pub(crate) use sync_handle::SyncDoneTx;
pub use sync_handle::{SyncError, SyncHandle};

mod registry_handle;
pub use registry_handle::RegistryHandle;

mod registry_map;
pub(crate) use registry_map::{RegistryEntry, RegistryMap};

mod message_utils;
pub(crate) use message_utils::MessageUtils;

// TEMP:
#![allow(missing_docs)]

//! # `cosplay-client`

mod setup;
pub use setup::{ClientSetup, ClientSetupError};

mod path;
pub use path::{SocketPath, SocketPathError};

mod request_queue;
pub use request_queue::RequestQueue;
#[cfg(test)]
pub(crate) use request_queue::RequestQueueRx;
pub(crate) use request_queue::RequestQueueTx;

mod request_error;
pub use request_error::RequestError;

mod event_mediator;
pub use event_mediator::EventMediator;
#[cfg(test)]
pub(crate) use event_mediator::MediatorRx;
pub(crate) use event_mediator::{MediatorMessage, MediatorTx};

mod handle;
pub(crate) use handle::Handle;

mod object_handle;
pub(crate) use object_handle::ObjectHandleTx;
pub use object_handle::{ObjectHandle, ObjectHandleMessage};

mod sync_handle;
pub(crate) use sync_handle::SyncDoneTx;
pub use sync_handle::{SyncError, SyncHandle};

mod registry_handle;
pub use registry_handle::RegistryHandle;

mod registry_map;
pub(crate) use registry_map::{RegistryEntry, RegistryMap};

// handle impls

mod shm_handle;
pub use shm_handle::WlShmHandle;

mod message_utils;
pub(crate) use message_utils::MessageUtils;

#[cfg(test)]
mod test_utils;
#[cfg(test)]
pub(crate) use test_utils::TestDriver;

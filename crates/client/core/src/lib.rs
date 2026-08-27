// TEMP:
#![allow(missing_docs)]

//! # `cosplay-core-client`

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

pub mod handle;
pub(crate) use handle::*;

mod message_utils;
pub(crate) use message_utils::MessageUtils;

#[cfg(test)]
mod test_utils;
#[cfg(test)]
pub(crate) use test_utils::TestDriver;

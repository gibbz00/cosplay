// TEMP:
#![allow(missing_docs)]

//! # `cosplay-client`

mod core;
pub use core::{Client, ClientSetupError};

mod path;
pub use path::{SocketPath, SocketPathError};

mod registry_map;
pub(crate) use registry_map::{RegistryEntry, RegistryMap};

mod request_queue;
pub(crate) use request_queue::RequestQueue;

//! Types for working with the Wayland wire format.

pub mod message;
pub(crate) use message::*;

mod id;
pub(crate) use id::*;

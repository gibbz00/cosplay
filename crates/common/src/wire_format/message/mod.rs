//! Base packet frame sent over the Wayland socket.

mod core;
pub(crate) use core::MessageTemp;

mod header;
pub(crate) use header::{MessageHeader, MessageHeaderDecodeError};

mod body;
pub use body::MessageBody;

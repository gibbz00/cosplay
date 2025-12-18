//! Base packet frame sent over the Wayland socket.

mod core;
pub(crate) use core::Message;

mod header;
pub(crate) use header::{MessageHeader, MessageHeaderDecodeError};

mod r#type;
pub use r#type::{Event, MessageType, Request};

mod body;
pub use body::MessageBody;

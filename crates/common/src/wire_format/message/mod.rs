mod core;
pub(crate) use core::Message;

mod header;
pub(crate) use header::{MessageHeader, MessageHeaderDecodeError};

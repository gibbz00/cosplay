mod core;
pub(crate) use core::Message;

mod header;
pub(crate) use header::{MessageHeader, MessageHeaderDecodeError};

mod body {
    // TODO: T is either Request or Event
    pub trait MessageBody<T> {}
}
pub(crate) use body::MessageBody;

//! # `async-wayland-common`

pub mod entity;
pub(crate) use entity::{Client, Entity, Server};

pub mod id;
pub(crate) use id::*;

pub mod message;
pub(crate) use message::{Message, MessageDecoderError};

mod interface;
pub use interface::Interface;

mod socket_write;
pub use socket_write::{SocketWrite, SocketWriteError};

mod encode;
pub(crate) use encode::{FullMessage, FullMessageEncoder};

// TODO: move to separate codec utils crate?
#[doc(hidden)]
pub mod argument_codec;

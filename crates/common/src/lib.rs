//! async-wayland commons.

pub mod socket_path;

mod codec;

mod entity;
pub(crate) use entity::{Client, Server};

mod wire_format;
pub(crate) use wire_format::*;

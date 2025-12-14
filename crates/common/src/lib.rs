//! async-wayland commons.

mod codec;

pub mod socket_path;

mod entity;
pub(crate) use entity::{Client, Server};

mod wire_format;
pub(crate) use wire_format::*;

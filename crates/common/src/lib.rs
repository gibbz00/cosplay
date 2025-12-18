//! # `async-wayland-common`

mod codec;

pub mod entity;
pub(crate) use entity::{Client, Server};

pub mod wire_format;
pub(crate) use wire_format::*;

pub mod id;
pub(crate) use id::*;

pub mod message;
pub(crate) use message::{Event, Message, Request};

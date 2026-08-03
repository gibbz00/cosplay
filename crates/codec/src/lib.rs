//! # `async-wayland-codec` - Opaque and typed wire format encoding.

mod opaque;
pub use opaque::{OpaqueMessage, OpaqueMessageDecoder, OpaqueMessageEncoder};

mod arguments;

mod object_id;
pub(crate) use object_id::*;

mod fixed;
pub(crate) use fixed::Fixed;

mod entity;
pub(crate) use entity::{Client, Entity, Server};

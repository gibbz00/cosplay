//! # `async-wayland-codec` - Opaque and typed wire format encoding.

mod opaque;
pub use opaque::{OpaqueMessage, OpaqueMessageDecoder, OpaqueMessageEncoder};

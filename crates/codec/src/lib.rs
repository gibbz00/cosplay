//! # `async-wayland-codec` - Opaque and typed wire format encoding.

mod opaque;
pub(crate) use opaque::OpaqueMessage;

mod decoder;
pub use decoder::OpaqueMessageDecoder;

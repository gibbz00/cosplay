//! # `async-wayland-codec` - Opaque and typed wire format encoding.

mod opaque {
    use std::os::fd::OwnedFd;

    use bytes::BytesMut;

    pub struct OpaqueMessage {
        pub(crate) object_id: u32,
        pub(crate) op_code: u16,
        pub(crate) body: BytesMut,
        pub(crate) fds: Vec<OwnedFd>,
    }

    #[derive(Debug, PartialEq)]
    pub struct OpaqueMessageFrame {
        pub(crate) object_id: u32,
        pub(crate) op_code: u16,
        pub(crate) body: BytesMut,
    }
}
pub(crate) use opaque::{OpaqueMessage, OpaqueMessageFrame};

mod decoder;
pub use decoder::OpaqueMessageDecoder;

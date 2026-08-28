//! # `cosplay-agent` - Common utilities shared between `cosplay-client` and `cosplay-server`.

mod agent_marker;
pub use agent_marker::{Agent, Client, Server};

pub mod object_id_pool;

pub mod misc {
    use cosplay_codec::*;
    use cosplay_protocols_wayland::wl_display::WlDisplay;

    pub const WL_DISPLAY_ID: ObjectId<WlDisplay> = ObjectId::new(OpaqueObjectId::new(1));
}

pub mod geometry {
    //! Geometric primitives commonly found in protocol messages.

    #[derive(Debug, Clone)]
    #[allow(missing_docs)]
    pub struct Rectangle {
        pub x: i32,
        pub y: i32,
        pub width: i32,
        pub height: i32,
    }
}

// TODO(refactor): move to a separate crate?
pub mod pixel_format;

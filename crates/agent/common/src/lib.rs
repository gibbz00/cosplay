//! # `cosplay-agent` - Common utilities shared between `cosplay-client` and `cosplay-server`.

mod agent_marker;
pub use agent_marker::{Agent, Client, Server};

pub mod misc {
    use cosplay_codec::*;
    use cosplay_protocols_wayland::wl_display::WlDisplay;

    pub const WL_DISPLAY_ID: ObjectId<WlDisplay> = ObjectId::new(OpaqueObjectId::new(1));
}

pub mod object_id_pool;

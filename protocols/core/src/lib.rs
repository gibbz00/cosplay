//! # `async-wayland-core-protocol`

// TODO: autogenerate
mod wl_display {
    mod interface {
        pub struct WlDisplay;
    }
    pub use interface::WlDisplay;

    pub mod request {
        pub struct GetRegistry {
            pub registry: async_wayland_common::id::NewObjectId<
                async_wayland_common::entity::Client,
                crate::wl_registry::WlRegistry,
            >,
        }

        impl async_wayland_common::message::Message for GetRegistry {
            type Type = async_wayland_common::message::Request;

            const OP_CODE: u16 = 2;
        }
    }

    pub mod event {}
}

pub mod wl_registry {
    mod interface {
        pub struct WlRegistry;
    }
    pub use interface::WlRegistry;

    pub mod request {}

    pub mod event {}
}

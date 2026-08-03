//! Wayland protocol entity markers.

/// Marker trait for protocol entities.
pub trait Entity: private::Sealed {}

/// Server [Entity] Marker
pub struct Server;

impl Entity for Server {}

/// Client [Entity] Marker
pub struct Client;

impl Entity for Client {}

mod private {
    pub(super) trait Sealed {}

    impl Sealed for super::Client {}

    impl Sealed for super::Server {}
}

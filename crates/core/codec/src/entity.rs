//! Wayland protocol entity markers.

/// Marker trait for protocol entities.
#[sealed::sealed]
pub trait Entity {}

/// Server [Entity] Marker
pub struct Server;

#[sealed::sealed]
impl Entity for Server {}

/// Client [Entity] Marker
pub struct Client;

#[sealed::sealed]
impl Entity for Client {}

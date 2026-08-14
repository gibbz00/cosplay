/// Marker trait for protocol agents.
#[sealed::sealed]
pub trait Agent {}

/// Server [Agent] Marker
pub struct Server;

#[sealed::sealed]
impl Agent for Server {}

/// Client [Agent] Marker
pub struct Client;

#[sealed::sealed]
impl Agent for Client {}

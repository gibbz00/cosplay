/// Marker trait for indicating message direction.
#[sealed::sealed]
pub trait MessageType {}

/// Request Message Type
///
/// Marker for messages to be sent from a client to a server.
pub struct Request;

#[sealed::sealed]
impl MessageType for Request {}

/// Event Message Type
///
/// Marker for messages to be sent from a server to a client.
pub struct Event;

#[sealed::sealed]
impl MessageType for Event {}

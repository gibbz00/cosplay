use crate::*;

/// Marker trait for denoting message directions.
///
/// Sealed trait exclusively implemented by the [`Request`] and [`Event`] markers.
#[sealed::sealed]
pub trait Direction {}

/// Request [Direction] Marker
pub struct Request;
#[sealed::sealed]
impl Direction for Request {}

/// Event [Direction] Marker
pub struct Event;
#[sealed::sealed]
impl Direction for Event {}

/// Meant to be implemented on an [`Interface`] for converting an opaque message
/// into one of the interface's request or events.
pub trait Inbound<D: Direction> {
    /// Most often an enum with one variant for each of the interface's messages
    /// for a given direction.
    type Enum;

    /// Convert an opaque message into [`Self::Enum`].
    ///
    /// It can be assumed that the [`OpaqueMessage::object_id`] has
    /// already been verified to belong to [`Self`].
    fn from_opaque(message: OpaqueMessage) -> Result<Self::Enum, IntoInboundError>;
}

/// Returned from [`Events::from_opaque`]
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum IntoInboundError {
    #[error("Failed to deserialize {interface}::{event}: {error}")]
    Deserialize {
        interface: &'static str,
        event: &'static str,
        error: ArgumentDecodeError,
    },
    #[error("Received an event with an the unknown opcode '{}'.", _0.opcode())]
    UnknownOpcode(OpaqueMessage),
}

impl IntoInboundError {
    #[doc(hidden)]
    pub fn from_decode<M>(error: ArgumentDecodeError) -> Self
    where
        M: Message,
        M::Interface: Interface,
    {
        IntoInboundError::Deserialize { interface: <M::Interface as Interface>::NAME, event: M::NAME, error }
    }
}

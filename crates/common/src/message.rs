use crate::*;

/// Base message declaration trait.
///
/// Implementations are more often than not auto generated from an XML protocol definition.
pub trait Message {
    /// Interface for which the message belongs to.
    type Interface: Interface;

    /// Associated type for indicating whether a message is a [Request] or an [Event].
    type Type: MessageType;

    /// Message Operation Code
    ///
    /// Message op_codes are inferred from the message posisiton in XML
    /// interface specification, grouped by message type. For example, an
    /// interface with the requests A and B and the event X, will have the
    /// opcodes 1, 2, and 1 respectively.
    const OP_CODE: u16;

    /// Payload size to be included in the message header.
    ///
    /// The size returned as usize even if it is technically limited to a u16.
    /// The idea is that this should limit the amount of (possibly unchecked)
    /// u16 conversions.
    fn size(&self) -> usize;

    /// Encode message arguments into bytes.
    fn encode(&self, dst: &mut bytes::BytesMut);

    /// Return the argument decoder.
    fn decoder() -> impl MessageDecoder<Message = Self>;
}

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

/// Used for stateful decoder implementations returned from [Message::decoder]
pub trait MessageDecoder {
    /// The message to be decoded.
    type Message;

    /// Message pretty much identical to [tokio_util::codec::Decoder::decode].
    ///
    /// Kept as a separate trait to avoid a `tokio_util` dependency in the
    /// generated protocol crates.
    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Message>, MessageDecoderError>;
}

/// For decoder implementations returned from [Message::decoder]
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum MessageDecoderError {
    #[error("failed to parse object id")]
    ObjectId(#[from] ObjectIdFromRawError),
}

use crate::*;

/// Implemented for requests and events on structs containing the corresponding message arguments,
/// usually in combination with [`EncodeMessage`] and [`DecodeMessage`],
pub trait Message {
    /// Associate the message to the interface it belongs to.
    ///
    /// Provides typesafety to [`ObjectId`], among other things.
    type Interface;

    /// Message operation code.
    ///
    /// Unique within the message type (request or event) and within the interface.
    const OP_CODE: u16;
}

/// Serialize a [`Message`] into an opaque argument bag.
pub trait EncodeMessage {
    #[allow(missing_docs)]
    fn encode(self, bag: &mut ArgumentBag<'_>);
}

/// Deserialize a [`Message`] from an opaque argument bag.
pub trait DecodeMessage: Sized {
    #[allow(missing_docs)]
    fn decode(bag: &mut ArgumentBag<'_>) -> Result<Self, ArgumentDecodeError>;
}

use std::{collections::VecDeque, os::fd::OwnedFd};

use bytes::BytesMut;

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
    fn decode(bag: &mut ArgumentBag<'_>) -> Result<Self, DecodeMessageError>;
}

/// Returned from [`DecodeMessage::decode`].
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum DecodeMessageError {
    #[error("Failed to parse primitive argument: {0}")]
    ParsePrimitive(#[from] ArgumentDecodeError),
    // TODO: add "other" fallback
}

/// An opaque message convertible to and from a concrete [`Message`].
pub struct OpaqueMessage {
    pub(crate) op_code: u16,
    pub(crate) argument_buffer: BytesMut,
    pub(crate) fd_buffer: VecDeque<OwnedFd>,
}

impl OpaqueMessage {
    /// Retrieve the message op code.
    pub const fn op_code(&self) -> u16 {
        self.op_code
    }

    /// Create a new opaque message from a concrete [`Message`].
    pub fn from_concrete<M: Message + EncodeMessage>(message: M) -> Self {
        let mut bytes = BytesMut::new();
        let mut fd_buffer = VecDeque::new();

        {
            let mut bag = ArgumentBag { bytes: &mut bytes, fd_buffer: &mut fd_buffer };
            message.encode(&mut bag);
        }

        Self { op_code: M::OP_CODE, argument_buffer: bytes, fd_buffer }
    }

    /// Deserialize an opaque message into a concrete [`Message`].
    pub fn into_concrete<M: Message + DecodeMessage>(self) -> Result<M, DecodeMessageError> {
        let Self { mut argument_buffer, mut fd_buffer, .. } = self;

        let mut bag = ArgumentBag { bytes: &mut argument_buffer, fd_buffer: &mut fd_buffer };

        let message = M::decode(&mut bag)?;

        Ok(message)
    }
}

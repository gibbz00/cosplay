use std::{collections::VecDeque, os::fd::OwnedFd};

use bytes::BytesMut;

use crate::*;

/// An opaque message convertible to and from a concrete [`Message`].
#[derive(Debug)]
pub struct OpaqueMessage {
    pub(crate) object_id: OpaqueObjectId,
    pub(crate) op_code: u16,
    pub(crate) argument_buffer: BytesMut,
    pub(crate) fd_buffer: VecDeque<OwnedFd>,
}

/// Returned from [`OpaqueMessage::matches`]
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum OpapueMessageMismatchError {
    #[error("Received opcode '{0}' does not match the expected message opcode '{1}'.")]
    Opcode(u16, u16),
    #[error("Received object identifier '{0}' does not match the expected object identifier '{1}'.")]
    ObjectId(u32, u32),
}

impl OpaqueMessage {
    /// Get a the object ID stored within the message.
    pub const fn object_id(&self) -> OpaqueObjectId {
        self.object_id
    }

    /// Get a the opcode stored within the message.
    pub const fn opcode(&self) -> u16 {
        self.op_code
    }

    /// Check `self` contains the same opcode as `M: Message`.
    ///
    /// Note that this does not mean that `self` can be deserialized into `M`,
    /// given that multiple messages share the same opcode.
    pub const fn has_opcode<M: Message>(&self) -> bool {
        self.op_code == M::OP_CODE
    }

    /// Assert `self` contains a message of type M belonging to the provided `object`.
    pub fn matches<M: Message>(&self, object: ObjectId<M::Interface>) -> Result<(), OpapueMessageMismatchError> {
        if !self.has_opcode::<M>() {
            return Err(OpapueMessageMismatchError::Opcode(self.op_code, M::OP_CODE));
        }

        if object.inner != self.object_id {
            return Err(OpapueMessageMismatchError::ObjectId(object.inner(), self.object_id.inner()));
        }

        Ok(())
    }

    /// Create a new opaque message from a concrete [`Message`].
    pub fn from_concrete<M: Message + EncodeMessage>(object_id: ObjectId<M::Interface>, message: M) -> Self {
        let mut bytes = BytesMut::new();
        let mut fd_buffer = VecDeque::new();

        {
            let mut bag = ArgumentBag { bytes: &mut bytes, fd_buffer: &mut fd_buffer };
            message.encode(&mut bag);
        }

        Self {
            object_id: object_id.as_opaque(),
            op_code: M::OP_CODE,
            argument_buffer: bytes,
            fd_buffer,
        }
    }

    /// Deserialize an opaque message into a concrete [`Message`].
    ///
    /// This function does not verify that the contained opcode matches that of `Message`, nor that
    /// the contained object_id points to the corresponding [`Message::Interface`]. Users are
    /// encouraged to call first [`OpaqueMessage::matches`] for that part.
    pub fn into_concrete<M: DecodeMessage>(self) -> Result<M, ArgumentDecodeError> {
        let Self { mut argument_buffer, mut fd_buffer, .. } = self;

        let mut bag = ArgumentBag { bytes: &mut argument_buffer, fd_buffer: &mut fd_buffer };

        let message = M::decode(&mut bag)?;

        Ok(message)
    }
}

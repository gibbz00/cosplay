// TEMP:
#![allow(missing_docs)]

use std::{collections::VecDeque, os::fd::OwnedFd};

use bytes::BytesMut;

use crate::*;

pub trait Message {
    const OP_CODE: u16;
}

pub trait EncodeMessage {
    fn encode(self, bag: &mut ArgumentBag<'_>);
}

pub trait DecodeMessage: Sized {
    fn decode(bag: &mut ArgumentBag<'_>) -> Result<Self, DecodeMessageError>;
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeMessageError {
    #[error("Failed to parse primitive argument: {0}")]
    ParsePrimitive(#[from] ArgumentDecodeError),
    // TODO: add "other" fallback
}

pub struct OpaqueMessage {
    pub(crate) op_code: u16,
    pub(crate) argument_buffer: BytesMut,
    pub(crate) fd_buffer: VecDeque<OwnedFd>,
}

impl OpaqueMessage {
    pub fn from_concrete<M: Message + EncodeMessage>(message: M) -> Self {
        let mut bytes = BytesMut::new();
        let mut fd_buffer = VecDeque::new();

        {
            let mut bag = ArgumentBag { bytes: &mut bytes, fd_buffer: &mut fd_buffer };
            message.encode(&mut bag);
        }

        Self { op_code: M::OP_CODE, argument_buffer: bytes, fd_buffer }
    }

    pub fn into_concrete<M: Message + DecodeMessage>(self) -> Result<M, DecodeMessageError> {
        let Self { mut argument_buffer, mut fd_buffer, .. } = self;

        let mut bag = ArgumentBag { bytes: &mut argument_buffer, fd_buffer: &mut fd_buffer };

        let message = M::decode(&mut bag)?;

        Ok(message)
    }
}

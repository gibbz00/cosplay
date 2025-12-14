use std::marker::PhantomData;

use crate::*;

// mod decode
pub struct MessageDecoder<E> {
    entity_marker: PhantomData<E>,
}

#[derive(Debug, thiserror::Error)]
pub enum MessageDecoderError {
    #[error("encountered unknown IO error")]
    Io(#[from] std::io::Error),
}

impl<E> tokio_util::codec::Decoder for MessageDecoder<E> {
    type Error = MessageDecoderError;
    type Item = Message<E>;

    fn decode(&mut self, src: &mut tokio_util::bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }
}

// mod encode
pub struct MessageEncoder<E> {
    entity_marker: PhantomData<E>,
}

#[derive(Debug, thiserror::Error)]
pub enum MessageEncoderError {
    #[error("encountered unknown IO error")]
    Io(#[from] std::io::Error),
}

impl<E> tokio_util::codec::Encoder<Message<E>> for MessageEncoder<E> {
    type Error = MessageEncoderError;

    fn encode(&mut self, message: Message<E>, dst: &mut tokio_util::bytes::BytesMut) -> Result<(), Self::Error> {
        todo!()
    }
}

use std::marker::PhantomData;

use bytes::BytesMut;

use crate::*;

// mod decode
pub struct MessageDecoder<E> {
    entity_marker: PhantomData<E>,
}

#[derive(Debug, thiserror::Error)]
pub enum MessageDecoderError<E: ObjectIdBounds> {
    #[error("failed to decode header")]
    Header(#[from] MessageHeaderDecodeError<E>),
    #[error("encountered unknown IO error")]
    Io(#[from] std::io::Error),
}

impl<E: ObjectIdBounds> tokio_util::codec::Decoder for MessageDecoder<E> {
    type Error = MessageDecoderError<E>;
    type Item = Message<E>;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let _header = MessageHeader::<E>::decode(src)?;
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

impl<E: ObjectIdBounds> tokio_util::codec::Encoder<Message<E>> for MessageEncoder<E> {
    type Error = MessageEncoderError;

    fn encode(&mut self, message: Message<E>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        message.header.encode(dst);
        todo!()
    }
}

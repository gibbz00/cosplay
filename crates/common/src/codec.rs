use std::marker::PhantomData;

use bytes::BytesMut;

use crate::*;

// mod decode
pub struct MessageDecoder<T> {
    message_type_marker: PhantomData<T>,
}

#[derive(Debug, thiserror::Error)]
pub enum MessageDecoderError {
    #[error("failed to decode header")]
    Header(#[from] MessageHeaderDecodeError),
    #[error("encountered unknown IO error")]
    Io(#[from] std::io::Error),
}

impl<T> tokio_util::codec::Decoder for MessageDecoder<T> {
    type Error = MessageDecoderError;
    type Item = MessageTemp<T>;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let _header = MessageHeader::decode(src)?;
        todo!()
    }
}

// mod encode
pub struct MessageEncoder<T> {
    message_type_marker: PhantomData<T>,
}

#[derive(Debug, thiserror::Error)]
pub enum MessageEncoderError {
    #[error("encountered unknown IO error")]
    Io(#[from] std::io::Error),
}

impl<T> tokio_util::codec::Encoder<MessageTemp<T>> for MessageEncoder<T> {
    type Error = MessageEncoderError;

    fn encode(&mut self, message: MessageTemp<T>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        message.header.encode(dst);
        todo!()
    }
}

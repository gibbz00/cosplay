use crate::*;

// mod decode
pub struct MessageDecoder {}

#[derive(Debug, thiserror::Error)]
pub enum MessageDecoderError {
    #[error("encountered unknown IO error")]
    Io(#[from] std::io::Error),
}

impl tokio_util::codec::Decoder for MessageDecoder {
    type Error = MessageDecoderError;
    type Item = Message;

    fn decode(&mut self, src: &mut tokio_util::bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }
}

// mod encode
pub struct MessageEncoder {}

#[derive(Debug, thiserror::Error)]
pub enum MessageEncoderError {
    #[error("encountered unknown IO error")]
    Io(#[from] std::io::Error),
}

impl tokio_util::codec::Encoder<Message> for MessageEncoder {
    type Error = MessageEncoderError;

    fn encode(&mut self, message: Message, dst: &mut tokio_util::bytes::BytesMut) -> Result<(), Self::Error> {
        todo!()
    }
}

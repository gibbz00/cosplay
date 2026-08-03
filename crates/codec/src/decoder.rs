use bytes::{Buf, BufMut, BytesMut};

use crate::*;

const HEADER_LENGTH: usize = 8;

#[derive(Default)]
pub struct OpaqueMessageEncoder {
    __priv: (),
}

impl tokio_util::codec::Encoder<&OpaqueMessage> for OpaqueMessageEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, message: &OpaqueMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let OpaqueMessage { object_id, op_code, body } = message;

        dst.put_u32_ne(*object_id);

        let total_length = body.len() + HEADER_LENGTH;

        dst.put_u32_ne(((total_length as u32) << 16) + *op_code as u32);

        dst.extend_from_slice(body);

        Ok(())
    }
}

#[derive(Default)]
pub struct OpaqueMessageDecoder {
    stage: FrameDecoderStage,
}

#[derive(Debug, thiserror::Error)]
pub enum OpaqueMessageDecodeError {
    #[error("Message size {0} is too short, expected at least {HEADER_LENGTH}.")]
    InvalidSize(usize),
    #[error("Error received from underlying I/O stream: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Default)]
enum FrameDecoderStage {
    #[default]
    WantsHeader,
    WantsBody {
        object_id: u32,
        body_size: usize,
        op_code: u16,
    },
}

impl tokio_util::codec::Decoder for OpaqueMessageDecoder {
    type Error = OpaqueMessageDecodeError;
    type Item = OpaqueMessage;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let stage = std::mem::take(&mut self.stage);

        match stage {
            FrameDecoderStage::WantsHeader => {
                if src.len() < HEADER_LENGTH {
                    return Ok(None);
                }

                let object_id = src.get_u32_ne();

                let word = src.get_u32_ne();

                let message_size = (word >> 16) as usize;

                let body_size = message_size
                    .checked_sub(HEADER_LENGTH)
                    .ok_or(OpaqueMessageDecodeError::InvalidSize(message_size))?;

                let op_code = ((word << 16) >> 16) as u16;

                self.stage = FrameDecoderStage::WantsBody { object_id, body_size, op_code };

                self.decode(src)
            }
            FrameDecoderStage::WantsBody { object_id, body_size, op_code } => {
                if src.len() < body_size {
                    self.stage = FrameDecoderStage::WantsBody { object_id, body_size, op_code };
                    return Ok(None);
                }

                let body = src.split_to(body_size);

                Ok(Some(OpaqueMessage { object_id, op_code, body }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use tokio_util::codec::{Decoder, Encoder};

    use super::*;

    #[test]
    fn encode_decode_single_frame() {
        let mut buffer = BytesMut::new();

        let message = OpaqueMessage { object_id: 123, op_code: 456, body: BytesMut::from_iter(b"hello") };

        OpaqueMessageEncoder::default().encode(&message, &mut buffer).unwrap();

        let actual = OpaqueMessageDecoder::default().decode(&mut buffer).unwrap().unwrap();

        assert_eq!(message, actual);
    }

    #[test]
    fn decode_await_read() {
        let mut buffer = BytesMut::new();

        let mut decoder = OpaqueMessageDecoder::default();

        assert!(decoder.decode(&mut buffer).unwrap().is_none());
    }

    #[test]
    fn decode_size_error() {
        let mut buffer = BytesMut::new();

        // object id
        buffer.put_u32_ne(1);

        let total_length = 6;
        assert!(total_length < HEADER_LENGTH);

        buffer.put_u32_ne((total_length as u32) << 16);

        let actual = OpaqueMessageDecoder::default().decode(&mut buffer).transpose().unwrap();

        assert_matches!(actual, Err(OpaqueMessageDecodeError::InvalidSize(6)));
    }

    #[test]
    fn encode_decode_multiple_frames() {
        let mut buffer = BytesMut::new();

        let message_0 = OpaqueMessage { object_id: 123, op_code: 456, body: BytesMut::from_iter(b"hello") };
        let message_1 = OpaqueMessage { object_id: 789, op_code: 111, body: BytesMut::from_iter(b"codec") };

        let mut encoder = OpaqueMessageEncoder::default();
        encoder.encode(&message_0, &mut buffer).unwrap();
        encoder.encode(&message_1, &mut buffer).unwrap();

        let mut decoder = OpaqueMessageDecoder::default();

        let actual_0 = decoder.decode(&mut buffer).unwrap().unwrap();
        assert_eq!(message_0, actual_0);

        let actual_1 = decoder.decode(&mut buffer).unwrap().unwrap();
        assert_eq!(message_1, actual_1);
    }
}

use bytes::{Buf, BufMut, BytesMut};

const HEADER_LENGTH: usize = 8;

#[derive(Debug, PartialEq)]
pub struct OpaqueFrame {
    pub(crate) object_id: u32,
    pub(crate) op_code: u16,
    pub(crate) argument_buffer: BytesMut,
}

#[derive(Default)]
pub struct OpaqueFrameEncoder {
    __priv: (),
}

impl tokio_util::codec::Encoder<&OpaqueFrame> for OpaqueFrameEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, frame: &OpaqueFrame, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let OpaqueFrame { object_id, op_code, argument_buffer } = frame;

        dst.put_u32_ne(*object_id);

        let total_length = argument_buffer.len() + HEADER_LENGTH;

        dst.put_u32_ne(((total_length as u32) << 16) + *op_code as u32);

        dst.extend_from_slice(argument_buffer);

        Ok(())
    }
}

#[derive(Default)]
pub struct OpaqueFrameDecoder {
    stage: DecoderStage,
}

#[derive(Debug, thiserror::Error)]
pub enum OpaqueFrameDecodeError {
    #[error("Message size {0} is too short, expected at least {HEADER_LENGTH}.")]
    InvalidSize(usize),
    #[error("Error received from underlying I/O stream: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Default)]
enum DecoderStage {
    #[default]
    WantsHeader,
    WantsArguments {
        object_id: u32,
        arguments_size: usize,
        op_code: u16,
    },
}

impl tokio_util::codec::Decoder for OpaqueFrameDecoder {
    type Error = OpaqueFrameDecodeError;
    type Item = OpaqueFrame;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let stage = std::mem::take(&mut self.stage);

        match stage {
            DecoderStage::WantsHeader => {
                if src.len() < HEADER_LENGTH {
                    return Ok(None);
                }

                let object_id = src.get_u32_ne();

                let word = src.get_u32_ne();

                let message_size = (word >> 16) as usize;

                let arguments_size = message_size
                    .checked_sub(HEADER_LENGTH)
                    .ok_or(OpaqueFrameDecodeError::InvalidSize(message_size))?;

                let op_code = ((word << 16) >> 16) as u16;

                self.stage = DecoderStage::WantsArguments { object_id, arguments_size, op_code };

                self.decode(src)
            }
            DecoderStage::WantsArguments { object_id, arguments_size, op_code } => {
                if src.len() < arguments_size {
                    self.stage = DecoderStage::WantsArguments { object_id, arguments_size, op_code };
                    return Ok(None);
                }

                let argument_buffer = src.split_to(arguments_size);

                Ok(Some(OpaqueFrame { object_id, op_code, argument_buffer }))
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

        let frame = OpaqueFrame {
            object_id: 123,
            op_code: 456,
            argument_buffer: BytesMut::from_iter(b"hello"),
        };

        OpaqueFrameEncoder::default().encode(&frame, &mut buffer).unwrap();

        let actual = OpaqueFrameDecoder::default().decode(&mut buffer).unwrap().unwrap();

        assert_eq!(frame, actual);
    }

    #[test]
    fn decode_await_read() {
        let mut buffer = BytesMut::new();

        let mut decoder = OpaqueFrameDecoder::default();

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

        let actual = OpaqueFrameDecoder::default().decode(&mut buffer).transpose().unwrap();

        assert_matches!(actual, Err(OpaqueFrameDecodeError::InvalidSize(6)));
    }

    #[test]
    fn encode_decode_multiple_frames() {
        let mut buffer = BytesMut::new();

        let frame_0 = OpaqueFrame {
            object_id: 123,
            op_code: 456,
            argument_buffer: BytesMut::from_iter(b"hello"),
        };
        let frame_1 = OpaqueFrame {
            object_id: 789,
            op_code: 111,
            argument_buffer: BytesMut::from_iter(b"codec"),
        };

        let mut encoder = OpaqueFrameEncoder::default();
        encoder.encode(&frame_0, &mut buffer).unwrap();
        encoder.encode(&frame_1, &mut buffer).unwrap();

        let mut decoder = OpaqueFrameDecoder::default();

        let actual_0 = decoder.decode(&mut buffer).unwrap().unwrap();
        assert_eq!(frame_0, actual_0);

        let actual_1 = decoder.decode(&mut buffer).unwrap().unwrap();
        assert_eq!(frame_1, actual_1);
    }
}

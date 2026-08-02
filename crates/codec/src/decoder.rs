use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Buf;
use tokio::io::AsyncRead;

use crate::*;

const HEADER_LENGTH: usize = 8;

pub struct OpaqueMessageDecoder<R> {
    inner: tokio_util::codec::FramedRead<R, OpaqueMessageFrameDecoder>,
}

#[derive(Debug, thiserror::Error)]
pub enum OpaqueDecodeError {
    #[error("Message size {0} is too short, expected at least {HEADER_LENGTH}.")]
    InvalidSize(usize),
    #[error("Error received from underlying I/O stream: {0}")]
    Io(#[from] std::io::Error),
}

impl<R: AsyncRead + async_wayland_ancillary::AncillaryRead + Unpin> futures_core::Stream for OpaqueMessageDecoder<R> {
    type Item = Result<OpaqueMessage, OpaqueDecodeError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let next = std::task::ready!(std::pin::pin!(&mut self.inner).poll_next(cx));

        let mapped_next = next.map(|result| {
            // No need to drain on error. (Shouldn't be any fds from recv if so.)
            result.map(|OpaqueMessageFrame { object_id, op_code, body: data }| OpaqueMessage {
                object_id,
                op_code,
                body: data,
                fds: self.inner.get_mut().drain(),
            })
        });

        Poll::Ready(mapped_next)
    }
}

#[derive(Default)]
struct OpaqueMessageFrameDecoder {
    stage: FrameDecoderStage,
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

impl tokio_util::codec::Decoder for OpaqueMessageFrameDecoder {
    type Error = OpaqueDecodeError;
    type Item = OpaqueMessageFrame;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
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
                    .ok_or(OpaqueDecodeError::InvalidSize(message_size))?;

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

                Ok(Some(OpaqueMessageFrame { object_id, op_code, body }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use bytes::{BufMut, BytesMut};
    use tokio_util::codec::Decoder;

    use super::*;

    /// Populates dst and returns the expected result from decoding dst.
    fn mock(dst: &mut BytesMut, object_id: u32, op_code: u16, body: &[u8]) -> OpaqueMessageFrame {
        dst.put_u32_ne(object_id);

        let total_length = body.len() + HEADER_LENGTH;

        dst.put_u32_ne(((total_length as u32) << 16) + op_code as u32);

        dst.extend_from_slice(body);

        OpaqueMessageFrame { object_id, op_code, body: BytesMut::from_iter(body) }
    }

    #[test]
    fn decode_single_frame() {
        let mut buffer = BytesMut::new();

        let expected = mock(&mut buffer, 123, 456, b"hello");

        let actual = OpaqueMessageFrameDecoder::default().decode(&mut buffer).unwrap().unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn decode_await_read() {
        let mut buffer = BytesMut::new();

        let mut decoder = OpaqueMessageFrameDecoder::default();

        assert!(decoder.decode(&mut buffer).unwrap().is_none());
    }

    #[test]
    fn size_error() {
        let mut buffer = BytesMut::new();

        // object id
        buffer.put_u32_ne(1);

        let total_length = 6;
        assert!(total_length < HEADER_LENGTH);

        buffer.put_u32_ne((total_length as u32) << 16);

        let actual = OpaqueMessageFrameDecoder::default().decode(&mut buffer).transpose().unwrap();

        assert_matches!(actual, Err(OpaqueDecodeError::InvalidSize(6)));
    }

    #[test]
    fn decode_multiple_frames() {
        let mut buffer = BytesMut::new();

        let expected_0 = mock(&mut buffer, 123, 456, b"hello");
        let expected_1 = mock(&mut buffer, 789, 111, b"codec");

        let mut decoder = OpaqueMessageFrameDecoder::default();

        let actual_0 = decoder.decode(&mut buffer).unwrap().unwrap();
        assert_eq!(expected_0, actual_0);

        let actual_1 = decoder.decode(&mut buffer).unwrap().unwrap();
        assert_eq!(expected_1, actual_1);
    }
}

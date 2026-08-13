use async_wayland_ancillary::AncillaryBuffer;
use futures_util::{SinkExt, StreamExt};

use crate::*;

pub struct WaylandMessageSink<W> {
    frame_writer: tokio_util::codec::FramedWrite<W, OpaqueFrameEncoder>,
}

impl<W: tokio::io::AsyncWrite + AncillaryBuffer + Unpin> WaylandMessageSink<W> {
    pub async fn send_concrete<M: Message + EncodeMessage>(&mut self, object_id: OpaqueObjectId, message: M) -> std::io::Result<()> {
        let opaque_message = OpaqueMessage::from_concrete(message);
        self.send_opaque(object_id, opaque_message).await
    }

    pub async fn send_opaque(&mut self, object_id: OpaqueObjectId, opaque_message: OpaqueMessage) -> std::io::Result<()> {
        let OpaqueMessage { op_code, argument_buffer, fd_buffer } = opaque_message;

        let frame = OpaqueFrame { object_id: object_id.0, op_code, argument_buffer };

        // Overwrite makes sure any previous file descriptors
        // aren't included the new message to send.
        *self.frame_writer.get_mut().file_descriptors() = fd_buffer;

        // Flushing (over just feed) guarantees that file descriptors
        // forwarded in previously fed messages, which would make them
        // be sent earlier than they should.
        self.frame_writer.send(&frame).await
    }
}

pub struct WaylandMessageStream<R> {
    frame_reader: tokio_util::codec::FramedRead<R, OpaqueFrameDecoder>,
}

#[derive(Debug, thiserror::Error)]
pub enum WaylandMessageStreamError {
    #[error("Failed to decode bytes into an opaque frame: {0}")]
    Opaque(#[from] OpaqueFrameDecodeError),
    #[error("Failed to message from frame: {0}")]
    Argument(#[from] DecodeMessageError),
    #[error("Received opcode '{0}' does not match expected message opcate '{1}'.")]
    OpcodeMismatch(u16, u16),
}

impl<R: tokio::io::AsyncRead + AncillaryBuffer + Unpin> WaylandMessageStream<R> {
    pub async fn receive_concrete<M: Message + DecodeMessage>(&mut self) -> Option<Result<(u32, M), WaylandMessageStreamError>> {
        self.receive_opaque().await.map(|result| {
            let (object_id, opaque_message) = result?;

            if opaque_message.op_code != M::OP_CODE {
                return Err(WaylandMessageStreamError::OpcodeMismatch(opaque_message.op_code, M::OP_CODE));
            }

            let message = opaque_message.into_concrete()?;

            Ok((object_id, message))
        })
    }

    pub async fn receive_opaque(&mut self) -> Option<Result<(u32, OpaqueMessage), WaylandMessageStreamError>> {
        self.frame_reader.next().await.map(|result| {
            let frame = result?;

            let OpaqueFrame { object_id, op_code, argument_buffer } = frame;

            // Drain entire file descriptor buffer to ensure they
            // aren't mistaken for belonging to the next frame.
            let fd_buffer = std::mem::take(self.frame_reader.get_mut().file_descriptors());

            let message = OpaqueMessage { op_code, argument_buffer, fd_buffer };

            Ok((object_id, message))
        })
    }
}

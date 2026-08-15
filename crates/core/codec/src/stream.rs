use std::{collections::VecDeque, os::fd::OwnedFd};

use cosplay_ancillary::AncillaryBuffer;
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{FramedRead, FramedWrite};

use crate::*;

/// Stream for receiving serializing, encoding and sending wayland messages.
pub struct WaylandMessageSink<W> {
    frame_writer: FramedWrite<W, OpaqueFrameEncoder>,
}

impl<W> WaylandMessageSink<W> {
    /// Create a new message sink from an underlying I/O sink.
    pub fn new(sink: W) -> Self {
        Self { frame_writer: FramedWrite::new(sink, OpaqueFrameEncoder::default()) }
    }

    /// Consume self and return the underlying I/O sink.
    pub fn into_inner(self) -> W {
        self.frame_writer.into_inner()
    }
}

impl<W: AsyncWrite + AncillaryBuffer + Unpin> WaylandMessageSink<W> {
    /// Send (and flush) a concrete wayland message.
    pub async fn send_concrete<M: Message + EncodeMessage>(
        &mut self,
        object_id: ObjectId<M::Interface>,
        message: M,
    ) -> std::io::Result<()> {
        let opaque_message = OpaqueMessage::from_concrete(object_id, message);
        self.send_opaque(opaque_message).await
    }

    /// Send (and flush) an opaque wayland message.
    pub async fn send_opaque(&mut self, opaque_message: OpaqueMessage) -> std::io::Result<()> {
        let OpaqueMessage { object_id, op_code, argument_buffer, fd_buffer } = opaque_message;

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

/// Stream for receiving, decoding, and desiralizing wayland messages.
pub struct WaylandMessageStream<R> {
    frame_reader: FramedRead<R, OpaqueFrameDecoder>,
}

impl<R> WaylandMessageStream<R> {
    /// Create a new message stream from an underlying I/O stream.
    pub fn new(stream: R) -> Self {
        Self { frame_reader: FramedRead::new(stream, OpaqueFrameDecoder::default()) }
    }

    /// Consume self and return the underlying I/O stream.
    pub fn into_inner(self) -> R {
        self.frame_reader.into_inner()
    }
}

impl<R: AsyncRead + AncillaryBuffer + Unpin> WaylandMessageStream<R> {
    /// Receive an opaque wayland message.
    ///
    /// Most users will then want to call [`OpaqueMessage::matches`] and
    /// [`OpaqueMessage::into_concrete`].
    pub async fn receive_opaque(&mut self) -> Option<Result<OpaqueMessage, OpaqueFrameDecodeError>> {
        self.frame_reader.next().await.map(|result| {
            let frame = result?;

            let OpaqueFrame { object_id, op_code, argument_buffer } = frame;

            // Drain entire file descriptor buffer to ensure they
            // aren't mistaken for belonging to the next frame.
            let fd_buffer = std::mem::take(self.frame_reader.get_mut().file_descriptors());

            let object_id = OpaqueObjectId(object_id);
            let message = OpaqueMessage { object_id, op_code, argument_buffer, fd_buffer };

            Ok(message)
        })
    }
}

/// In-memory buffer backed by Vec usable by both [`WaylandMessageSink`] and
/// [`WaylandMessageStream`].
///
/// Mostly useful for testing message passing without needing to use actual Unix sockets.
#[derive(Default)]
pub struct WaylandMemoryBuffer {
    bytes: Vec<u8>,
    fd_buffer: VecDeque<OwnedFd>,
}

impl AsyncRead for WaylandMemoryBuffer {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let bytes = std::mem::take(&mut self.bytes);
        std::pin::pin!(bytes.as_slice()).poll_read(cx, buf)
    }
}

impl AsyncWrite for WaylandMemoryBuffer {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::pin!(&mut self.bytes).poll_write(cx, buf)
    }

    fn poll_flush(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
        std::pin::pin!(&mut self.bytes).poll_flush(cx)
    }

    fn poll_shutdown(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
        std::pin::pin!(&mut self.bytes).poll_shutdown(cx)
    }
}

impl AncillaryBuffer for WaylandMemoryBuffer {
    fn file_descriptors(&mut self) -> &mut VecDeque<OwnedFd> {
        &mut self.fd_buffer
    }
}

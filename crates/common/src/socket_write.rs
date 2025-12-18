use futures_util::SinkExt;
use tokio::io::AsyncWrite;
use tokio_util::codec::FramedWrite;

use crate::*;

/// Error returned from [SocketWrite::send].
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum SocketWriteError {
    #[error("unknown IO error occurred")]
    Io(#[from] std::io::Error),
}

/// Encode and write messages into a writer.
// IMPROVEMENT: implement Sink?
pub struct SocketWrite<W> {
    inner: FramedWrite<W, FullMessageEncoder>,
}

impl<W: AsyncWrite + Unpin> SocketWrite<W> {
    /// Prepare and flush a message over the wayland socket.
    pub async fn send<M: Message>(
        &mut self,
        id: ObjectId<<M::Interface as Interface>::Owner>,
        message: &M,
    ) -> Result<(), SocketWriteError> {
        let full_message = FullMessage { id, message };
        self.inner.send(full_message).await
    }
}

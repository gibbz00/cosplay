use cosplay_codec::{OpaqueMessage, WaylandMessageSink};
use cosplay_net::WaylandUnixStreamWriteHalf;

pub type RequestQueueTx = tokio::sync::mpsc::UnboundedSender<OpaqueMessage>;
pub(crate) type RequestQueueRx = tokio::sync::mpsc::UnboundedReceiver<OpaqueMessage>;

pub struct RequestQueue {
    writer: WaylandMessageSink<WaylandUnixStreamWriteHalf>,
    rx: RequestQueueRx,
}

impl RequestQueue {
    pub(crate) fn new(writer: WaylandMessageSink<WaylandUnixStreamWriteHalf>) -> (RequestQueueTx, Self) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let this = Self { writer, rx };

        (tx, this)
    }

    pub async fn run(mut self) {
        loop {
            match self.rx.recv().await {
                Some(request) => {
                    if let Err(error) = self.writer.send_opaque(request).await {
                        tracing::error!(%error, "Failed write request to the underlying I/O sink.");
                        break;
                    }
                }
                None => {
                    tracing::debug!("Request channel closed.");
                    break;
                }
            }
        }

        tracing::debug!("Aborting request queue event loop.");
    }
}

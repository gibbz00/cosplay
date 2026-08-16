use cosplay_codec::{OpaqueMessage, WaylandMessageSink};
use cosplay_net::WaylandUnixStreamWriteHalf;

pub struct RequestQueue {
    writer: WaylandMessageSink<WaylandUnixStreamWriteHalf>,
    rx: tokio::sync::mpsc::UnboundedReceiver<OpaqueMessage>,
}

impl RequestQueue {
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

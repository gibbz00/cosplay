use std::sync::Arc;

use cosplay_agent::{misc::WL_DISPLAY_ID, object_id_pool::ObjectIdRetriever};
use cosplay_codec::{NewObjectId, OpaqueMessage};
use cosplay_protocols_wayland::{wl_callback, wl_display};

use crate::*;

pub type SyncDoneTx = tokio::sync::oneshot::Sender<wl_callback::Done>;

// TODO: creatable directly from any object handle?
/// Pseudo-object handle to internally managed `wl_display` object.
///
/// Special in that it does not have a dedicated destructor, nor a creation
/// hierarchy originating from registered global.
pub struct SyncHandle {
    id_retriever: Arc<ObjectIdRetriever<cosplay_agent::Client>>,
    mediator_tx: MediatorTx,
    request_queue_tx: RequestQueueTx,
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("General request error: {0}")]
    Common(#[from] RequestError),
}

impl SyncHandle {
    pub(crate) fn new(
        id_retriever: Arc<ObjectIdRetriever<cosplay_agent::Client>>,
        mediator_tx: MediatorTx,
        request_queue_tx: RequestQueueTx,
    ) -> Self {
        Self { id_retriever, mediator_tx, request_queue_tx }
    }

    pub async fn sync(&self) -> Result<wl_callback::Done, SyncError> {
        let object_id = self.id_retriever.try_next().ok_or(RequestError::NoIdAvailable)?;

        let (tx, rx) = tokio::sync::oneshot::channel();

        self.mediator_tx
            .send(MediatorMessage::Sync(object_id, tx))
            .map_err(|_| RequestError::MediatorDown)?;

        self.request_queue_tx
            .send(OpaqueMessage::from_concrete(
                WL_DISPLAY_ID,
                wl_display::Sync { callback: NewObjectId::new(object_id) },
            ))
            .map_err(|_| RequestError::RequestQueueDown)?;

        rx.await.map_err(|_| RequestError::MediatorDown).map_err(Into::into)
    }
}

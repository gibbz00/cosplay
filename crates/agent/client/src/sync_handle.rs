use std::sync::Arc;

use cosplay_agent::object_id_pool::ObjectIdRetriever;
use cosplay_protocols_wayland::wl_callback;

use crate::*;

pub type SyncDoneTx = tokio::sync::oneshot::Sender<wl_callback::Done>;
pub type SyncDoneRx = tokio::sync::oneshot::Receiver<wl_callback::Done>;

/// Pseudo-object handle to internally handled `wl_display` object.
///
/// Special in that it does not have a dedicated destructor, nor a creation
/// hierarchy which leads to a registered global.
pub struct SyncHandle {
    id_retriever: Arc<ObjectIdRetriever<cosplay_agent::Client>>,
    mediator_tx: MediatorTx,
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("General request error: {0}")]
    Common(#[from] RequestError),
}

impl SyncHandle {
    pub(crate) fn new(id_retriever: Arc<ObjectIdRetriever<cosplay_agent::Client>>, mediator_tx: MediatorTx) -> Self {
        Self { id_retriever, mediator_tx }
    }

    pub async fn sync(&self) -> Result<wl_callback::Done, SyncError> {
        let object_id = self.id_retriever.try_next().ok_or(RequestError::NoIdAvailable)?;

        let (tx, rx) = tokio::sync::oneshot::channel();

        self.mediator_tx
            .send(MediatorMessage::Sync(object_id, tx))
            .map_err(|_| RequestError::MediatorDown)?;

        rx.await.map_err(|_| RequestError::MediatorDown).map_err(Into::into)
    }
}

use std::sync::Arc;

use cosplay_agent::{misc::WL_DISPLAY_ID, object_id_pool::ObjectIdRetriever};
use cosplay_protocols_wayland::wl_display::{self, WlDisplay};

use crate::*;

// TODO: Creatable directly from any object handle?
/// Pseudo-object handle to internally managed `wl_display` object.
///
/// Special in that it does not have a dedicated destructor, nor a creation
/// hierarchy originating from registered global.
pub struct SyncHandle {
    request_handle: RequestHandle<WlDisplay>,
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Failed to send sync request: {0}")]
    Request(#[from] RequestError),
    #[error("Failed to receive response: {0}")]
    Response(#[from] ObjectEventsError),
}

impl SyncHandle {
    pub(crate) fn new(
        id_retriever: Arc<ObjectIdRetriever<cosplay_agent::Client>>,
        mediator_tx: MediatorTx,
        request_queue_tx: RequestQueueTx,
    ) -> Self {
        let request_handle = RequestHandle {
            id: WL_DISPLAY_ID,
            resolved_version: 1,
            id_retriever,
            mediator_tx,
            request_queue_tx,
        };

        Self { request_handle }
    }

    pub async fn sync(&self) -> Result<(), SyncError> {
        self.request_handle
            .init_callback(|id| wl_display::Sync { callback: id })?
            .await
            // "The callback_data passed in the callback
            // is undefined and should be ignored."
            .map(|_data| ())
            .map_err(Into::into)
    }
}

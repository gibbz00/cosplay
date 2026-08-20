use std::sync::Arc;

use cosplay_agent::object_id_pool::ObjectIdRetriever;
use cosplay_codec::{ObjectId, OpaqueMessage};

use crate::*;

// # Lifecycle management
//
// ## Creation
//
// `new_id`s are created using the `ObjectIdRetriever`.
//
// A sync/register message must be sent to over the mediator channel **before** send the
// corresponding request to the request channel. This prevents the receival of a read
// events before the corresponding object id has been registered in the mediator's event
// forwarding map.
//
// ## Removal
//
// Most objects should have a drop implementation which sends the object's destructor request. This
// ensures that the server sends a `wl_display::delete_id` event, which in turn allows the event
// mediator to return the ID to the object ID pool.
pub struct ObjectHandle<I> {
    pub(crate) id: ObjectId<I>,
    pub(crate) resolved_version: u32,
    pub(crate) id_retriever: Arc<ObjectIdRetriever<cosplay_agent::Client>>,
    pub(crate) mediator_tx: MediatorTx,
    pub(crate) request_queue_tx: RequestQueueTx,
    pub(crate) inbound_rx: ObjectHandleRx,
}

pub enum ObjectHandleMessage {
    Event(OpaqueMessage),
    Error { code: u32, message: String },
}

pub type ObjectHandleTx = tokio::sync::mpsc::UnboundedSender<ObjectHandleMessage>;
pub type ObjectHandleRx = tokio::sync::mpsc::UnboundedReceiver<ObjectHandleMessage>;

impl<I> ObjectHandle<I> {
    pub async fn recv(&mut self) -> Option<ObjectHandleMessage> {
        self.inbound_rx.recv().await
    }
}

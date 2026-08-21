use std::sync::Arc;

use cosplay_agent::object_id_pool::ObjectIdRetriever;
use cosplay_codec::{ObjectId, OpaqueMessage};
use tokio::sync::mpsc::error::TryRecvError;

use crate::*;

// # Lifecycle management
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
    pub(crate) fn events_iter(&mut self) -> ObjectSyncEventsIter<'_> {
        ObjectSyncEventsIter { inbound_rx: &mut self.inbound_rx }
    }

    pub(crate) fn subobject_with_version<J>(&self, resolved_version: u32) -> Result<ObjectHandle<J>, RequestError> {
        let new_id = self.id_retriever.try_next().ok_or(RequestError::NoIdAvailable)?;

        tracing::trace!(parent_id = self.id.inner(), %new_id, "Creating a new subject.");

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // NOTE: Send register object to mediator **before** sending the request
        // over the request queue in order to avoid having a data race in which
        // the mediator drops unmapped events.

        // IMPROVEMENT: Return id to pool if send to mediator or request queue fails?
        // However, there isn't much to do anyways if either channel is closed...

        self.mediator_tx
            .send(MediatorMessage::Register(new_id, tx))
            .map_err(|_| RequestError::MediatorDown)?;

        let object_handle = ObjectHandle {
            id: ObjectId::new(new_id),
            inbound_rx: rx,
            resolved_version,
            id_retriever: self.id_retriever.clone(),
            mediator_tx: self.mediator_tx.clone(),
            request_queue_tx: self.request_queue_tx.clone(),
        };

        Ok(object_handle)
    }
}

pub struct ObjectSyncEventsIter<'a> {
    inbound_rx: &'a mut ObjectHandleRx,
}

#[derive(Debug, thiserror::Error)]
pub enum ObjectSyncEventsError {
    #[error("Mediator down. Unable to receive any new events.")]
    MediatorDown,
}

impl Iterator for ObjectSyncEventsIter<'_> {
    type Item = Result<ObjectHandleMessage, ObjectSyncEventsError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inbound_rx.try_recv() {
            Ok(event) => Some(Ok(event)),
            Err(error) => match error {
                TryRecvError::Empty => None,
                TryRecvError::Disconnected => Some(Err(ObjectSyncEventsError::MediatorDown)),
            },
        }
    }
}

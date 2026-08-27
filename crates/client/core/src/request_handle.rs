use std::{marker::PhantomData, sync::Arc};

use cosplay_agent::object_id_pool::ObjectIdRetriever;
use cosplay_codec::{EncodeMessage, Message, NewObjectId, ObjectId, OpaqueMessage};

use crate::*;

#[impl_tools::autoimpl(Debug)]
pub struct RequestHandle<I> {
    pub(crate) id: ObjectId<I>,
    pub(crate) resolved_version: u32,
    pub(crate) id_retriever: Arc<ObjectIdRetriever<cosplay_agent::Client>>,
    pub(crate) mediator_tx: MediatorTx,
    pub(crate) request_queue_tx: RequestQueueTx,
}

/// Common request errors.
#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("Failed to allocate a new object ID. No ID available.")]
    NoIdAvailable,
    #[error("Failed to communicate with event mediator.")]
    MediatorDown,
    #[error("Request channel dropped.")]
    RequestQueueDown,
}

impl<I> RequestHandle<I> {
    pub fn enqueue<M: Message<Interface = I> + EncodeMessage>(&self, message: M) -> Result<(), RequestError> {
        self.request_queue_tx
            .send(OpaqueMessage::from_concrete(self.id, message))
            // IMPROVEMENT: return message object back?
            .map_err(|_| RequestError::RequestQueueDown)
    }

    pub fn init_subobject<M: Message<Interface = I> + EncodeMessage, J>(
        &self,
        create_request: impl FnOnce(NewObjectId<J>) -> M,
    ) -> Result<ObjectHandle<J>, RequestError> {
        self.init_subobject_with_version(self.resolved_version, create_request)
    }

    pub(crate) fn init_subobject_with_version<M: Message<Interface = I> + EncodeMessage, J>(
        &self,
        resolved_version: u32,
        create_request: impl FnOnce(NewObjectId<J>) -> M,
    ) -> Result<ObjectHandle<J>, RequestError> {
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

        self.enqueue(create_request(NewObjectId::new(new_id)))?;

        let request_handle = RequestHandle {
            id: ObjectId::new(new_id),
            resolved_version,
            id_retriever: self.id_retriever.clone(),
            mediator_tx: self.mediator_tx.clone(),
            request_queue_tx: self.request_queue_tx.clone(),
        };

        let event_handle = EventHandle { inbound_rx: rx, interface_marker: PhantomData };

        let object_handle = ObjectHandle { request: request_handle, event: event_handle };

        Ok(object_handle)
    }
}

use std::{marker::PhantomData, sync::Arc};

use cosplay_agent::object_id_pool::ObjectIdRetriever;
use cosplay_codec::{EncodeMessage, Event, Inbound, IntoInboundError, Message, ObjectId, OpaqueMessage};
use tokio::sync::mpsc::error::TryRecvError;

use crate::*;

// # Lifecycle management
//
// ## Removal
//
// Most objects should have a drop implementation which sends the object's destructor request. This
// ensures that the server sends a `wl_display::delete_id` event, which in turn allows the event
// mediator to return the ID to the object ID pool.
#[impl_tools::autoimpl(Debug)]
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
    pub fn events_iter(&mut self) -> ObjectEventsIter<'_, I> {
        ObjectEventsIter { inbound_rx: &mut self.inbound_rx, interface_marker: PhantomData }
    }

    pub fn queue_request<M: Message<Interface = I> + EncodeMessage>(&self, message: M) -> Result<(), RequestError> {
        self.request_queue_tx
            .send(OpaqueMessage::from_concrete(self.id, message))
            // IMPROVEMENT: return message object back?
            .map_err(|_| RequestError::RequestQueueDown)
    }

    pub(crate) fn init_subobject<J>(&self) -> Result<ObjectHandle<J>, RequestError> {
        self.init_subobject_with_version(self.resolved_version)
    }

    pub(crate) fn init_subobject_with_version<J>(&self, resolved_version: u32) -> Result<ObjectHandle<J>, RequestError> {
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

pub struct ObjectEventsIter<'a, I> {
    inbound_rx: &'a mut ObjectHandleRx,
    interface_marker: PhantomData<I>,
}

pub enum ObjectEvent<E> {
    Event(E),
    Error { code: u32, message: String },
}

#[derive(Debug, thiserror::Error)]
pub enum ObjectEventsError {
    #[error("Mediator down. Unable to receive any new events.")]
    MediatorDown,
    #[error("Failed to convert opaque message into inbound event: {0}")]
    Convert(#[from] IntoInboundError),
}

impl<I> Iterator for ObjectEventsIter<'_, I>
where
    I: Inbound<Event>,
{
    type Item = Result<ObjectEvent<I::Enum>, ObjectEventsError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.inbound_rx.try_recv() {
            Ok(event) => Some(match event {
                ObjectHandleMessage::Event(message) => I::from_opaque(message).map(ObjectEvent::Event).map_err(Into::into),
                ObjectHandleMessage::Error { code, message } => Ok(ObjectEvent::Error { code, message }),
            }),
            Err(error) => match error {
                TryRecvError::Empty => None,
                TryRecvError::Disconnected => Some(Err(ObjectEventsError::MediatorDown)),
            },
        }
    }
}

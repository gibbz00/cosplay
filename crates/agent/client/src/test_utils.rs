use std::sync::Arc;

use cosplay_agent::object_id_pool::ObjectIdReturner;
use cosplay_codec::{DecodeMessage, EncodeMessage, Message, ObjectId, OpaqueMessage, OpaqueObjectId};

use crate::*;

pub(crate) struct TestDriver {
    pub(crate) id_returner: ObjectIdReturner,
    pub(crate) mediator_rx: MediatorRx,
    pub(crate) request_queue_rx: RequestQueueRx,
    pub(crate) inbound_tx: ObjectHandleTx,
}

impl TestDriver {
    pub(crate) fn new<I>() -> (Self, ObjectHandle<I>) {
        let (id_retriever, id_returner) = cosplay_agent::object_id_pool::create();

        let (mediator_tx, mediator_rx) = tokio::sync::mpsc::unbounded_channel();

        let (request_queue_tx, request_queue_rx) = tokio::sync::mpsc::unbounded_channel();

        let (inbound_tx, inbound_rx) = tokio::sync::mpsc::unbounded_channel();

        let root_handle = ObjectHandle {
            id: ObjectId::new(OpaqueObjectId::new(1)),
            resolved_version: 1,
            id_retriever: Arc::new(id_retriever),
            mediator_tx,
            request_queue_tx,
            inbound_rx,
        };

        let this = Self { id_returner, mediator_rx, request_queue_rx, inbound_tx };

        (this, root_handle)
    }

    pub(crate) fn send_event<M: Message + EncodeMessage>(&self, object_id: ObjectId<M::Interface>, message: M) {
        let opaque_message = OpaqueMessage::from_concrete(object_id, message);

        let channel_message = ObjectHandleMessage::Event(opaque_message);

        self.inbound_tx.send(channel_message).expect("Object handle channel closed.");
    }

    pub(crate) fn assert_queued_destructor_on_drop<M: Message + DecodeMessage, T>(&mut self, handle_id: ObjectId<M::Interface>, handle: T) {
        assert!(self.request_queue_rx.is_empty());

        drop(handle);

        self.assert_outbound_request::<M>(handle_id);
    }

    pub(crate) fn assert_outbound_request<M: Message + DecodeMessage>(&mut self, handle_id: ObjectId<M::Interface>) -> M {
        let opaque_request = self.request_queue_rx.try_recv().unwrap();

        assert!(opaque_request.matches::<M>(handle_id).is_ok());

        opaque_request.into_concrete().unwrap()
    }
}

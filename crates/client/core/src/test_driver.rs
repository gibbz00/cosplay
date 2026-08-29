use std::{marker::PhantomData, sync::Arc};

use cosplay_agent::object_id_pool::ObjectIdReturner;
use cosplay_codec::{DecodeMessage, EncodeMessage, Message, ObjectId, OpaqueMessage, OpaqueObjectId};

use crate::{event_mediator::MediatorRx, request_queue::RequestQueueRx, *};

pub struct TestDriver {
    pub id_returner: ObjectIdReturner,
    pub mediator_rx: MediatorRx,
    pub request_queue_rx: RequestQueueRx,
    pub inbound_tx: ObjectHandleTx,
}

impl TestDriver {
    pub fn new_global<H: GlobalHandle>() -> (Self, H) {
        let (this, raw) = Self::new_raw::<H::Interface>();
        (this, H::from_raw(raw))
    }

    pub fn new_raw<I>() -> (Self, ObjectHandle<I>) {
        let (id_retriever, id_returner) = cosplay_agent::object_id_pool::create();

        let (mediator_tx, mediator_rx) = tokio::sync::mpsc::unbounded_channel();

        let (request_queue_tx, request_queue_rx) = tokio::sync::mpsc::unbounded_channel();

        let (inbound_tx, inbound_rx) = tokio::sync::mpsc::unbounded_channel();

        let request_handle = RequestHandle {
            id: ObjectId::new(OpaqueObjectId::new(1)),
            resolved_version: 1,
            id_retriever: Arc::new(id_retriever),
            mediator_tx,
            request_queue_tx,
        };

        let event_handle = EventHandle { inbound_rx, interface_marker: PhantomData };

        let root_handle = ObjectHandle { request: request_handle, event: event_handle };

        let this = Self { id_returner, mediator_rx, request_queue_rx, inbound_tx };

        (this, root_handle)
    }

    pub fn send_event<M: Message + EncodeMessage>(&self, object_id: ObjectId<M::Interface>, message: M) {
        let opaque_message = OpaqueMessage::from_concrete(object_id, message);

        let channel_message = ObjectHandleMessage::Event(opaque_message);

        self.inbound_tx.send(channel_message).expect("Object handle channel closed.");
    }

    pub fn assert_queued_destructor_on_drop<M: Message + DecodeMessage, T>(&mut self, handle_id: ObjectId<M::Interface>, handle: T) {
        assert!(self.request_queue_rx.is_empty());

        drop(handle);

        self.assert_outbound_request::<M>(handle_id);
    }

    pub fn assert_outbound_request<M: Message + DecodeMessage>(&mut self, handle_id: ObjectId<M::Interface>) -> M {
        let opaque_request = self.request_queue_rx.try_recv().unwrap();

        assert!(opaque_request.matches::<M>(handle_id).is_ok());

        opaque_request.into_concrete().unwrap()
    }
}

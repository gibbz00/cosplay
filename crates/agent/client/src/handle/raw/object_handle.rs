use cosplay_codec::OpaqueMessage;

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
    pub request: RequestHandle<I>,
    pub event: EventHandle<I>,
}

pub enum ObjectHandleMessage {
    Event(OpaqueMessage),
    Error { code: u32, message: String },
}

pub type ObjectHandleTx = tokio::sync::mpsc::UnboundedSender<ObjectHandleMessage>;
pub type ObjectHandleRx = tokio::sync::mpsc::UnboundedReceiver<ObjectHandleMessage>;

impl<I> ObjectHandle<I> {
    pub fn into_split(self) -> (RequestHandle<I>, EventHandle<I>) {
        let Self { request, event } = self;
        (request, event)
    }
}

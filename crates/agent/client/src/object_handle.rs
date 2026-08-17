use cosplay_codec::{ObjectId, OpaqueMessage};

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
    pub(crate) inbound_rx: ObjectHandleRx,
    pub(crate) resolved_version: u32,
}

pub enum ObjectHandleMessage {
    Event(OpaqueMessage),
    Error { code: u32, message: String },
}

pub type ObjectHandleTx = tokio::sync::mpsc::UnboundedSender<ObjectHandleMessage>;
pub type ObjectHandleRx = tokio::sync::mpsc::UnboundedReceiver<ObjectHandleMessage>;

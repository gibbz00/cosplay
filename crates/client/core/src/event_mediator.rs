use std::collections::HashMap;

use cosplay_agent::{misc::WL_DISPLAY_ID, object_id_pool::ObjectIdReturner};
use cosplay_codec::{DecodeMessage, Interface, Message, OpaqueMessage, OpaqueObjectId, WaylandMessageStream};
use cosplay_net::WaylandUnixStreamReadHalf;
use cosplay_protocols_wayland::wl_display;

use crate::*;

/// Respective requests should be sent before
pub enum MediatorMessage {
    Register(OpaqueObjectId, ObjectHandleTx),
}

pub type MediatorTx = tokio::sync::mpsc::UnboundedSender<MediatorMessage>;
pub type MediatorRx = tokio::sync::mpsc::UnboundedReceiver<MediatorMessage>;

pub struct EventMediator {
    reader: WaylandMessageStream<WaylandUnixStreamReadHalf>,
    mediator_rx: MediatorRx,
    object_map: HashMap<OpaqueObjectId, ObjectHandleTx>,
    id_returner: ObjectIdReturner,
}

impl EventMediator {
    pub(crate) fn new(reader: WaylandMessageStream<WaylandUnixStreamReadHalf>, id_returner: ObjectIdReturner) -> (MediatorTx, Self) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let this = Self { reader, mediator_rx: rx, object_map: Default::default(), id_returner };

        (tx, this)
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                // NB: Biased to handle mediator events first, see `ObjectHandle` for more.
                biased;

                mediator_event = self.mediator_rx.recv() => {
                    match mediator_event {
                        Some(message) => self.handle_mediator_message(message),
                        None => {
                            tracing::debug!("Mediator channel closed.");
                            break;
                        }
                    }
                },
                read_event = self.reader.receive_opaque() => {
                    match read_event {
                        Some(decode_result) => match decode_result {
                            Ok(event) => self.handle_read_message(event),
                            Err(error) => {
                                // Consider framing errors as fatal.
                                tracing::error!(%error, "Failed to read or frame bytes from socket read half.");
                                break;
                            }
                        },
                        None => {
                            tracing::debug!("Socket read half closed.");
                            break;
                        }
                    }
                },
            }
        }

        tracing::debug!("Aborting event mediator event loop.");
    }

    fn handle_mediator_message(&mut self, message: MediatorMessage) {
        match message {
            MediatorMessage::Register(object_id, object_handle_tx) => {
                tracing::trace!(%object_id, "Registering new object handle.");

                if self.object_map.insert(object_id, object_handle_tx).is_some() {
                    tracing::error!(%object_id, "Object map insertion wrote over previous object transmitter. Object identifiers should be unique.");
                }
            }
        }
    }

    fn handle_read_message(&mut self, event: OpaqueMessage) {
        let object_id = event.object_id();
        let opcode = event.opcode();

        tracing::trace!(%object_id, %opcode, "Handling inbound event.");

        // Check if `wl_display::delete_id`.

        if event.matches::<wl_display::DeleteId>(WL_DISPLAY_ID).is_ok() {
            if let Some(wl_display::DeleteId { id }) = into_concrete_logged(event) {
                // We don't want to introduce an ObjectId(0) into the object
                // pool. One could wish that the protocol could just have used he
                // correct argument type...
                if id == 0 {
                    tracing::warn!("Server sent '0' for the object ID, ignoring.");
                    return;
                }

                let object_id = OpaqueObjectId::new(id);

                // IMPROVEMENT(log): If no ID was removed. Problem is that the
                // ID could have been part part of a wl_callback, whose entry
                // entry is removed in conjunction with the oneshot retrieval.
                self.object_map.remove(&object_id);

                self.return_id_logged(object_id);
            }

            return;
        }

        // Check if `wl_display::error`.

        if event.matches::<wl_display::Error>(WL_DISPLAY_ID).is_ok() {
            if let Some(wl_display::Error { object_id, code, message }) = into_concrete_logged(event) {
                match object_id == WL_DISPLAY_ID.as_opaque() {
                    true => {
                        tracing::warn!(code, message, "wl_display received a wl_display::error from server.");
                    }
                    false => {
                        tracing::debug!(%object_id, code, message, "Received a wl_display::error event from the server. Attempting forward to corresponding object handle.");

                        self.forward_to_handle(&object_id, ObjectHandleMessage::Error { code, message });
                    }
                }
            }

            return;
        }

        self.forward_to_handle(&object_id, ObjectHandleMessage::Event(event))
    }

    fn forward_to_handle(&self, object_id: &OpaqueObjectId, message: ObjectHandleMessage) {
        match self.object_map.get(object_id) {
            Some(object_handle_tx) => {
                tracing::trace!(%object_id, "Forwarding message to object handle.");

                if object_handle_tx.send(message).is_err() {
                    // Don't attempt to return to object pool here, event may have
                    // "raced" with a destructor request. Doing so would lead to
                    // duplicate IDs in object pool.
                    tracing::debug!(%object_id, "Unable to forward message; handle channel closed.");
                }
            }
            None => {
                tracing::warn!(%object_id, "No object handle found for forwarding message to.");
            }
        }
    }

    fn return_id_logged(&self, object_id: OpaqueObjectId) {
        if self.id_returner.return_id(object_id).is_err() {
            tracing::debug!(%object_id, "Unable to return object id; all retrievers dropped.")
        }
    }
}

fn into_concrete_logged<M: Message + DecodeMessage>(event: OpaqueMessage) -> Option<M>
where
    M::Interface: Interface,
{
    let object_id = event.object_id();
    let opcode = event.opcode();

    event
        .into_concrete::<M>()
        .inspect_err(|error| {
            tracing::error!(
                %object_id,
                %opcode,
                interface = <M::Interface as Interface>::NAME,
                event = M::NAME,
                %error,
                "Failed to event into its concrete counterpart."
            );
        })
        .ok()
}

use cosplay_codec::{Interface, Message, OpaqueMessage, OpaqueNewObjectId};
use cosplay_protocols_wayland::wl_registry::{self, WlRegistry};
use tokio::sync::mpsc::error::TryRecvError;

use crate::*;

pub struct RegistryHandle {
    object_handle: ObjectHandle<WlRegistry>,
    registry_map: RegistryMap,
}

#[derive(Debug, thiserror::Error)]
pub enum BindError {
    #[error("Global not present in registry.")]
    NotRegistered,
    #[error("General request error: {0}")]
    Common(#[from] RequestError),
}

impl RegistryHandle {
    pub(crate) fn new(object_handle: ObjectHandle<WlRegistry>) -> Self {
        Self { object_handle, registry_map: Default::default() }
    }

    pub async fn bind_raw<I: Interface>(&mut self) -> Result<ObjectHandle<I>, BindError> {
        // Make sure that map is up to date.
        loop {
            match self.object_handle.inbound_rx.try_recv() {
                Ok(message) => {
                    match message {
                        ObjectHandleMessage::Event(event) => {
                            if event.opcode() == wl_registry::Global::OP_CODE {
                                if let Some(global) = MessageUtils::into_concrete_logged::<wl_registry::Global>(event) {
                                    self.registry_map.register(global);
                                }
                            } else if event.opcode() == wl_registry::GlobalRemove::OP_CODE {
                                if let Some(global_remove) = MessageUtils::into_concrete_logged::<wl_registry::GlobalRemove>(event) {
                                    self.registry_map.remove(global_remove);
                                }
                            } else {
                                // TODO(log): unknown event
                            }
                        }
                        ObjectHandleMessage::Error { code, message } => {
                            // Assuming that this should not occur so long as
                            // the registry implementation is correct? Hard to
                            // tell from the wayland.xml
                            tracing::error!(code, message, "Registry received an unhandled error.")
                        }
                    }
                }
                Err(recv_error) => match recv_error {
                    TryRecvError::Empty => break,
                    TryRecvError::Disconnected => return Err(BindError::Common(RequestError::MediatorDown)),
                },
            }
        }

        let RegistryEntry { number_name, server_version } = self.registry_map.get::<I>().ok_or(BindError::NotRegistered)?;

        let resolved_version = std::cmp::min(*server_version, I::VERSION);

        let subobject = self.object_handle.subobject_with_version(resolved_version)?;

        let interface_name = I::NAME.to_string();

        tracing::debug!(number_name, interface_name, new_id = subobject.id.inner(), "Sending bind request.");

        self.object_handle
            .request_queue_tx
            .send(OpaqueMessage::from_concrete(
                self.object_handle.id,
                wl_registry::Bind {
                    name: *number_name,
                    id: OpaqueNewObjectId {
                        // Seems a bit superfluous given the name argument. Regardless,
                        // most compositors will error if this does not match with the
                        // name sent over `wl_registry::global`.
                        interface_name,
                        interface_version: resolved_version,
                        object_id: subobject.id.as_opaque(),
                    },
                },
            ))
            .map_err(|_| RequestError::RequestQueueDown)?;

        Ok(subobject)
    }
}

// TODO:
// impl Drop {
//     fn drop() {
//         // If wl_fixes exists in registry map, send wl_fixes.destroy_registry()
//     }
// }

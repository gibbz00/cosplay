use std::{path::Path, sync::Arc};

use cosplay_agent::misc::WL_DISPLAY_ID;
use cosplay_codec::{NewObjectId, ObjectId, WaylandMessageSink, WaylandMessageStream};
use cosplay_net::{WaylandUnixStreamReadHalf, WaylandUnixStreamWriteHalf};
use cosplay_protocols_wayland::wl_display::GetRegistry;

use crate::*;

type SocketWrite = WaylandMessageSink<WaylandUnixStreamWriteHalf>;
type SocketRead = WaylandMessageStream<WaylandUnixStreamReadHalf>;

pub struct ClientSetup {}

#[derive(Debug, thiserror::Error)]
pub enum ClientSetupError {
    #[error("Failed to resolve wayland socket path: {0}")]
    PathResolver(#[from] SocketPathError),
    #[error("Failed to connect to wayland unix socket: {0}")]
    Connect(std::io::Error),
    #[error("Failed to send first client request: {0}")]
    SendFirst(std::io::Error),
}

impl ClientSetup {
    pub async fn setup(path: Option<&Path>) -> Result<(RequestQueue, EventMediator, RegistryHandle, SyncHandle), ClientSetupError> {
        let (reader, mut writer) = Self::init_socket_halves(path)?;

        let (id_retriever, id_returner) = cosplay_agent::object_id_pool::create();

        // Skip first since it is for `wl_display`.
        id_retriever.try_next();

        let registry_id = id_retriever
            .try_next()
            .expect("Exhausted all object IDs from a newly created ID pool");

        // IMPROVEMENT: check for errors from server before passing the control over to the actors?
        writer
            .send_concrete(WL_DISPLAY_ID, GetRegistry { registry: NewObjectId::new(registry_id) })
            .await
            .map_err(ClientSetupError::SendFirst)?;

        let (mediator_tx, mediator) = EventMediator::new(reader, id_returner);

        let (inbound_tx, inbound_rx) = tokio::sync::mpsc::unbounded_channel();

        mediator_tx
            .send(MediatorMessage::Register(registry_id, inbound_tx))
            .expect("Event mediator channel closed at startup.");

        let object_handle = ObjectHandle {
            id: ObjectId::new(registry_id),
            inbound_rx,
            // Assume one for now. Interface isn't frozen, but at the same time,
            // there isn't any way for the server to advertise its version?
            resolved_version: 1,
        };

        let (request_queue_tx, request_queue) = RequestQueue::new(writer);

        let id_retriever = Arc::new(id_retriever);

        let registry_handle = RegistryHandle::new(object_handle, id_retriever.clone(), mediator_tx.clone(), request_queue_tx.clone());

        let sync_handle = SyncHandle::new(id_retriever, mediator_tx, request_queue_tx);

        Ok((request_queue, mediator, registry_handle, sync_handle))
    }

    fn init_socket_halves(path: Option<&Path>) -> Result<(SocketRead, SocketWrite), ClientSetupError> {
        let socket_path = SocketPath::resolve(path)?;

        let (socket_read, socket_write) = cosplay_net::WaylandUnixStream::connect(socket_path.as_ref())
            .map_err(ClientSetupError::Connect)?
            .into_split();

        let reader = WaylandMessageStream::new(socket_read);

        let writer = WaylandMessageSink::new(socket_write);

        Ok((reader, writer))
    }
}

use std::path::Path;

use cosplay_agent::{NewObjectIdFactory, misc::WL_DISPLAY_ID};
use cosplay_codec::{
    ArgumentDecodeError, Interface, ObjectId, OpaqueFrameDecodeError, OpaqueNewObjectId, WaylandMessageSink, WaylandMessageStream,
};
use cosplay_net::{WaylandUnixStreamReadHalf, WaylandUnixStreamWriteHalf};
use cosplay_protocols_wayland::{
    wl_callback,
    wl_display::{self, GetRegistry, Sync},
    wl_registry::{self, Global, WlRegistry},
};

use crate::*;

type SocketWrite = WaylandMessageSink<WaylandUnixStreamWriteHalf>;
type SocketRead = WaylandMessageStream<WaylandUnixStreamReadHalf>;

pub struct Client {
    pub id_factory: NewObjectIdFactory<cosplay_agent::Client>,
    registry_id: ObjectId<WlRegistry>,
    registry_map: RegistryMap,
    pub writer: SocketWrite,
    pub reader: SocketRead,
}

#[derive(Debug, thiserror::Error)]
pub enum ClientSetupError {
    #[error("Failed to resolve wayland socket path: {0}")]
    PathResolver(#[from] SocketPathError),
    #[error("Failed to connect to wayland unix socket: {0}")]
    Connect(std::io::Error),
    #[error("Failed to send client request: {0}")]
    Send(std::io::Error),
    #[error("Failed decode message frame: {0}")]
    DecodeFrame(#[from] OpaqueFrameDecodeError),
    #[error("Failed deserialize opaque message arguments: {0}")]
    DecodeArguments(#[from] ArgumentDecodeError),
    #[error("Failed to get register from wl_display; error code '{0}', message: '{1}'")]
    GetRegister(u32, String),
}

impl Client {
    pub async fn setup(path: Option<&Path>) -> Result<Self, ClientSetupError> {
        let (reader, writer) = Self::init_socket_halves(path)?;
        Self::init_registry(reader, writer).await
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

    async fn init_registry(mut reader: SocketRead, mut writer: SocketWrite) -> Result<Self, ClientSetupError> {
        let mut id_factory = NewObjectIdFactory::new();
        // Skip first since it is already assigned to wl_display.
        id_factory.next::<()>();

        let mut registry_map = RegistryMap::default();

        let registry_id = id_factory.next();
        writer
            .send_concrete(WL_DISPLAY_ID, GetRegistry { registry: registry_id })
            .await
            .map_err(ClientSetupError::Send)?;

        let done_id = id_factory.next();
        writer
            .send_concrete(WL_DISPLAY_ID, Sync { callback: done_id })
            .await
            .map_err(ClientSetupError::Send)?;

        let registry_id = registry_id.promote();
        let done_id = done_id.promote();

        while let Some(inbound_frame) = reader.receive_opaque().await {
            let msg = inbound_frame?;

            if msg.matches::<wl_display::Error>(WL_DISPLAY_ID).is_ok() {
                // IMPROVEMENT: double check object_id?
                let wl_display::Error { code, message, .. } = msg.into_concrete::<wl_display::Error>()?;
                return Err(ClientSetupError::GetRegister(code, message));
            }

            if msg.matches::<wl_registry::Global>(registry_id).is_ok() {
                let global = msg.into_concrete::<wl_registry::Global>()?;
                registry_map.register(global);
                continue;
            }

            if msg.matches::<wl_registry::GlobalRemove>(registry_id).is_ok() {
                let global_remove = msg.into_concrete::<wl_registry::GlobalRemove>()?;
                registry_map.remove(global_remove);
                continue;
            }

            // TODO: log unknown message

            if msg.matches::<wl_callback::Done>(done_id).is_ok() {
                break;
            }
        }

        Ok(Self { id_factory, registry_id, registry_map, writer, reader })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientBindError {
    #[error("Global not present in registry.")]
    NotRegistered,
    #[error("Failed to send bind request: {0}.")]
    Send(#[from] std::io::Error),
}

impl Client {
    pub async fn bind<I: Interface>(&mut self) -> Result<(ObjectId<I>, u32), ClientBindError> {
        let RegistryEntry { number_name, server_version } = self.registry_map.get::<I>().ok_or(ClientBindError::NotRegistered)?;

        let new_id = self.id_factory.next::<I>().promote();

        let actual_version = std::cmp::min(*server_version, I::VERSION);

        self.writer
            .send_concrete(
                self.registry_id,
                wl_registry::Bind {
                    name: *number_name,
                    id: OpaqueNewObjectId {
                        // Seems a bit superfluous given the name argument. Regardless,
                        // most compositors will error if this does not match with the
                        // name sent over `wl_registry::global`.
                        interface_name: I::NAME.to_string(),
                        interface_version: actual_version,
                        object_id: new_id.as_opaque(),
                    },
                },
            )
            .await?;

        Ok((new_id, actual_version))
    }
}

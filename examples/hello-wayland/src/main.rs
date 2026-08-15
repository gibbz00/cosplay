//! `cosplay` counterpart of <https://github.com/emersion/hello-wayland>

use cosplay_agent::{Client, NewObjectIdFactory};
use cosplay_codec::*;
use cosplay_net::{WaylandUnixStreamReadHalf, WaylandUnixStreamWriteHalf};
use cosplay_protocols_wayland::{
    wl_callback,
    wl_compositor::WlCompositor,
    wl_display::{self, *},
    wl_registry::{self, *},
    wl_shm::WlShm,
};
use cosplay_protocols_xdg_shell::xdg_wm_base::XdgWmBase;

const WL_DISPLAY_ID: ObjectId<WlDisplay> = ObjectId::new(OpaqueObjectId::new(1));

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (mut r, mut w) = setup_handles()?;

    let mut id_factory = NewObjectIdFactory::<Client>::new();

    // Skip first since it is for wl_display
    id_factory.next::<WlRegistry>();
    let registry_id = id_factory.next();

    w.send_concrete(WL_DISPLAY_ID, GetRegistry { registry: registry_id }).await?;

    let registry_id = registry_id.promote();

    let callback_id = id_factory.next();
    w.send_concrete(WL_DISPLAY_ID, Sync { callback: callback_id }).await?;

    let mut wl_shm_global = None;
    let mut wl_compositor_global = None;
    let mut xdg_base_global = None;

    while let Some(inbound_frame) = r.receive_opaque().await {
        let (id, msg) = inbound_frame?;

        let id = id.inner();
        let opcode = msg.op_code();

        tracing::info!(id, opcode, "Received a new message from the server.");

        if id == WL_DISPLAY_ID.inner() && opcode == wl_display::Error::OP_CODE {
            let error_message = msg.into_concrete::<wl_display::Error>()?;
            tracing::warn!(?error_message, "Server returned global error.");
            continue;
        }

        if id == registry_id.inner() && opcode == wl_registry::Global::OP_CODE {
            let global_message = msg.into_concrete::<wl_registry::Global>()?;
            tracing::info!(?global_message, "Received global from server.");

            match global_message.interface.as_str() {
                WlShm::NAME => wl_shm_global = Some(global_message),
                WlCompositor::NAME => wl_compositor_global = Some(global_message),
                XdgWmBase::NAME => xdg_base_global = Some(global_message),
                _ => {}
            }

            continue;
        }

        if id == callback_id.inner() {
            let done_message = msg.into_concrete::<wl_callback::Done>()?;
            tracing::info!(?done_message, "Received sync callback from server.");
            break;
        }
    }

    let (Some(shm_msg), Some(compositor_msg), Some(xdg_base_msg)) = (wl_shm_global, wl_compositor_global, xdg_base_global) else {
        todo!("error")
    };

    let wl_shm_id = bind::<WlShm>(&mut w, &mut id_factory, registry_id, shm_msg).await?;

    let wl_compositor_id = bind::<WlCompositor>(&mut w, &mut id_factory, registry_id, compositor_msg).await?;

    let xdg_base_id = bind::<XdgWmBase>(&mut w, &mut id_factory, registry_id, xdg_base_msg).await?;

    while let Some(inbound_frame) = r.receive_opaque().await {
        let (id, msg) = inbound_frame?;

        let id = id.inner();
        let opcode = msg.op_code();

        tracing::info!(id, opcode, "Received a new message from the server.");

        if id == WL_DISPLAY_ID.inner() && opcode == wl_display::Error::OP_CODE {
            let error_message = msg.into_concrete::<wl_display::Error>()?;
            tracing::warn!(?error_message, "Server returned global error.");
            continue;
        }
    }

    Ok(())
}

fn setup_handles() -> anyhow::Result<(
    WaylandMessageStream<WaylandUnixStreamReadHalf>,
    WaylandMessageSink<WaylandUnixStreamWriteHalf>,
)> {
    let socket_path = cosplay_net::SocketPath::resolve(None)?;

    let (socket_read, socket_write) = cosplay_net::WaylandUnixStream::connect(socket_path.as_ref())?.into_split();

    let stream = WaylandMessageStream::new(socket_read);

    let sink = WaylandMessageSink::new(socket_write);

    Ok((stream, sink))
}

async fn bind<I: Interface>(
    w: &mut WaylandMessageSink<WaylandUnixStreamWriteHalf>,
    id_factory: &mut NewObjectIdFactory<Client>,
    registry: ObjectId<WlRegistry>,
    global: Global,
) -> anyhow::Result<(ObjectId<I>, u32)> {
    let id = id_factory.next::<I>().promote();

    let Global { name, interface, version } = global;

    tracing::info!(id = id.inner(), name = %interface, "Creating registry bind.");

    let actual_version = std::cmp::min(version, I::VERSION);

    w.send_concrete(
        registry,
        Bind {
            name,
            id: OpaqueNewObjectId {
                // Seems a bit superfluous given the name argument. Regardless,
                // most compositors will error if this does not match with the
                // name sent over `wl_registry::global`.
                interface_name: interface,
                // Most compositors require this to be less than or equal to the
                // version they advertised in `wl_registry::global`.
                interface_version: actual_version,
                object_id: id.as_opaque(),
            },
        },
    )
    .await?;

    Ok((id, actual_version))
}

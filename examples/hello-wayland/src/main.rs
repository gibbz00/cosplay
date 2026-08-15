//! `cosplay` counterpart of <https://github.com/emersion/hello-wayland>

use cosplay_agent::{Client, Server};
use cosplay_codec::{Message, ObjectId, OpaqueObjectId, WaylandMessageSink, WaylandMessageStream};
use cosplay_net::{WaylandUnixStreamReadHalf, WaylandUnixStreamWriteHalf};
use cosplay_protocols_wayland::{
    wl_callback,
    wl_display::{self, WlDisplay},
    wl_registry,
};

const WL_DISPLAY_ID: ObjectId<WlDisplay> = ObjectId::new(OpaqueObjectId::new(1));

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (mut r, mut w) = setup_handles()?;

    let mut id_factory = cosplay_agent::NewObjectIdFactory::<Client>::new();

    // Skip first since it is for wl_display
    id_factory.next::<cosplay_protocols_wayland::wl_registry::WlRegistry>();
    let registry_id = id_factory.next();

    w.send_concrete(WL_DISPLAY_ID, wl_display::GetRegistry { registry: registry_id })
        .await?;

    let callback_id = id_factory.next();
    w.send_concrete(WL_DISPLAY_ID, wl_display::Sync { callback: callback_id }).await?;

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
            continue;
        }

        if id == callback_id.inner() {
            let done_message = msg.into_concrete::<wl_callback::Done>()?;
            tracing::info!(?done_message, "Received sync callback from server.");
            break;
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

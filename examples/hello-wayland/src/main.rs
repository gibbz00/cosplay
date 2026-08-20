//! `cosplay` counterpart of <https://github.com/emersion/hello-wayland>

use cosplay_client::ObjectHandleMessage;
use cosplay_codec::{Enumeration, Message};
use cosplay_protocols_wayland::{
    wl_compositor::WlCompositor,
    wl_seat::WlSeat,
    wl_shm::{self, WlShm},
};
use cosplay_protocols_xdg_shell::xdg_wm_base::XdgWmBase;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let (request_queue, event_mediator, mut registry_handle, sync_handle) = cosplay_client::ClientSetup::setup(None).await?;

    tokio::spawn(request_queue.run());
    tokio::spawn(event_mediator.run());

    // Sync roundtrip to ensure globals advertisement has finished.
    sync_handle.sync().await?;

    let mut wl_shm_handle = registry_handle.bind::<WlShm>().await?;

    let wl_seat_handle = registry_handle.bind::<WlSeat>().await?;

    let wl_compositor_handle = registry_handle.bind::<WlCompositor>().await?;

    let xdg_base_handle = registry_handle.bind::<XdgWmBase>().await?;

    while let Some(shm_event) = wl_shm_handle.recv().await {
        match shm_event {
            ObjectHandleMessage::Event(opaque_message) => {
                let opcode = opaque_message.opcode();

                match opcode == wl_shm::Format::OP_CODE {
                    true => {
                        let format = opaque_message.into_concrete::<wl_shm::Format>().unwrap().format.inner();
                        tracing::info!(?format, "New server pixel format support announced.")
                    }
                    false => {
                        tracing::warn!(opcode, "Unknown event forwarded to wl_shm.")
                    }
                }
            }
            ObjectHandleMessage::Error { code, message } => {
                let code = wl_shm::Error::from_repr(code);
                tracing::error!(?code, message, "Error event forwarded to wl_shm.");
            }
        }
    }

    Ok(())
}

//! `cosplay` counterpart of <https://github.com/emersion/hello-wayland>

use cosplay_protocols_wayland::{wl_compositor::WlCompositor, wl_shm::WlShm};
use cosplay_protocols_xdg_shell::xdg_wm_base::XdgWmBase;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let (request_queue, event_mediator, mut registry_handle, sync_handle) = cosplay_client::ClientSetup::setup(None).await?;

    let task_1 = tokio::spawn(request_queue.run());
    let task_2 = tokio::spawn(event_mediator.run());

    // Sync roundtrip before ensure global advertisement is done.
    sync_handle.sync().await?;

    let wl_shm_id = registry_handle.bind::<WlShm>().await?;

    let wl_compositor_id = registry_handle.bind::<WlCompositor>().await?;

    let xdg_base_id = registry_handle.bind::<XdgWmBase>().await?;

    tokio::try_join!(task_1, task_2)?;

    Ok(())
}

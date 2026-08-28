//! `cosplay` counterpart of <https://github.com/emersion/hello-wayland>

use cosplay_wayland_client::{compositor::WlCompositorHandle, seat::WlSeatHandle, shared_memory::WlShmHandle};
use cosplay_xdg_shell_client::XdgWmBaseHandle;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let (request_queue, event_mediator, mut registry_handle, sync_handle) = cosplay_core_client::ClientSetup::setup(None).await?;

    tokio::spawn(request_queue.run());
    tokio::spawn(event_mediator.run());

    // Sync roundtrip to ensure globals advertisement has finished.
    sync_handle.sync().await?;

    let wl_shm_handle = registry_handle.bind::<WlShmHandle>()?;

    let mut wl_seat_handle = registry_handle.bind::<WlSeatHandle>()?;

    // Sync roundtrip to ensure seat capability exchange has finished.
    sync_handle.sync().await?;

    let wl_pointer_handle = wl_seat_handle.get_pointer()?;

    let wl_compositor_handle = registry_handle.bind::<WlCompositorHandle>()?;

    let xdg_base_handle = registry_handle.bind::<XdgWmBaseHandle>()?;

    let xdg_surface = xdg_base_handle.create_toplevel(&wl_compositor_handle).await?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

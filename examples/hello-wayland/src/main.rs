//! `cosplay` adaptation of <https://github.com/emersion/hello-wayland>

use cosplay_core_client::{RegistryHandle, SyncHandle};
use cosplay_protocols_wayland::wl_shm::PixelFormat;
use cosplay_wayland_client::{compositor::CompositorHandle, seat::SeatHandle, shared_memory::ShmHandle};
use cosplay_xdg_shell_client::{TopLevelState, XdgWmBaseGlobal};

mod cat;
use cat::CatImage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let (request_queue, event_mediator, mut registry_handle, sync_handle) = cosplay_core_client::ClientSetup::setup(None).await?;

    tokio::spawn(request_queue.run());
    tokio::spawn(event_mediator.run());

    // Sync roundtrip to ensure that all globals have been advertised.
    sync_handle.sync().await?;

    // Prepare surface.
    let wl_compositor_handle = registry_handle.bind::<CompositorHandle>()?;

    let (wm_base_handle, ping_pong_task) = registry_handle.bind::<XdgWmBaseGlobal>()?.into_parts();
    tokio::spawn(ping_pong_task.run());

    let toplevel = wm_base_handle.create_toplevel(&wl_compositor_handle).await?;
    toplevel.set_title("hello-wayland")?;

    // Prepare buffer.
    let shm_handle = create_shm_handle(&mut registry_handle, &sync_handle).await?;
    let mut buffer = shm_handle.create_shm_buffer(CatImage::WIDTH, CatImage::HEIGHT, PixelFormat::Argb8888)?;
    buffer.write(CatImage::BYTES);

    // Display buffer. Dropping the assigned variables causes the corresponding
    // object destructors to be sent.
    let (mut toplevel, _committed_buffer) = toplevel.attach(Some(buffer))?.commit()?;

    loop {
        match toplevel.next_state().await? {
            TopLevelState::ShouldClose => break,
            TopLevelState::Configure { .. } => continue,
        }
    }

    Ok(())
}

async fn create_shm_handle(
    registry_handle: &mut RegistryHandle,
    sync_handle: &SyncHandle,
) -> Result<ShmHandle, Box<dyn std::error::Error>> {
    let mut shm_handle = registry_handle.bind::<ShmHandle>()?;

    // Sync roundtrip to ensure that the server has finished its announcement of
    // all supported pixel formats over `wl_shm::format` events.
    sync_handle.sync().await?;

    shm_handle.sync_supported_formats()?;

    Ok(shm_handle)
}

// TODO: for pointer grab functionality
async fn create_pointer(registry_handle: &mut RegistryHandle, sync_handle: &SyncHandle) -> Result<(), Box<dyn std::error::Error>> {
    let mut wl_seat_handle = registry_handle.bind::<SeatHandle>()?;

    // Sync roundtrip to ensure seat capability exchange has finished.
    sync_handle.sync().await?;

    let wl_pointer_handle = wl_seat_handle.get_pointer()?;

    Ok(())
}

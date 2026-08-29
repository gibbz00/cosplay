//! `cosplay` adaptation of <https://github.com/emersion/hello-wayland>

use cosplay_core_client::{RegistryHandle, SyncHandle};
use cosplay_protocols_wayland::wl_shm::PixelFormat;
use cosplay_wayland_client::{
    compositor::WlCompositorHandle,
    seat::WlSeatHandle,
    shared_memory::{WlCombinedBufferHandle, WlShmHandle},
};
use cosplay_xdg_shell_client::XdgWmBaseHandle;

mod cat;
use cat::CatImage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let (request_queue, event_mediator, mut registry_handle, sync_handle) = cosplay_core_client::ClientSetup::setup(None).await?;

    tokio::spawn(request_queue.run());
    tokio::spawn(event_mediator.run());

    // Sync roundtrip to ensure globals advertisement has finished.
    sync_handle.sync().await?;

    let buffer = create_buffer(&mut registry_handle)?;

    let wl_compositor_handle = registry_handle.bind::<WlCompositorHandle>()?;

    let xdg_base_handle = registry_handle.bind::<XdgWmBaseHandle>()?;

    let xdg_toplevel = xdg_base_handle.create_toplevel(&wl_compositor_handle, buffer).await?;

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

fn create_buffer(registry_handle: &mut RegistryHandle) -> Result<WlCombinedBufferHandle, Box<dyn std::error::Error>> {
    let wl_shm_handle = registry_handle.bind::<WlShmHandle>()?;

    let mut wl_buffer = wl_shm_handle.create_combined_buffer(CatImage::WIDTH, CatImage::HEIGHT, PixelFormat::Argb8888)?;

    // SAFETY: Little risk of concurrent access since wl_buffer has yet to be
    // attached to a surface for compositor reads.
    unsafe {
        wl_buffer.region_mut().write(CatImage::BYTES);
    }

    Ok(wl_buffer)
}

// TODO: for pointer grab functionality
async fn create_pointer(registry_handle: &mut RegistryHandle, sync_handle: &SyncHandle) -> Result<(), Box<dyn std::error::Error>> {
    let mut wl_seat_handle = registry_handle.bind::<WlSeatHandle>()?;
    // Sync roundtrip to ensure seat capability exchange has finished.
    sync_handle.sync().await?;

    let wl_pointer_handle = wl_seat_handle.get_pointer()?;

    Ok(())
}

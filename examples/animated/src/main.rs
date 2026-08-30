//! `cosplay` adaptation of <https://github.com/emersion/hello-wayland>

use cosplay_agent::geometry::Rectangle;
use cosplay_core_client::{RegistryHandle, SyncHandle};
use cosplay_protocols_wayland::wl_shm::PixelFormat;
use cosplay_wayland_client::{
    compositor::{Empty, WlCompositorHandle},
    shared_memory::{Available, ShmBuffer, WlShmHandle},
};
use cosplay_xdg_shell_client::{XdgToplevelHandle, XdgWmBaseGlobal};

const HEIGHT: u16 = u8::MAX as u16;
const WIDTH: u16 = u8::MAX as u16;
const PIXEL_FORMAT: PixelFormat = PixelFormat::Argb8888;
const PIXEL_DENSITY: usize = const {
    match cosplay_agent::pixel_format::bytes_per_pixel(PIXEL_FORMAT) {
        Some(density) => density as usize,
        None => unreachable!(),
    }
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let (request_queue, event_mediator, mut registry_handle, sync_handle) = cosplay_core_client::ClientSetup::setup(None).await?;

    tokio::spawn(request_queue.run());
    tokio::spawn(event_mediator.run());

    // Sync roundtrip to ensure that all globals have been advertised.
    sync_handle.sync().await?;

    // Prepare surface.
    let wl_compositor_handle = registry_handle.bind::<WlCompositorHandle>()?;
    let (xdg_base_handle, ping_pong_task) = registry_handle.bind::<XdgWmBaseGlobal>()?.into_parts();
    tokio::spawn(ping_pong_task.run());
    let toplevel_surface = xdg_base_handle.create_toplevel(&wl_compositor_handle).await?;

    // Prepare buffer.
    let shm_handle = create_shm_handle(&mut registry_handle, &sync_handle).await?;
    let buffer = shm_handle.create_shm_buffer(HEIGHT, WIDTH, PIXEL_FORMAT)?;

    animate(toplevel_surface, buffer).await
}

async fn create_shm_handle(
    registry_handle: &mut RegistryHandle,
    sync_handle: &SyncHandle,
) -> Result<WlShmHandle, Box<dyn std::error::Error>> {
    let mut shm_handle = registry_handle.bind::<WlShmHandle>()?;

    // Sync roundtrip to ensure that the server has finished its announcement of
    // all supported pixel formats over `wl_shm::format` events.
    sync_handle.sync().await?;

    shm_handle.sync_supported_formats()?;

    Ok(shm_handle)
}

async fn animate(mut surface: XdgToplevelHandle<Empty>, mut buffer: ShmBuffer<Available>) -> Result<(), Box<dyn std::error::Error>> {
    let mut timestamp = 0;

    loop {
        let callback_frame = surface.as_ref().callback_frame()?;

        fill(&mut buffer, timestamp_to_period(timestamp));

        let pending_surface = surface.attach(Some(buffer))?;

        // Indicate that we want to overwrite everything.
        pending_surface
            .as_ref()
            .damage_buffer(Rectangle { x: 0, y: 0, width: WIDTH, height: HEIGHT })?;

        let (empty_surface, committed_buffer) = pending_surface.commit()?;

        surface = empty_surface;

        // Expecting this to be about the roundtrip duration commit request + compositor memcopy + release
        // event. Local testing showed a value of about 1ms. Which is well below the limit of ~16ms for 60
        // FPS.
        buffer = committed_buffer.expect("Surface attached with null buffer.").release().await?;

        // Wait for when the callback suggests that we should commit a new frame.
        timestamp = callback_frame.await?;
    }
}

fn timestamp_to_period(timestamp: u32) -> u8 {
    const PERIOD_MS: u32 = 4000;

    // [0, PERIOD_MS]
    let millisecond = timestamp % PERIOD_MS;
    // [0, 255]
    ((millisecond * 255) / PERIOD_MS) as u8
}

fn fill(buffer: &mut ShmBuffer<Available>, period: u8) {
    use palette::IntoColor;

    let color: palette::Srgb = palette::Hsv::new_srgb(period, 190, 190).into_format().into_color();
    let (r, g, b) = color.into_components();
    let pixel_data = [lossy(b), lossy(g), lossy(r), 0xFF];

    for chunk in buffer.as_mut_slice().as_chunks_mut::<PIXEL_DENSITY>().0 {
        *chunk = pixel_data;
    }

    const fn lossy(unit: f32) -> u8 {
        (unit * 255.0) as u8
    }
}

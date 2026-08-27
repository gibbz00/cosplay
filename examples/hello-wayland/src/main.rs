//! `cosplay` counterpart of <https://github.com/emersion/hello-wayland>

use cosplay_client::handle::{compositor::WlCompositorHandle, seat::WlSeatHandle, shared_memory::WlShmHandle};
use cosplay_protocols_xdg_shell::xdg_wm_base::XdgWmBase;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let (request_queue, event_mediator, mut registry_handle, sync_handle) = cosplay_client::ClientSetup::setup(None).await?;

    tokio::spawn(request_queue.run());
    tokio::spawn(event_mediator.run());

    // Sync roundtrip to ensure globals advertisement has finished.
    sync_handle.sync().await?;

    let wl_shm_handle = registry_handle.bind::<WlShmHandle>().await?;

    let mut wl_seat_handle = registry_handle.bind::<WlSeatHandle>().await?;

    // Sync roundtrip to ensure seat capability exchange has finished.
    sync_handle.sync().await?;

    let wl_pointer_handle = wl_seat_handle.get_pointer()?;

    let wl_compositor_handle = registry_handle.bind::<WlCompositorHandle>().await?;

    let surface = wl_compositor_handle.create_surface()?;

    let xdg_base_handle = registry_handle.bind_raw::<XdgWmBase>().await?;

    Ok(())
}

mod xdg_handles {
    use std::sync::Arc;

    use cosplay_client::{
        RequestError,
        handle::{
            compositor::{UnassignedRole, WlSurfaceHandle},
            raw::{EventHandle, ObjectEvent, ObjectHandle, RequestHandle},
        },
    };
    use cosplay_protocols_xdg_shell::{
        xdg_surface::XdgSurface,
        xdg_wm_base::{self, Ping, Pong, XdgWmBase, XdgWmBaseEvent},
    };

    fn init_xdg_wm_base(object_handle: ObjectHandle<XdgWmBase>) {
        let (request_handle, event_handle) = object_handle.into_split();

        let request_handle = Arc::new(request_handle);

        let ping_pong = run_ping_pong(request_handle, event_handle);
    }

    pub struct XdgSurfacePlaceholderRole;

    fn get_xdg_surface(
        request_handle: &Arc<RequestHandle<XdgWmBase>>,
        surface: WlSurfaceHandle<UnassignedRole>,
    ) -> Result<(WlSurfaceHandle<XdgSurfacePlaceholderRole>, ObjectHandle<XdgSurface>), RequestError> {
        let xdg_surface = request_handle.init_subobject(|id| xdg_wm_base::GetXdgSurface { id, surface: surface.id() })?;

        let wayland_surface = surface.with_role();

        Ok((wayland_surface, xdg_surface))
    }

    async fn run_ping_pong(request_handle: Arc<RequestHandle<XdgWmBase>>, mut event_handle: EventHandle<XdgWmBase>) {
        while let Some(result) = event_handle.recv().await {
            match result {
                Ok(ObjectEvent::Event(XdgWmBaseEvent::Ping(Ping { serial }))) => {
                    if request_handle.queue_request(Pong { serial }).is_err() {
                        // TODO: log and break
                    }
                }
                Ok(ObjectEvent::Error { code, message }) => {
                    // TODO: log error
                }
                Err(error) => {
                    // TODO: log error
                }
            }
        }

        // TODO(log):
    }
}

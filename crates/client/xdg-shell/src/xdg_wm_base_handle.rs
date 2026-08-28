use std::sync::Arc;

use cosplay_core_client::*;
use cosplay_protocols_xdg_shell::xdg_wm_base::{self, Ping, Pong, XdgWmBase, XdgWmBaseEvent};
use cosplay_wayland_client::compositor::{UnassignedRole, WlCompositorHandle, WlSurfaceHandle};

use crate::*;

// FIXME: document tokio dependency and risk of panic in from_raw

pub struct XdgWmBaseHandle {
    // FIXME: Graceful destructor request: "Destroying a bound xdg_wm_base object while there are surfaces still alive created by this
    // xdg_wm_base object instance is illegal and will result in a defunct_surfaces error."
    request_handle: Arc<RequestHandle<XdgWmBase>>,
}

impl GlobalHandle for XdgWmBaseHandle {
    type Interface = XdgWmBase;

    fn from_raw(handle: ObjectHandle<Self::Interface>) -> Self {
        let (request_handle, event_handle) = handle.into_split();

        let request_handle = Arc::new(request_handle);

        let task = ping_pong_task(request_handle.clone(), event_handle);

        tokio::spawn(task);

        return Self { request_handle };

        async fn ping_pong_task(request_handle: Arc<RequestHandle<XdgWmBase>>, mut event_handle: EventHandle<XdgWmBase>) {
            while let Some(result) = event_handle.recv().await {
                match result {
                    Ok(ObjectEvent::Event(XdgWmBaseEvent::Ping(Ping { serial }))) => {
                        tracing::debug!(serial, "Received ping request. Returning pong.");

                        if request_handle.enqueue(Pong { serial }).is_err() {
                            tracing::info!("Request channel closed. Aborting");
                            return;
                        }
                    }
                    Ok(ObjectEvent::Error { code, message }) => {
                        tracing::warn!(code, message, "Received unhandled error event.");
                    }
                    Err(error) => {
                        tracing::error!(%error, "Failed to deserialize inbound event.");
                    }
                }
            }

            tracing::info!("Event channel closed. Aborting.");
        }
    }
}

impl XdgWmBaseHandle {
    pub async fn create_toplevel(&self, compositor: &WlCompositorHandle) -> Result<XdgToplevelHandle, XdgToplevelError> {
        // Create within the library to prevent users from passing a surface
        // that has a a role or a buffer already attached.
        let wayland_surface = compositor.create_surface()?;

        let xdg_surface = self
            .request_handle
            .init_subobject(|id| xdg_wm_base::GetXdgSurface { id, surface: wayland_surface.id() })?;

        XdgToplevelHandle::new(wayland_surface, xdg_surface).await
    }
}

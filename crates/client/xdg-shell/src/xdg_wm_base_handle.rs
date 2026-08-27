use std::sync::Arc;

use cosplay_core_client::*;
use cosplay_protocols_xdg_shell::xdg_wm_base::{self, Ping, Pong, XdgWmBase, XdgWmBaseEvent};
use cosplay_wayland_client::compositor::{UnassignedRole, WlSurfaceHandle};

use crate::*;

// FIXME: document tokio dependency and risk of panic in from_raw

pub struct XdgWmBaseHandle {
    // FIXME: graceful destructor request
    request_handle: Arc<RequestHandle<XdgWmBase>>,
}

impl GlobalHandle for XdgWmBaseHandle {
    type Interface = XdgWmBase;

    fn from_raw(handle: ObjectHandle<Self::Interface>) -> Self {
        let (request_handle, event_handle) = handle.into_split();

        let request_handle = Arc::new(request_handle);

        let task = Self::ping_pong_task(request_handle.clone(), event_handle);

        tokio::spawn(task);

        Self { request_handle }
    }
}

impl XdgWmBaseHandle {
    pub fn get_xdg_surface(
        &self,
        surface: WlSurfaceHandle<UnassignedRole>,
    ) -> Result<(WlSurfaceHandle<XdgSurfacePlaceholderRole>, XdgSurfaceHandle), RequestError> {
        let xdg_surface = self
            .request_handle
            .init_subobject(|id| xdg_wm_base::GetXdgSurface { id, surface: surface.id() })
            .map(XdgSurfaceHandle::new)?;

        let wayland_surface = surface.with_role();

        Ok((wayland_surface, xdg_surface))
    }

    async fn ping_pong_task(request_handle: Arc<RequestHandle<XdgWmBase>>, mut event_handle: EventHandle<XdgWmBase>) {
        while let Some(result) = event_handle.recv().await {
            match result {
                Ok(ObjectEvent::Event(XdgWmBaseEvent::Ping(Ping { serial }))) => {
                    if request_handle.enqueue(Pong { serial }).is_err() {
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

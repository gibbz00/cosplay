use cosplay_codec::{Enumeration, IntoInboundError, ReleaseRequest};
use cosplay_core_client::{ObjectEvent, ObjectHandle, RequestError};
use cosplay_protocols_xdg_shell::{
    xdg_surface::{AckConfigure, Configure, Error, GetToplevel, XdgSurface, XdgSurfaceEvent},
    xdg_toplevel::XdgToplevel,
    xdg_wm_base::GetXdgSurface,
};
use cosplay_wayland_client::{
    compositor::{Empty, Pending, WlCompositorHandle, WlSurfaceHandle},
    shared_memory::{Available, Committed, ShmBuffer},
};

use crate::*;

pub struct XdgToplevelHandle<S> {
    wayland_surface_handle: WlSurfaceHandle<S>,
    raw_handles: RawHandles,
}

// NB: handles not wrapped in `ScopedObjectHandle` for manual
// destructor request ordering in Drop implementation.
struct RawHandles {
    xdg_surface: ObjectHandle<XdgSurface>,
    toplevel: ObjectHandle<XdgToplevel>,
}

impl Drop for RawHandles {
    fn drop(&mut self) {
        // "An xdg_surface must only be destroyed after its role object has been
        // destroyed, otherwise a defunct_role_object error is raised."
        let _ = self.toplevel.request().enqueue(<XdgToplevel as ReleaseRequest>::message());
        let _ = self.xdg_surface.request().enqueue(<XdgSurface as ReleaseRequest>::message());
    }
}

#[derive(Debug, thiserror::Error)]
pub enum XdgToplevelError {
    #[error("Failed to encqueue request.")]
    Request(#[from] RequestError),
    #[error("Unexpected error forwarded to xdg_surface, code: {0:?}, message: {1}")]
    XdgSurfaceEventError(Error, String),
    #[error("Unable to convert inbound event: {0}")]
    IntoInbound(#[from] IntoInboundError),
    #[error("Event channel closed.")]
    EventChannelClosed,
}

impl XdgToplevelHandle<Empty> {
    pub(super) async fn new(base_handle: &XdgWmBaseHandle, compositor_handle: &WlCompositorHandle) -> Result<Self, XdgToplevelError> {
        // IMPROVEMENT: log and improve error messaging?

        // Encapsulated surface creation in order to prevent users from passing
        // a surface that has a a role or a buffer already attached.
        //
        // - "A role must be assigned before any other requests are made to the xdg_surface object."
        // - "Creating an xdg_surface from a wl_surface which has a buffer attached or committed is a client
        //   error."
        let wayland_surface = compositor_handle.create_surface()?;

        let mut xdg_surface = base_handle
            .request_handle
            .init_subobject(|id| GetXdgSurface { id, surface: wayland_surface.id() })?;

        let toplevel = xdg_surface.request().init_subobject(|id| GetToplevel { id })?;

        // TODO: Intermediary toplevel setup before first commit? (Title, app ID etc.)

        // "After creating a role-specific object and setting it up (e.g. by sending the title, app ID, size
        // constraints, parent, etc), the client must perform an initial commit without any buffer
        // attached."
        wayland_surface.commit()?;

        // TODO: try_recv events from wl_surface and xdg_toplevel to then apply before the next commit.
        //
        // - wl_surface.preferred_buffer_scale
        // - xdg_toplevel::configure
        // - xdg_toplevel::configure_bounds
        // - xdg_toplevel::wm_capabilities

        // "The client must acknowledge it and is then allowed to attach a buffer to map the surface."
        match xdg_surface.event().recv().await {
            Some(Ok(ObjectEvent::Event(XdgSurfaceEvent::Configure(Configure { serial })))) => {
                xdg_surface.request().enqueue(AckConfigure { serial })?;
            }
            Some(Ok(ObjectEvent::Error { code, message })) => {
                let code = Error::from_repr(code);
                return Err(XdgToplevelError::XdgSurfaceEventError(code, message));
            }
            Some(Err(inbound_err)) => return Err(XdgToplevelError::IntoInbound(inbound_err)),
            None => {
                return Err(XdgToplevelError::EventChannelClosed);
            }
        }

        Ok(Self {
            wayland_surface_handle: wayland_surface,
            raw_handles: RawHandles { xdg_surface, toplevel },
        })
    }

    pub fn attach(self, buffer: Option<ShmBuffer<Available>>) -> Result<XdgToplevelHandle<Pending>, RequestError> {
        let Self { wayland_surface_handle, raw_handles: handles } = self;

        let wayland_surface_handle = wayland_surface_handle.attach(buffer)?;

        Ok(XdgToplevelHandle { wayland_surface_handle, raw_handles: handles })
    }
}

impl XdgToplevelHandle<Pending> {
    pub fn commit(self) -> Result<(XdgToplevelHandle<Empty>, Option<ShmBuffer<Committed>>), RequestError> {
        let Self { wayland_surface_handle, raw_handles: handles } = self;

        let (surface, buffer) = wayland_surface_handle.commit()?;

        let this = XdgToplevelHandle { wayland_surface_handle: surface, raw_handles: handles };

        Ok((this, buffer))
    }
}

#[cfg(test)]
mod tests {
    use cosplay_core_client::TestDriver;
    use cosplay_protocols_xdg_shell::{xdg_surface, xdg_toplevel};

    use super::*;

    #[test]
    fn drop_toplevel_then_surface() {
        let (mut driver, xdg_surface) = TestDriver::new_raw::<XdgSurface>();
        let toplevel = xdg_surface.request().init_subobject(|id| GetToplevel { id }).unwrap();

        let surface_id = xdg_surface.id();
        let toplevel_id = toplevel.id();

        drop(RawHandles { xdg_surface, toplevel });

        driver.assert_outbound_request::<GetToplevel>(surface_id);
        driver.assert_outbound_request::<xdg_toplevel::Destroy>(toplevel_id);
        driver.assert_outbound_request::<xdg_surface::Destroy>(surface_id);
    }
}

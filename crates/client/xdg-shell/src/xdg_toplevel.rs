use cosplay_codec::{Enumeration, IntoInboundError};
use cosplay_core_client::{ObjectEvent, ObjectHandle, RequestError};
use cosplay_protocols_xdg_shell::{
    xdg_surface::{AckConfigure, Configure, Error, XdgSurface, XdgSurfaceEvent},
    xdg_toplevel::XdgToplevel,
    xdg_wm_base::GetXdgSurface,
};
use cosplay_wayland_client::{
    compositor::{WlCompositorHandle, WlSurfaceHandle},
    shared_memory::WlCombinedBufferHandle,
};

use crate::*;

pub struct XdgToplevelHandle {
    wayland_surface_handle: WlSurfaceHandle,

    // FIXME: Graceful destructor request; release toplevel before surface.
    // "An xdg_surface must only be destroyed after its role object has been destroyed, otherwise a defunct_role_object error is raised."
    xdg_surface_handle: ObjectHandle<XdgSurface>,
    toplevel_handle: ObjectHandle<XdgToplevel>,
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

impl XdgToplevelHandle {
    pub(super) async fn new(
        base_handle: &XdgWmBaseHandle,
        compositor_handle: &WlCompositorHandle,
        // TEMP: just for POC
        buffer: WlCombinedBufferHandle,
    ) -> Result<Self, XdgToplevelError> {
        // IMPROVEMENT: log and improve error messaging?

        // Encapsulated surface creation in order to prevent users from passing
        // a surface that has a a role or a buffer already attached.
        //
        // - "A role must be assigned before any other requests are made to the xdg_surface object."
        // - "Creating an xdg_surface from a wl_surface which has a buffer attached or committed is a client
        //   error."
        let mut wayland_surface = compositor_handle.create_surface()?;

        let mut xdg_surface = base_handle
            .request_handle
            .init_subobject(|id| GetXdgSurface { id, surface: wayland_surface.id() })?;

        let toplevel = xdg_surface
            .request()
            .init_subobject(|id| cosplay_protocols_xdg_shell::xdg_surface::GetToplevel { id })?;

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

        // "The client must acknowledge it and is then allowed to attach a buffer to map the surface."

        // TODO: requires for setup or can this be moved out?
        {
            wayland_surface.attach(Some(buffer))?;

            // TODO(extra): pass size parameters?
            // A resizable client would store these [xdg_toplevel::configure args], and resize itself when
            // receiving the xdg_surface.configure event.

            wayland_surface.commit()?;
        }

        Ok(Self {
            wayland_surface_handle: wayland_surface,
            xdg_surface_handle: xdg_surface,
            toplevel_handle: toplevel,
        })
    }
}

use cosplay_codec::{Enumeration, IntoInboundError};
use cosplay_core_client::{ObjectEvent, ObjectHandle, RequestError};
use cosplay_protocols_xdg_shell::{
    xdg_surface::{AckConfigure, Configure, Error, XdgSurface, XdgSurfaceEvent},
    xdg_toplevel::XdgToplevel,
};
use cosplay_wayland_client::compositor::{UnassignedRole, WlSurfaceHandle};

pub struct XdgToplevelSurfaceRole;

pub struct XdgToplevelHandle {
    wayland_surface_handle: WlSurfaceHandle<XdgToplevelSurfaceRole>,

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
        wayland_surface: WlSurfaceHandle<UnassignedRole>,
        mut xdg_surface: ObjectHandle<XdgSurface>,
    ) -> Result<Self, XdgToplevelError> {
        // IMPROVEMENT: log and improve error messaging?

        // "A role must be assigned before any other requests are made to the xdg_surface object."
        let wayland_surface = wayland_surface.with_role();

        let toplevel = xdg_surface
            .request()
            .init_subobject(|id| cosplay_protocols_xdg_shell::xdg_surface::GetToplevel { id })?;

        // TODO: further toplevel setup?
        //
        // "After creating a role-specific object and setting it up (e.g. by
        // sending the title, app ID, size constraints, parent, etc)."

        // After creating a role-specific object and setting it up (e.g. by sending the title, app ID, size
        // constraints, parent, etc), the client must perform an initial commit without any buffer attached.
        wayland_surface.commit()?;

        // TODO: listen to events from wl_surface and toplevel configure?
        //
        // The compositor will reply with initial wl_surface state such as wl_surface.preferred_buffer_scale
        // followed by an xdg_surface.configure event.
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

        // TODO: attach buffer

        // TODO(extra): pass size parameters?
        // A resizable client would store these [xdg_toplevel::configure args], and resize itself when
        // receiving the xdg_surface.configure event.

        // TODO: commit buffer

        Ok(Self {
            wayland_surface_handle: wayland_surface,
            xdg_surface_handle: xdg_surface,
            toplevel_handle: toplevel,
        })
    }
}

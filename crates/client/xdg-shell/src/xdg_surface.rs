use cosplay_core_client::ObjectHandle;
use cosplay_protocols_xdg_shell::xdg_surface::XdgSurface;
use cosplay_wayland_client::compositor::{OverridableRole, SurfaceRole};

pub struct XdgSurfacePlaceholderRole;

impl SurfaceRole for XdgSurfacePlaceholderRole {
    type Overridable = OverridableRole;
}

pub struct XdgSurfaceHandle {
    // FIXME: graceful destructor request
    handle: ObjectHandle<XdgSurface>,
}

impl XdgSurfaceHandle {
    pub(crate) fn new(handle: ObjectHandle<XdgSurface>) -> Self {
        Self { handle }
    }
}

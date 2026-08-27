// TEMP:
#![allow(missing_docs)]

mod xdg_wm_base_handle;
pub use xdg_wm_base_handle::XdgWmBaseHandle;

mod xdg_surface;
pub use xdg_surface::XdgSurfaceHandle;
pub(crate) use xdg_surface::XdgSurfacePlaceholderRole;

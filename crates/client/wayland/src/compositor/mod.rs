mod compositor_handle;
pub use compositor_handle::WlCompositorHandle;

mod region_handle;
pub use region_handle::WlRegionHandle;

mod surface_handle;
pub use surface_handle::WlSurfaceHandle;

mod surface_role;
pub use surface_role::{FixedRole, OverridableRole, SurfaceRole, UnassignedRole};

#[sealed::sealed]
pub trait SurfaceRoleChange {}

pub struct OverridableRole {
    _priv: (),
}

#[sealed::sealed]
impl SurfaceRoleChange for OverridableRole {}

pub struct FixedRole {
    _priv: (),
}

#[sealed::sealed]
impl SurfaceRoleChange for FixedRole {}

pub trait SurfaceRole {
    type Overridable: SurfaceRoleChange;
}

pub struct UnassignedRole;

impl SurfaceRole for UnassignedRole {
    type Overridable = OverridableRole;
}

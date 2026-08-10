#[sealed::sealed]
pub trait EnumRepr {}
#[sealed::sealed]
impl EnumRepr for u32 {}
#[sealed::sealed]
impl EnumRepr for i32 {}

pub trait Enumeration {
    type Repr: EnumRepr;

    fn from_repr(repr: Self::Repr) -> Self;

    fn to_repr(&self) -> Self::Repr;
}

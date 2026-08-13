#[sealed::sealed]
pub trait EnumRepr {}
#[sealed::sealed]
impl EnumRepr for u32 {}
#[sealed::sealed]
impl EnumRepr for i32 {}

/// Bridge for providing blanket implementations to the sealed [`MarshalArgument`] and
/// [`ParseArgument`] traits.
///
/// Used types representable from a single u32 or i32, i.e. enums and bitfields.
pub trait Enumeration {
    #[allow(missing_docs)]
    type Repr: EnumRepr;

    #[allow(missing_docs)]
    fn from_repr(repr: Self::Repr) -> Self;

    #[allow(missing_docs)]
    fn to_repr(&self) -> Self::Repr;
}

use std::marker::PhantomData;

/// Enums in Wayland may have different representations depending on where they are used.
///
/// This wrapper is for declaring argument of different reprs. Ex:
///
/// ```
/// # use cosplay_codec::EnumArg;
/// enum Foo {
///     A,
///     B,
/// }
///
/// struct Request {
///     arg_1: EnumArg<Foo, u32>,
///     arg_2: EnumArg<Foo, i32>,
/// }
/// ```
///
/// This is not only a theoretical aspect of the protocol, but (unfortunately?) something that is
/// used in practice. For example; `wl_output.transform` is represented by an i32 in
/// `wl_surface::set_buffer_transform`, but as an u23 in `wl_surface::preferred_buffer_transform`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EnumArg<E, R>(E, PhantomData<R>);

impl<E, R> EnumArg<E, R> {
    /// Retrieve then underlying enum from the arg wrapper.
    pub fn inner(self) -> E {
        self.0
    }
}

impl<E, R> From<E> for EnumArg<E, R> {
    fn from(value: E) -> Self {
        Self(value, PhantomData)
    }
}

/// Bridge for enabling blanket implementations to the sealed [`MarshalArgument`] and
/// [`ParseArgument`] traits for enums wrapped in [`EnumArg`].
#[allow(private_bounds)]
pub trait Enumeration<R: EnumRepr> {
    #[allow(missing_docs)]
    fn from_repr(repr: R) -> Self;

    #[allow(missing_docs)]
    fn to_repr(&self) -> R;
}

#[sealed::sealed]
pub(crate) trait EnumRepr {}
#[sealed::sealed]
impl EnumRepr for u32 {}
#[sealed::sealed]
impl EnumRepr for i32 {}

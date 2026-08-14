use std::marker::PhantomData;

/// Newtype wrapper around a u32.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OpaqueObjectId(pub u32);

/// Wrapper around an [`OpaqueObjectId`] but with an interface type parameter assigned to it.
#[impl_tools::autoimpl(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ObjectId<I> {
    pub(crate) inner: OpaqueObjectId,
    pub(crate) interface_marker: PhantomData<I>,
}

impl<I> ObjectId<I> {
    /// Construct typed object id from an opaque identifier.
    pub fn new(inner: OpaqueObjectId) -> Self {
        Self { inner, interface_marker: PhantomData }
    }
}

/// `new_id` without a defined interface. Notably used in `wl_registry::bind`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OpaqueNewObjectId {
    pub(crate) interface_name: String,
    pub(crate) interface_version: u32,
    pub(crate) inner: u32,
}

/// `new_id` message argument. Wraps an [`ObjectId<I>`].
#[impl_tools::autoimpl(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NewObjectId<I>(pub(crate) ObjectId<I>);

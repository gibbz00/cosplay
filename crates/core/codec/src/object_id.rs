use std::marker::PhantomData;

/// Newtype wrapper around a u32.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OpaqueObjectId(pub(crate) u32);

impl OpaqueObjectId {
    /// Construct a new opaque object id.
    pub const fn new(inner: u32) -> Self {
        Self(inner)
    }

    /// Retrieve the raw object id identifier value.
    pub const fn inner(&self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for OpaqueObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Wrapper around an [`OpaqueObjectId`] but with an interface type parameter assigned to it.
#[impl_tools::autoimpl(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectId<I> {
    pub(crate) inner: OpaqueObjectId,
    pub(crate) interface_marker: PhantomData<I>,
}

impl<I> ObjectId<I> {
    /// Construct typed object id from an opaque identifier.
    pub const fn new(inner: OpaqueObjectId) -> Self {
        Self { inner, interface_marker: PhantomData }
    }

    /// Retrieve the raw object id identifier value.
    pub const fn inner(&self) -> u32 {
        self.inner.inner()
    }

    /// Create an opaque copy of self.
    pub const fn as_opaque(&self) -> OpaqueObjectId {
        self.inner
    }
}

/// `new_id` without a defined interface. Notably used in `wl_registry::bind`.
#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(missing_docs)]
pub struct OpaqueNewObjectId {
    pub interface_name: String,
    pub interface_version: u32,
    pub object_id: OpaqueObjectId,
}

/// `new_id` message argument. Wraps an [`ObjectId<I>`].
#[impl_tools::autoimpl(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NewObjectId<I>(pub(crate) ObjectId<I>);

impl<I> NewObjectId<I> {
    /// Construct a typed `new_id`.
    pub const fn new(inner: OpaqueObjectId) -> Self {
        Self(ObjectId::new(inner))
    }

    /// Retrieve the raw object id identifier value.
    pub const fn inner(&self) -> u32 {
        self.0.inner()
    }

    /// Promote a new_id to a regular [`ObjectId`]
    pub const fn promote(self) -> ObjectId<I> {
        self.0
    }
}

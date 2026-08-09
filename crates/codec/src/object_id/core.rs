use std::marker::PhantomData;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OpaqueObjectId(pub(crate) u32);

#[impl_tools::autoimpl(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ObjectId<I> {
    pub(crate) inner: OpaqueObjectId,
    pub(crate) interface_marker: PhantomData<I>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OpaqueNewObjectId {
    pub(crate) interface_name: String,
    pub(crate) interface_version: u32,
    pub(crate) inner: u32,
}

/// `new_id` message argument
///
/// Wraps an [`ObjectId<I>`].
#[impl_tools::autoimpl(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NewObjectId<I>(pub(crate) ObjectId<I>);

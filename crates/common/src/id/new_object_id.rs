use std::marker::PhantomData;

use crate::*;

/// `new_id` message argument
///
/// Wraps an [`ObjectId<E>`] and a [`PhantomData`] marker for the interface `I`.
#[impl_tools::autoimpl(Debug, PartialEq, Eq, Clone, Copy)]
pub struct NewObjectId<E, I> {
    pub(crate) inner: ObjectId<E>,
    interface_marker: PhantomData<I>,
}

impl<E, I> NewObjectId<E, I> {
    pub(crate) const fn new(inner: ObjectId<E>) -> Self {
        Self { inner, interface_marker: PhantomData }
    }
}

#[impl_tools::autoimpl(Debug, PartialEq, Eq, Clone)]
pub struct OpaqueNewId<E> {
    pub(crate) interface_name: String,
    pub(crate) interface_version: u32,
    pub(crate) inner: ObjectId<E>,
}

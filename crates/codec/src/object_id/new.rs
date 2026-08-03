use std::marker::PhantomData;

use crate::*;

/// `new_id` message argument
///
/// Wraps an [`ObjectId<E>`] and an interface marker `I`.
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

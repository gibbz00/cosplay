use std::marker::PhantomData;

use crate::*;

/// `new_id` message argument
///
/// Wraps an [`ObjectId<E>`] and a [`PhantomData`] marker for the interface `I`.
#[derive(Clone, Copy)]
#[impl_tools::autoimpl(Debug, PartialEq, Eq)]
pub struct NewObjectId<E, I> {
    inner: ObjectId<E>,
    interface_marker: PhantomData<I>,
}

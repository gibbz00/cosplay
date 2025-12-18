use std::marker::PhantomData;

use crate::*;

#[derive(Clone, Copy)]
#[impl_tools::autoimpl(Debug, PartialEq, Eq)]
pub struct NewObjectId<E, I> {
    inner: ObjectId<E>,
    interface_marker: PhantomData<I>,
}

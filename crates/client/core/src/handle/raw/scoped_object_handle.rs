use std::ops::{Deref, DerefMut};

use cosplay_codec::{EncodeMessage, Message, ReleaseRequest};

use crate::*;

#[impl_tools::autoimpl(Debug)]
pub struct ScopedObjectHandle<I: ReleaseRequest>(ObjectHandle<I>)
where
    I::Message: Message<Interface = I> + EncodeMessage;

impl<I: ReleaseRequest> Deref for ScopedObjectHandle<I>
where
    I::Message: Message<Interface = I> + EncodeMessage,
{
    type Target = ObjectHandle<I>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I: ReleaseRequest> DerefMut for ScopedObjectHandle<I>
where
    I::Message: Message<Interface = I> + EncodeMessage,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<I: ReleaseRequest> From<ObjectHandle<I>> for ScopedObjectHandle<I>
where
    I::Message: Message<Interface = I> + EncodeMessage,
{
    fn from(value: ObjectHandle<I>) -> Self {
        Self(value)
    }
}

impl<I: ReleaseRequest> Drop for ScopedObjectHandle<I>
where
    I::Message: Message<Interface = I> + EncodeMessage,
{
    fn drop(&mut self) {
        let _ = self.request.queue_request(I::message());
    }
}

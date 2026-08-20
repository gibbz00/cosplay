use crate::ObjectHandle;

// TODO: seal?
pub trait Handle {
    type Interface;

    fn from_raw(object_handle: ObjectHandle<Self::Interface>) -> Self;
}

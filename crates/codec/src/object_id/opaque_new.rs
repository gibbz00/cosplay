use crate::*;

#[impl_tools::autoimpl(Debug, PartialEq, Eq, Clone)]
pub struct OpaqueNewObjectId<E> {
    pub(crate) interface_name: String,
    pub(crate) interface_version: u32,
    pub(crate) inner: ObjectId<E>,
}

mod bounded;
pub(crate) use bounded::{ObjectIdBounds, ObjectIdDecodeError, ObjectIdFactory, verify_raw};

mod core;
pub(crate) use core::{NewObjectId, ObjectId, OpaqueNewObjectId, OpaqueObjectId};

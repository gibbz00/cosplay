mod bounded;
pub(crate) use bounded::{ObjectIdBounds, ObjectIdDecodeError, ObjectIdFactory, verify_raw};

mod core;
pub use core::{NewObjectId, ObjectId, OpaqueNewObjectId, OpaqueObjectId};

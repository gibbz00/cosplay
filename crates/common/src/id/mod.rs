//! Wayland object identifiers.

mod object_id;
pub(crate) use object_id::{ObjectId, ObjectIdBounds, ObjectIdFromRawError};

mod new_object_id;
pub use new_object_id::NewObjectId;

//! Wayland object identifiers.

mod any_id;
pub(crate) use any_id::AnyObjectId;

mod object_id;
pub(crate) use object_id::{ObjectId, ObjectIdBounds, ObjectIdFromRawError};

mod new_object_id;
pub use new_object_id::NewObjectId;

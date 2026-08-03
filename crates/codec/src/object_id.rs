mod bounds;
pub(crate) use bounds::ObjectIdBounds;

mod factory;
pub(crate) use factory::ObjectIdFactory;

mod core;
pub(crate) use core::{ObjectId, ObjectIdDecodeError};

use crate::*;

/// Interface declaration trait.
pub trait Interface {
    /// Interface Owner
    ///
    /// Used for associating type-level guarantees to [ObjectId].
    type Owner: Entity;
}

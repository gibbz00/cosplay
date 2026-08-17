//! Items for managing a limited pool of object IDs.

// HELP: Naive implementation that can probably be improved further down the
// line. Unsure how though. Suffers currently when allocating lots of objects
// and then releasing them.

use std::{
    marker::PhantomData,
    ops::RangeInclusive,
    sync::atomic::{AtomicU32, Ordering},
};

use cosplay_codec::OpaqueObjectId;

use crate::*;

/// Declare min and max object ID value to be used in an object ID pool.
#[sealed::sealed]
pub trait ObjectIdBounds: Agent {
    #[allow(missing_docs)]
    const RANGE: RangeInclusive<u32>;
}

#[sealed::sealed]
impl ObjectIdBounds for Client {
    const RANGE: RangeInclusive<u32> = 1..=0xFEFFFFFF;
}

#[sealed::sealed]
impl ObjectIdBounds for Server {
    const RANGE: RangeInclusive<u32> = 0xFF000000..=0xFFFFFFFF;
}

/// Create a new object ID pool for a given [`Agent`].
///
/// The agent parameter denotes the creator of the `NewObjectId` message argument.
///
/// Note that only one pool should be used per connection.
pub fn create<A: ObjectIdBounds>() -> (ObjectIdRetriever<A>, ObjectIdReturner) {
    let (tx, rx) = async_channel::unbounded();

    let retriever = ObjectIdRetriever {
        next: AtomicU32::new(*A::RANGE.start()),
        rx,
        bounds_marker: PhantomData,
    };

    let returner = ObjectIdReturner { tx };

    (retriever, returner)
}

/// Handle for removing object IDs from the object ID pool.
///
/// Intended to be used  items that create `new_id`s.
pub struct ObjectIdRetriever<A> {
    next: AtomicU32,
    rx: async_channel::Receiver<OpaqueObjectId>,
    bounds_marker: PhantomData<A>,
}

impl<A: ObjectIdBounds> ObjectIdRetriever<A> {
    /// Non-blocking retrieval of a new object ID.
    ///
    /// Will first check if any IDs have been returned by the [`ObjectIdReturner`]. If
    /// none, it tries create a new one from its increment counter. If the increment
    /// counter has reached its max value, none is returned. This means that there are no
    /// free object IDs available.
    pub fn try_next(&self) -> Option<OpaqueObjectId> {
        if let Ok(returned) = self.rx.try_recv() {
            return Some(returned);
        }

        self.next
            // ABA problem should be a non-issue, but optimized memory ordering
            // goes a bit over my head here.
            .try_update(Ordering::SeqCst, Ordering::SeqCst, |value| {
                (value < *A::RANGE.end()).then_some(value + 1)
            })
            .ok()
            .map(OpaqueObjectId::new)
    }
}

/// Handle for returning object ID back to the ID pool.
///
/// In servers, it is intended to be held by the receiver of requests marked
/// as destructors. In clients it is intended to be held by the event receiver
/// of `wl_display::delete_id`. Presumably, most object have a destructors
/// which triggers the `delete_id` event. (An exception to this would be
/// `wl_callback`. Its object should instead be returned with the receival of
/// `wl_callback::done`.)
pub struct ObjectIdReturner {
    tx: async_channel::Sender<OpaqueObjectId>,
}

impl ObjectIdReturner {
    /// Return an ID back to the pool.
    ///
    /// Returns `Err` if the [`ObjectIdRetriever`] has been dropped.
    pub fn return_id(&self, object_id: OpaqueObjectId) -> Result<(), OpaqueObjectId> {
        self.tx.try_send(object_id).map_err(|err| match err {
            async_channel::TrySendError::Full(_) => unreachable!("Unbounded channel marked as full."),
            async_channel::TrySendError::Closed(id) => id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_pool_start() {
        let (retriever, _) = create::<Client>();
        let first = retriever.try_next().unwrap().inner();
        assert_eq!(1, first);
    }

    #[test]
    fn server_pool_start() {
        let (retriever, _) = create::<Server>();
        let first = retriever.try_next().unwrap().inner();
        assert_eq!(0xFF000000, first);
    }

    #[test]
    fn pool_next_from_increment() {
        let (retriever, _) = create::<Client>();

        let first = retriever.try_next().unwrap().inner();
        let second = retriever.try_next().unwrap().inner();

        assert_eq!(first + 1, second);
    }

    #[test]
    fn pool_from_returned() {
        let (retriever, returner) = create::<Client>();

        let first = retriever.try_next().unwrap();
        let first_value = first.inner();

        let second = retriever.try_next().unwrap();
        let second_value = second.inner();

        returner.return_id(first).unwrap();

        let next = retriever.try_next().unwrap();
        let next_value = next.inner();

        assert_ne!(second_value + 1, next_value);
        assert_eq!(first_value, next_value);
    }

    #[test]
    fn retriever_dropped() {
        let (retriever, returner) = create::<Client>();

        let id = retriever.try_next().unwrap();

        drop(retriever);

        assert!(returner.return_id(id).is_err());
    }

    #[test]
    fn client_pool_increment_exhausted() {}

    #[test]
    fn server_pool_increment_exhausted() {}
}

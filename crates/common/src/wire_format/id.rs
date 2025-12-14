use std::{marker::PhantomData, ops::RangeInclusive};

use crate::*;

trait ObjectIdBounds {
    const RANGE: RangeInclusive<u32>;
}

impl ObjectIdBounds for Client {
    const RANGE: RangeInclusive<u32> = 1..=0xFEFFFFFF;
}

impl ObjectIdBounds for Server {
    const RANGE: RangeInclusive<u32> = 0xFF000000..=0xFFFFFFFF;
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[impl_tools::autoimpl(Debug)]
pub struct ObjectId<E> {
    inner: u32,
    entity_marker: PhantomData<E>,
}

impl<E: ObjectIdBounds> ObjectId<E> {
    const fn new() -> Self {
        Self { inner: *E::RANGE.start(), entity_marker: PhantomData }
    }

    const fn next(&self) -> Self {
        let next = match self.inner == *E::RANGE.end() {
            true => *E::RANGE.start(),
            false => self.inner + 1,
        };

        Self { inner: next, entity_marker: PhantomData }
    }
}

macro_rules! from_raw_impl {
    ($err:ty, $($check:tt)*) => {
        /// Parse a raw u32 into an ObjectId.
        ///
        /// Zero is used to represent a null or non-existent object, so `raw == 0` returns `Ok(None)`.
        ///
        /// A raw value outside the bounds for the given entity results in an error being returned.
        pub(crate) fn from_raw(raw: u32) -> Result<Option<Self>, $err> {
            if raw == 0 {
                return Ok(None);
            }

            ($($check)*(raw))?;

            Ok(Some(Self { inner: raw, entity_marker: PhantomData }))
        }
    };
}

#[derive(Debug, PartialEq, thiserror::Error)]
#[error("provided value '{0:X}' greater than allowed maximum '{max:X}' for client IDs", max = Client::RANGE.start())]
pub(crate) struct ObjectIdOutOfClientBounds(u32);

impl ObjectId<Client> {
    from_raw_impl!(
        ObjectIdOutOfClientBounds,
        |raw: u32| -> Result<(), ObjectIdOutOfClientBounds> {
            let end = *Client::RANGE.end();
            if raw > end {
                return Err(ObjectIdOutOfClientBounds(raw));
            }

            Ok(())
        }
    );
}

#[derive(Debug, PartialEq, thiserror::Error)]
#[error("provided value '{0:X}' less than allowed minimum '{min:X}' for server IDs", min = Server::RANGE.end())]
pub(crate) struct ObjectIdOutOfServerBounds(u32);

impl ObjectId<Server> {
    from_raw_impl!(
        ObjectIdOutOfServerBounds,
        |raw: u32| -> Result<(), ObjectIdOutOfServerBounds> {
            let start = *Server::RANGE.start();
            if raw < start {
                return Err(ObjectIdOutOfServerBounds(raw));
            }

            Ok(())
        }
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_new() {
        let actual = ObjectId::<Client>::new().inner;
        assert_eq!(1, actual);
    }

    #[test]
    fn server_new() {
        let actual = ObjectId::<Server>::new().inner;
        assert_eq!(0xFF000000, actual);
    }

    #[test]
    fn client_next() {
        let actual = ObjectId::<Client>::new().next().next();
        assert_eq!(3, actual.inner);
    }

    #[test]
    fn server_next() {
        let actual = ObjectId::<Server>::new().next().next();
        assert_eq!(0xFF000002, actual.inner);
    }

    #[test]
    fn client_wrapping_next() {
        let id = ObjectId::<Client> { inner: 0xFEFFFFFE, entity_marker: PhantomData };

        let next = id.next();
        assert_eq!(0xFEFFFFFF, next.inner);

        let next = next.next();
        assert_eq!(1, next.inner);
    }

    #[test]
    fn server_wrapping_next() {
        let id = ObjectId::<Server> { inner: 0xFFFFFFFE, entity_marker: PhantomData };

        let next = id.next();
        assert_eq!(0xFFFFFFFF, next.inner);

        let next = next.next();
        assert_eq!(0xFF000000, next.inner);
    }

    #[test]
    fn from_client_raw_ok() {
        let actual = ObjectId::<Client>::from_raw(2).unwrap().unwrap();
        assert_eq!(2, actual.inner);
    }

    #[test]
    fn from_server_raw_ok() {
        let actual = ObjectId::<Server>::from_raw(0xFF000003).unwrap().unwrap();
        assert_eq!(0xFF000003, actual.inner);
    }

    #[test]
    fn from_client_raw_none() {
        let actual = ObjectId::<Client>::from_raw(0).unwrap();
        assert!(actual.is_none());
    }

    #[test]
    fn from_server_raw_none() {
        let actual = ObjectId::<Server>::from_raw(0).unwrap();
        assert!(actual.is_none());
    }

    #[test]
    fn from_client_raw_greater_than_error() {
        let actual_error = ObjectId::<Client>::from_raw(0xFFFF0000).unwrap_err();
        let expected_error = ObjectIdOutOfClientBounds(0xFFFF0000);
        assert_eq!(expected_error, actual_error);
    }

    #[test]
    fn from_server_raw_less_than_error() {
        let actual_error = ObjectId::<Server>::from_raw(3).unwrap_err();
        let expected_error = ObjectIdOutOfServerBounds(3);
        assert_eq!(expected_error, actual_error);
    }
}

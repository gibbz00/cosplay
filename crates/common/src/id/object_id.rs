use std::{marker::PhantomData, ops::RangeInclusive};

use crate::*;

pub(crate) trait ObjectIdBounds {
    const RANGE: RangeInclusive<u32>;
}

impl ObjectIdBounds for Client {
    const RANGE: RangeInclusive<u32> = 1..=0xFEFFFFFF;
}

impl ObjectIdBounds for Server {
    const RANGE: RangeInclusive<u32> = 0xFF000000..=0xFFFFFFFF;
}

#[impl_tools::autoimpl(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ObjectId<E> {
    inner: u32,
    entity_marker: PhantomData<E>,
}

impl<E> ObjectId<E> {
    pub(crate) const fn inner(&self) -> u32 {
        self.inner
    }
}

impl<E: ObjectIdBounds> ObjectId<E> {
    const fn first() -> Self {
        Self { inner: *E::RANGE.start(), entity_marker: PhantomData }
    }

    const fn next(&self) -> Self {
        let next = match self.inner == *E::RANGE.end() {
            true => *E::RANGE.start(),
            false => self.inner + 1,
        };

        Self { inner: next, entity_marker: PhantomData }
    }

    /// Parse a raw u32 into an optional `ObjectId`.
    ///
    /// Zero is used to represent a null or non-existent object, so `raw == 0` returns `Ok(None)`.
    pub(crate) const fn from_raw_optional(raw: u32) -> Result<Option<Self>, ObjectIdFromRawError> {
        if raw == 0 {
            return Ok(None);
        }

        // IMPROVEMENT(perf): start/end bounds checking can be skipped here
        // depending if it is a Client or Server. Self::from_raw_expected checks
        // both ends, always.

        match Self::from_raw_expected(raw) {
            Ok(id) => Ok(Some(id)),
            Err(err) => Err(err),
        }
    }

    /// Parse a raw u32 into an `ObjectId`.
    ///
    /// A raw value outside the bounds for the given entity results in an error being returned.
    ///
    /// Use [ObjectId::from_raw_optional] for optional values.
    pub(crate) const fn from_raw_expected(raw: u32) -> Result<Self, ObjectIdFromRawError> {
        let start = *E::RANGE.start();

        if raw < start {
            return Err(ObjectIdFromRawError::RawLessThanMin(raw, start));
        }

        let end = *E::RANGE.end();
        if raw > end {
            return Err(ObjectIdFromRawError::RawGreaterThanMax(raw, end));
        }

        Ok(Self { inner: raw, entity_marker: PhantomData })
    }
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ObjectIdFromRawError {
    #[error("provided value '{0:X}' greater than allowed maximum '{1:X}'")]
    RawGreaterThanMax(u32, u32),
    #[error("provided value '{0:X}' less than allowed minimum '{1:X}'")]
    RawLessThanMin(u32, u32),
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::*;

    #[test]
    fn client_first() {
        let actual = ObjectId::<Client>::first().inner;
        assert_eq!(1, actual);
    }

    #[test]
    fn server_first() {
        let actual = ObjectId::<Server>::first().inner;
        assert_eq!(0xFF000000, actual);
    }

    #[test]
    fn client_next() {
        let actual = ObjectId::<Client>::first().next().next();
        assert_eq!(3, actual.inner);
    }

    #[test]
    fn server_next() {
        let actual = ObjectId::<Server>::first().next().next();
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
        let actual = ObjectId::<Client>::from_raw_expected(2).unwrap();
        assert_eq!(2, actual.inner);
    }

    #[test]
    fn from_server_raw_ok() {
        let actual = ObjectId::<Server>::from_raw_expected(0xFF000003).unwrap();
        assert_eq!(0xFF000003, actual.inner);
    }

    #[test]
    fn from_client_raw_greater_than_error() {
        let actual_error = ObjectId::<Client>::from_raw_expected(0xFFFF0000).unwrap_err();
        let expected_error = ObjectIdFromRawError::RawGreaterThanMax(0xFFFF0000, *Client::RANGE.end());
        assert_eq!(expected_error, actual_error);
    }

    #[test]
    fn from_server_raw_less_than_error() {
        let actual_error = ObjectId::<Server>::from_raw_expected(3).unwrap_err();
        let expected_error = ObjectIdFromRawError::RawLessThanMin(3, *Server::RANGE.start());
        assert_eq!(expected_error, actual_error);
    }

    #[test]
    fn optional_from_raw() {
        let actual = ObjectId::<Client>::from_raw_optional(0).unwrap();
        assert!(actual.is_none());

        let actual = ObjectId::<Client>::from_raw_optional(1).unwrap().unwrap();
        assert_eq!(1, actual.inner());
    }
}

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

#[derive(Clone, Copy)]
#[impl_tools::autoimpl(Debug, PartialEq, Eq)]
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

    pub const fn inner(&self) -> u32 {
        self.inner
    }

    /// Parse a raw u32 into an optional `ObjectId`.
    ///
    /// Zero is used to represent a null or non-existent object, so `raw == 0` returns `Ok(None)`.
    const fn from_raw(raw: u32) -> Result<Option<Self>, ObjectIdFromRawError<E>> {
        if raw == 0 {
            return Ok(None);
        }

        // IMPROVEMENT: use Result::map when or if const_result_trait_fn stabilizes
        match Self::from_raw_expected(raw) {
            Ok(id) => Ok(Some(id)),
            Err(err) => Err(err),
        }
    }

    /// Parse a raw u32 into an `ObjectId`.
    ///
    /// A raw value outside the bounds for the given entity results in an error being returned.
    pub(crate) const fn from_raw_expected(raw: u32) -> Result<Self, ObjectIdFromRawError<E>> {
        let start = *E::RANGE.start();
        if raw < start {
            return Err(ObjectIdFromRawError::RawLessThanMin(raw, PhantomData));
        }

        let end = *E::RANGE.end();
        if raw > end {
            return Err(ObjectIdFromRawError::RawGreaterThanMax(raw, PhantomData));
        }

        Ok(Self { inner: raw, entity_marker: PhantomData })
    }
}

#[derive(thiserror::Error)]
#[impl_tools::autoimpl(Debug, PartialEq)]
pub enum ObjectIdFromRawError<E: ObjectIdBounds> {
    #[error("provided value '{0:X}' greater than allowed maximum '{max:X}'", max = E::RANGE.end())]
    RawGreaterThanMax(u32, PhantomData<E>),
    #[error("provided value '{0:X}' less than allowed minimum '{min:X}'", min = E::RANGE.start())]
    RawLessThanMin(u32, PhantomData<E>),
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
        let expected_error = ObjectIdFromRawError::RawGreaterThanMax(0xFFFF0000, PhantomData);
        assert_eq!(expected_error, actual_error);
    }

    #[test]
    fn from_server_raw_less_than_error() {
        let actual_error = ObjectId::<Server>::from_raw(3).unwrap_err();
        let expected_error = ObjectIdFromRawError::RawLessThanMin(3, PhantomData);
        assert_eq!(expected_error, actual_error);
    }
}

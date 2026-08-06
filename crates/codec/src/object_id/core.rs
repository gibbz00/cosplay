use std::marker::PhantomData;

use crate::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct OpaqueObjectId(pub(crate) u32);

#[impl_tools::autoimpl(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ObjectId<E> {
    pub(crate) inner: u32,
    pub(super) entity_marker: PhantomData<E>,
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ObjectIdDecodeError {
    #[error("Provided value '{0:X}' greater than allowed maximum '{1:X}'.")]
    RawGreaterThanMax(u32, u32),
    #[error("Provided value '{0:X}' less than allowed minimum '{1:X}'.")]
    RawLessThanMin(u32, u32),
}

impl<E: ObjectIdBounds> ObjectId<E> {
    pub(crate) const fn from_raw(raw: u32) -> Result<Option<Self>, ObjectIdDecodeError> {
        if raw == 0 {
            return Ok(None);
        }

        let start = *E::RANGE.start();
        if raw < start {
            return Err(ObjectIdDecodeError::RawLessThanMin(raw, start));
        }

        let end = *E::RANGE.end();
        if raw > end {
            return Err(ObjectIdDecodeError::RawGreaterThanMax(raw, end));
        }

        Ok(Some(Self { inner: raw, entity_marker: PhantomData }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn from_client_raw_greater_than_error() {
        let actual_error = ObjectId::<Client>::from_raw(0xFFFF0000).unwrap_err();
        let expected_error = ObjectIdDecodeError::RawGreaterThanMax(0xFFFF0000, *Client::RANGE.end());
        assert_eq!(expected_error, actual_error);
    }

    #[test]
    fn from_server_raw_less_than_error() {
        let actual_error = ObjectId::<Server>::from_raw(3).unwrap_err();
        let expected_error = ObjectIdDecodeError::RawLessThanMin(3, *Server::RANGE.start());
        assert_eq!(expected_error, actual_error);
    }

    #[test]
    fn optional_from_raw() {
        let actual = ObjectId::<Client>::from_raw(0).unwrap();
        assert!(actual.is_none());

        let actual = ObjectId::<Client>::from_raw(1).unwrap().unwrap();
        assert_eq!(1, actual.inner);
    }
}

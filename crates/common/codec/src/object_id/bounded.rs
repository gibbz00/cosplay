mod bounds {
    use std::ops::RangeInclusive;

    use crate::*;

    pub(crate) trait ObjectIdBounds: Entity {
        const RANGE: RangeInclusive<u32>;
    }

    impl ObjectIdBounds for Client {
        const RANGE: RangeInclusive<u32> = 1..=0xFEFFFFFF;
    }

    impl ObjectIdBounds for Server {
        const RANGE: RangeInclusive<u32> = 0xFF000000..=0xFFFFFFFF;
    }
}
pub(crate) use bounds::ObjectIdBounds;

mod factory {
    use std::marker::PhantomData;

    use crate::*;

    pub struct ObjectIdFactory<E> {
        next: u32,
        entity_marker: PhantomData<E>,
    }

    impl<E: ObjectIdBounds> ObjectIdFactory<E> {
        pub const fn new() -> Self {
            Self { next: *E::RANGE.start(), entity_marker: PhantomData }
        }

        pub const fn next(&mut self) -> OpaqueObjectId {
            let current = self.next;

            self.next = match current == *E::RANGE.end() {
                true => *E::RANGE.start(),
                false => current + 1,
            };

            OpaqueObjectId(current)
        }
    }
    #[cfg(test)]
    mod tests {
        use std::marker::PhantomData;

        use super::*;

        #[test]
        fn client_factory_first_and_next() {
            let mut factory = ObjectIdFactory::<Client>::new();
            assert_eq!(1, factory.next().0);
            assert_eq!(2, factory.next().0);
        }

        #[test]
        fn server_factory_first() {
            let mut factory = ObjectIdFactory::<Server>::new();
            assert_eq!(0xFF000000, factory.next().0);
            assert_eq!(0xFF000001, factory.next().0);
        }

        #[test]
        fn client_factory_wrapping_next() {
            let mut factory = ObjectIdFactory::<Client> { next: 0xFEFFFFFF, entity_marker: PhantomData };

            assert_eq!(0xFEFFFFFF, factory.next().0);
            assert_eq!(1, factory.next().0);
        }

        #[test]
        fn server_factory_wrapping_next() {
            let mut factory = ObjectIdFactory::<Server> { next: 0xFFFFFFFF, entity_marker: PhantomData };

            assert_eq!(0xFFFFFFFF, factory.next().0);
            assert_eq!(0xFF000000, factory.next().0);
        }
    }
}
pub(crate) use factory::ObjectIdFactory;

mod parser {
    use crate::*;

    #[derive(Debug, PartialEq, thiserror::Error)]
    pub enum ObjectIdDecodeError {
        #[error("Provided value '{0:X}' greater than allowed maximum '{1:X}'.")]
        RawGreaterThanMax(u32, u32),
        #[error("Provided value '{0:X}' less than allowed minimum '{1:X}'.")]
        RawLessThanMin(u32, u32),
    }

    // Remains to be seen how much of use this is.
    pub(crate) const fn verify_raw<E: ObjectIdBounds>(raw: u32) -> Result<Option<OpaqueObjectId>, ObjectIdDecodeError> {
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

        Ok(Some(OpaqueObjectId(raw)))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn from_client_raw_ok() {
            let actual = verify_raw::<Client>(2).unwrap().unwrap();
            assert_eq!(2, actual.0);
        }

        #[test]
        fn from_server_raw_ok() {
            let actual = verify_raw::<Server>(0xFF000003).unwrap().unwrap();
            assert_eq!(0xFF000003, actual.0);
        }

        #[test]
        fn from_client_raw_greater_than_error() {
            let actual_error = verify_raw::<Client>(0xFFFF0000).unwrap_err();
            let expected_error = ObjectIdDecodeError::RawGreaterThanMax(0xFFFF0000, *Client::RANGE.end());
            assert_eq!(expected_error, actual_error);
        }

        #[test]
        fn from_server_raw_less_than_error() {
            let actual_error = verify_raw::<Server>(3).unwrap_err();
            let expected_error = ObjectIdDecodeError::RawLessThanMin(3, *Server::RANGE.start());
            assert_eq!(expected_error, actual_error);
        }

        #[test]
        fn optional_from_raw() {
            let actual = verify_raw::<Client>(0).unwrap();
            assert!(actual.is_none());

            let actual = verify_raw::<Client>(1).unwrap().unwrap();
            assert_eq!(1, actual.0);
        }
    }
}
pub(crate) use parser::{ObjectIdDecodeError, verify_raw};

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
}

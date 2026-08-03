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

    pub const fn next(&mut self) -> ObjectId<E> {
        let current = self.next;

        self.next = match current == *E::RANGE.end() {
            true => *E::RANGE.start(),
            false => current + 1,
        };

        ObjectId { inner: current, entity_marker: PhantomData }
    }
}
#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::*;

    #[test]
    fn client_factory_first_and_next() {
        let mut factory = ObjectIdFactory::<Client>::new();
        assert_eq!(1, factory.next().inner);
        assert_eq!(2, factory.next().inner);
    }

    #[test]
    fn server_factory_first() {
        let mut factory = ObjectIdFactory::<Server>::new();
        assert_eq!(0xFF000000, factory.next().inner);
        assert_eq!(0xFF000001, factory.next().inner);
    }

    #[test]
    fn client_factory_wrapping_next() {
        let mut factory = ObjectIdFactory::<Client> { next: 0xFEFFFFFF, entity_marker: PhantomData };

        assert_eq!(0xFEFFFFFF, factory.next().inner);
        assert_eq!(1, factory.next().inner);
    }

    #[test]
    fn server_factory_wrapping_next() {
        let mut factory = ObjectIdFactory::<Server> { next: 0xFFFFFFFF, entity_marker: PhantomData };

        assert_eq!(0xFFFFFFFF, factory.next().inner);
        assert_eq!(0xFF000000, factory.next().inner);
    }
}

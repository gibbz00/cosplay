use crate::*;

pub struct Message<E> {
    pub(crate) header: MessageHeader<E>,
    pub(crate) body: Vec<u8>,
}

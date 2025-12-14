use crate::*;

pub struct Message<E> {
    header: MessageHeader<E>,
    body: Vec<u8>,
}

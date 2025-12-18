use crate::*;

pub struct Message<T> {
    pub(crate) header: MessageHeader,
    pub(crate) body: Box<dyn MessageBody<T>>,
}

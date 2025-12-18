use crate::*;

pub struct MessageTemp<T> {
    pub(crate) header: MessageHeader,
    pub(crate) body: Box<dyn MessageBody<T>>,
}

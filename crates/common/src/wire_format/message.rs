mod core {
    pub struct Message {}
}
pub(crate) use core::Message;

mod header {
    use crate::*;

    // 8 bytes
    pub struct MessageHeader<E> {
        id: ObjectId<E>,
        /// header included
        size: u16,
        // TODO: typeset
        op_code: u16,
    }
}

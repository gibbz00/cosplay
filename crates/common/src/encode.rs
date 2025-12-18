use bytes::{BufMut, BytesMut};

use crate::*;

pub(crate) struct FullMessage<'a, M: Message> {
    pub id: ObjectId<<M::Interface as Interface>::Owner>,
    pub message: &'a M,
}

pub(crate) struct FullMessageEncoder;

impl<M: Message> tokio_util::codec::Encoder<FullMessage<'_, M>> for FullMessageEncoder {
    type Error = SocketWriteError;

    fn encode(&mut self, item: FullMessage<'_, M>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        const HEADER_SIZE: u16 = 8;
        dst.put_u32_ne(item.id.inner());
        // TODO: checked u16 conversion
        dst.put_u16_ne(HEADER_SIZE + item.message.size() as u16);
        dst.put_u16_ne(M::OP_CODE);
        item.message.encode(dst);

        Ok(())
    }
}

use bytes::{Buf, BufMut, BytesMut};

use crate::*;

#[derive(Debug, PartialEq)]
pub struct MessageHeader {
    pub(crate) id: AnyObjectId,
    /// header of 8 bytes included
    pub(crate) size: u16,
    // TODO: typeset?
    pub(crate) op_code: u16,
}

#[derive(Debug, thiserror::Error)]
pub enum MessageHeaderDecodeError {
    #[error("zero is not a valid ID value")]
    InvalidId,
}

impl MessageHeader {
    pub(crate) fn encode(&self, dst: &mut BytesMut) {
        dst.put_u32_ne(self.id.inner());
        dst.put_u16_ne(self.size);
        dst.put_u16_ne(self.op_code);
    }

    pub(crate) fn decode(src: &mut BytesMut) -> Result<Option<Self>, MessageHeaderDecodeError> {
        if src.len() < std::mem::size_of::<Self>() {
            return Ok(None);
        }

        let id = AnyObjectId::from_raw(src.get_u32_ne()).ok_or(MessageHeaderDecodeError::InvalidId)?;
        let size = src.get_u16_ne();
        let op_code = src.get_u16_ne();

        Ok(Some(Self { id, size, op_code }))
    }
}

#[cfg(test)]
mod tests {
    use bytes::BufMut;

    use super::*;

    #[test]
    fn size() {
        assert_eq!(8, std::mem::size_of::<MessageHeader>());
    }

    #[test]
    fn decode_header_none() {
        // At least 8 bytes are required
        let mut bytes = BytesMut::from_iter([0; 7]);
        let actual = MessageHeader::decode(&mut bytes);
        assert!(actual.unwrap().is_none());
    }

    #[test]
    fn decode_header_some() {
        let mut bytes = BytesMut::new();
        bytes.put_u32_ne(1);
        bytes.put_u16_ne(2);
        bytes.put_u16_ne(3);

        let actual = MessageHeader::decode(&mut bytes).unwrap().unwrap();

        let expected = MessageHeader { id: AnyObjectId::from_raw(1).unwrap(), size: 2, op_code: 3 };

        assert_eq!(expected, actual)
    }

    #[test]
    fn bijective_encoding() {
        let header = MessageHeader { id: AnyObjectId::from_raw(1).unwrap(), size: 2, op_code: 3 };

        let mut bytes = BytesMut::new();
        header.encode(&mut bytes);

        let decoded_header = MessageHeader::decode(&mut bytes).unwrap().unwrap();

        assert_eq!(header, decoded_header)
    }
}

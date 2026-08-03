use bytes::BytesMut;

#[derive(Debug, PartialEq)]
pub struct OpaqueMessage {
    pub(crate) object_id: u32,
    pub(crate) op_code: u16,
    pub(crate) body: BytesMut,
}

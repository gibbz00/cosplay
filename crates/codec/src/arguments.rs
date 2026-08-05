use std::{collections::VecDeque, os::fd::OwnedFd};

use bytes::{Buf, BufMut, BytesMut};

use crate::*;

const WORD_SIZE: usize = std::mem::size_of::<u32>();

pub struct ArgumentBody<'a> {
    bytes: &'a mut BytesMut,
    fd_buffer: &'a mut VecDeque<OwnedFd>,
}

#[sealed::sealed]
pub trait MarshalArgument: Sized {
    fn marshal(self, body: &mut ArgumentBody<'_>);
}

#[sealed::sealed]
pub trait ParseArgument: Sized {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ArgumentDecodeError {
    #[error("Not enough bytes left in argument buffer in order to decode the next argument.")]
    NotEnoughBytesLeft,
    #[error("Unable to convert message bytes to an UTF-8 string: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("Expected a string, received null.")]
    MissingString,
    #[error("Failed to decode object_id: {0}")]
    ObjectId(#[from] ObjectIdDecodeError),
    #[error("Expected an object id, received null.")]
    MissingObjectId,
    #[error("No file descriptor found.")]
    MissingFd,
}

#[sealed::sealed]
impl MarshalArgument for u32 {
    fn marshal(self, body: &mut ArgumentBody<'_>) {
        body.bytes.put_u32_ne(self);
    }
}

#[sealed::sealed]
impl ParseArgument for u32 {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        body.bytes.try_get_u32_ne().map_err(|_| ArgumentDecodeError::NotEnoughBytesLeft)
    }
}

#[sealed::sealed]
impl MarshalArgument for i32 {
    fn marshal(self, body: &mut ArgumentBody<'_>) {
        body.bytes.put_i32_ne(self);
    }
}

#[sealed::sealed]
impl ParseArgument for i32 {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        body.bytes.try_get_i32_ne().map_err(|_| ArgumentDecodeError::NotEnoughBytesLeft)
    }
}

#[sealed::sealed]
impl MarshalArgument for Fixed {
    fn marshal(self, body: &mut ArgumentBody<'_>) {
        let Fixed { integer, decimal } = self;

        let integer_bytes = integer.to_be_bytes();

        u32::from_be_bytes([integer_bytes[0], integer_bytes[1], integer_bytes[2], decimal]).marshal(body);
    }
}

#[sealed::sealed]
impl ParseArgument for Fixed {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        let inner = u32::parse(body)?.to_be_bytes();

        let fixed = Fixed {
            integer: i24::I24::from_be_bytes([inner[0], inner[1], inner[2]]),
            decimal: inner[3],
        };

        Ok(fixed)
    }
}

#[sealed::sealed]
impl<E> MarshalArgument for ObjectId<E> {
    fn marshal(self, body: &mut ArgumentBody<'_>) {
        self.inner.marshal(body);
    }
}

#[sealed::sealed]
impl<E: ObjectIdBounds> ParseArgument for ObjectId<E> {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        Option::<ObjectId<E>>::parse(body)?.ok_or(ArgumentDecodeError::MissingObjectId)
    }
}

#[sealed::sealed]
impl<E> MarshalArgument for Option<ObjectId<E>> {
    fn marshal(self, body: &mut ArgumentBody<'_>) {
        self.map(|id| id.inner).unwrap_or(0).marshal(body);
    }
}

#[sealed::sealed]
impl<E: ObjectIdBounds> ParseArgument for Option<ObjectId<E>> {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        let raw = ParseArgument::parse(body)?;
        ObjectId::from_raw(raw).map_err(Into::into)
    }
}

#[sealed::sealed]
impl<E, I> MarshalArgument for NewObjectId<E, I> {
    fn marshal(self, body: &mut ArgumentBody<'_>) {
        self.inner.marshal(body);
    }
}

#[sealed::sealed]
impl<E: ObjectIdBounds, I> ParseArgument for NewObjectId<E, I> {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        ObjectId::<E>::parse(body).map(NewObjectId::new)
    }
}

#[sealed::sealed]
impl<E> MarshalArgument for OpaqueNewObjectId<E> {
    fn marshal(self, body: &mut ArgumentBody<'_>) {
        let OpaqueNewObjectId { interface_name, interface_version, inner } = self;

        interface_name.marshal(body);
        interface_version.marshal(body);
        inner.marshal(body);
    }
}

#[sealed::sealed]
impl<E: ObjectIdBounds> ParseArgument for OpaqueNewObjectId<E> {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        let interface_name = String::parse(body)?;
        let interface_version = u32::parse(body)?;
        let inner = ObjectId::<E>::parse(body)?;

        Ok(Self { interface_name, interface_version, inner })
    }
}

#[sealed::sealed]
impl MarshalArgument for OwnedFd {
    fn marshal(self, body: &mut ArgumentBody<'_>) {
        // `push_back` as per `AncillaryBuffer::file_descriptors` instructions.
        body.fd_buffer.push_back(self);
    }
}

#[sealed::sealed]
impl ParseArgument for OwnedFd {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        // `pop_front` as per `AncillaryBuffer::file_descriptors` instructions.
        body.fd_buffer.pop_front().ok_or(ArgumentDecodeError::MissingFd)
    }
}

#[sealed::sealed]
impl MarshalArgument for Vec<u8> {
    fn marshal(self, body: &mut ArgumentBody<'_>) {
        let length = self.len();

        // FIXME: Handle potential overflow, ~4.2 GB
        // is not entirely unfeasable to create.
        body.bytes.put_u32_ne(length as u32);
        body.bytes.extend_from_slice(&self);
        body.bytes.put_bytes(0, padding(length));
    }
}

#[sealed::sealed]
impl ParseArgument for Vec<u8> {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        let length = body.bytes.get_u32_ne() as usize;
        parse_vec_impl(length, body)
    }
}

fn parse_vec_impl(length: usize, body: &mut ArgumentBody<'_>) -> Result<Vec<u8>, ArgumentDecodeError> {
    let padding = padding(length);

    if body.bytes.len() < length + padding {
        return Err(ArgumentDecodeError::NotEnoughBytesLeft);
    }

    let vec = body.bytes.split_to(length).to_vec();

    body.bytes.advance(padding);

    Ok(vec)
}

const fn padding(length: usize) -> usize {
    let padding = WORD_SIZE - (length % WORD_SIZE);

    if padding == WORD_SIZE {
        return 0;
    }

    padding
}

#[sealed::sealed]
impl MarshalArgument for String {
    fn marshal(self, body: &mut ArgumentBody<'_>) {
        let mut vec = Vec::from(self);
        vec.push(b'\0');
        vec.marshal(body);
    }
}

#[sealed::sealed]
impl ParseArgument for String {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        Option::<String>::parse(body)?.ok_or(ArgumentDecodeError::MissingString)
    }
}

#[sealed::sealed]
impl MarshalArgument for Option<String> {
    fn marshal(self, body: &mut ArgumentBody<'_>) {
        match self {
            Some(str) => str.marshal(body),
            None => 0u32.marshal(body),
        }
    }
}

#[sealed::sealed]
impl ParseArgument for Option<String> {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        let length = u32::parse(body)? as usize;

        match length == 0 {
            true => Ok(None),
            false => {
                let mut vec = parse_vec_impl(length, body)?;

                // For null-byte.
                vec.pop();

                String::from_utf8(vec).map(Some).map_err(Into::into)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        assert_matches,
        os::fd::{AsRawFd, FromRawFd, IntoRawFd},
    };

    use super::*;

    #[test]
    fn u32_encoding() {
        assert_bijective_encoding(123u32);
    }

    #[test]
    fn i32_encoding() {
        assert_bijective_encoding(41i32);
        assert_bijective_encoding(-41i32);
    }

    #[test]
    fn fixed_encoding() {
        let fixed = Fixed { integer: i24::i24!(123), decimal: 4 };
        assert_bijective_encoding(fixed);
    }

    #[test]
    fn object_id_encoding() {
        assert_bijective_encoding(mock_id());
    }

    #[test]
    fn optional_object_id_encoding() {
        assert_bijective_encoding(Some(mock_id()));
        assert_bijective_encoding(Option::<String>::None);
    }

    #[test]
    fn new_object_id_encoding() {
        let new_id = NewObjectId::<Client, ()>::new(mock_id());
        assert_bijective_encoding(new_id);
    }

    #[test]
    fn opaque_new_object_id_encoding() {
        let new_id = OpaqueNewObjectId::<Client> {
            interface_name: "wl_xxx".to_string(),
            interface_version: 1,
            inner: mock_id(),
        };
        assert_bijective_encoding(new_id);
    }

    #[test]
    fn fd_fifo_encoding() {
        let mut bytes = BytesMut::new();
        let mut fd_buffer = VecDeque::new();
        let mut body = ArgumentBody { bytes: &mut bytes, fd_buffer: &mut fd_buffer };

        // SAFETY: fds not used for any syscalls
        let (fd_0, fd_1) = unsafe { (OwnedFd::from_raw_fd(1), OwnedFd::from_raw_fd(2)) };

        let raw_0 = fd_0.as_raw_fd();
        let raw_1 = fd_1.as_raw_fd();

        fd_0.marshal(&mut body);
        fd_1.marshal(&mut body);

        let returned_fd_0 = <OwnedFd as ParseArgument>::parse(&mut body).unwrap();
        let returned_fd_1 = <OwnedFd as ParseArgument>::parse(&mut body).unwrap();

        assert_eq!([raw_0, raw_1], [returned_fd_0.as_raw_fd(), returned_fd_1.as_raw_fd()]);

        // To avoid close on drop.
        let _ = returned_fd_0.into_raw_fd();
        let _ = returned_fd_1.into_raw_fd();
    }

    #[test]
    fn vec_encoding() {
        assert_bijective_encoding(vec![]);
        assert_bijective_encoding(vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn vec_padding() {
        let mut bytes = BytesMut::new();
        let mut fd_buffer = VecDeque::new();
        let mut body = ArgumentBody { bytes: &mut bytes, fd_buffer: &mut fd_buffer };

        let vec = vec![9];

        assert_ne!(WORD_SIZE, vec.len());

        vec.marshal(&mut body);

        let expected = &[
            1, 0, 0, 0, // length
            9, 0, 0, 0, // value + padding
        ];
        assert_eq!(expected, &body.bytes[..]);
    }

    #[test]
    fn string_encoding() {
        assert_bijective_encoding("Löwe 老虎 Léopard".to_string());
    }

    #[test]
    fn optional_string_encoding() {
        assert_bijective_encoding(Some("🦀".to_string()));
        assert_bijective_encoding(Option::<String>::None);
    }

    #[test]
    fn string_padding() {
        let mut bytes = BytesMut::new();
        let mut fd_buffer = VecDeque::new();
        let mut body = ArgumentBody { bytes: &mut bytes, fd_buffer: &mut fd_buffer };

        "ab".to_string().marshal(&mut body);

        let expected = [
            3, 0, 0, 0, // length
            b'a', b'b', b'\0', 0, // string + padding
        ];

        assert_eq!(&expected, &body.bytes[..])
    }

    #[test]
    fn string_len_overflow_err() {
        let mut bytes = BytesMut::from_iter([
            5, 0, 0, 0, // length
            b'a', 0, 0, 0,
        ]);
        let mut fd_buffer = VecDeque::new();
        let mut body = ArgumentBody { bytes: &mut bytes, fd_buffer: &mut fd_buffer };

        let err = String::parse(&mut body).unwrap_err();

        assert_matches!(err, ArgumentDecodeError::NotEnoughBytesLeft);
    }

    #[test]
    fn optional_string_none_bytes() {
        let mut bytes = BytesMut::new();
        let mut fd_buffer = VecDeque::new();
        let mut body = ArgumentBody { bytes: &mut bytes, fd_buffer: &mut fd_buffer };

        Option::<String>::None.marshal(&mut body);

        assert_eq!(&[0, 0, 0, 0], &body.bytes[..])
    }

    fn mock_id() -> ObjectId<Client> {
        ObjectId::<Client>::from_raw(1).unwrap().unwrap()
    }

    fn assert_bijective_encoding<T>(value: T)
    where
        T: MarshalArgument + ParseArgument + std::fmt::Debug + PartialEq + Clone,
    {
        let mut bytes = BytesMut::new();
        let mut fd_buffer = VecDeque::new();

        let mut body = ArgumentBody { bytes: &mut bytes, fd_buffer: &mut fd_buffer };

        value.clone().marshal(&mut body);

        let output_value = ParseArgument::parse(&mut body).unwrap();

        assert_eq!(value, output_value);

        assert!(body.bytes.is_empty(), "remaining bytes found in buffer")
    }
}

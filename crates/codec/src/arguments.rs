use std::{collections::VecDeque, marker::PhantomData, os::fd::OwnedFd};

use bytes::{Buf, BufMut, BytesMut};

use crate::*;

const WORD_SIZE: usize = std::mem::size_of::<u32>();

/// A bag for placing and removing arguments from.
pub struct ArgumentBag<'a> {
    pub(crate) bytes: &'a mut BytesMut,
    pub(crate) fd_buffer: &'a mut VecDeque<OwnedFd>,
}

/// Convert primitive arguments to raw message bytes.
///
/// Trait is sealed so that and can not be implemented on third-party types. Lowering to primitive
/// arguments is instead done in [`EncodeMessage`].
#[sealed::sealed]
pub trait MarshalArgument: Sized {
    #[allow(missing_docs)]
    fn marshal(self, bag: &mut ArgumentBag<'_>);
}

/// Convert raw message bytes to primitive arguments.
///
/// Trait is sealed so that and can not be implemented on third-party types. Parsing to higher-level
/// types (enums, bitflags etc.) is instead done in [`DecodeMessage`].
#[sealed::sealed]
pub trait ParseArgument: Sized {
    #[allow(missing_docs)]
    fn parse(bag: &mut ArgumentBag<'_>) -> Result<Self, ArgumentDecodeError>;
}

#[derive(Debug, thiserror::Error)]
pub enum ArgumentDecodeError {
    #[error("Not enough bytes left in argument buffer in order to decode the next argument.")]
    NotEnoughBytesLeft,
    #[error("Unable to convert message bytes to an UTF-8 string: {0}")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("Expected a string, received null.")]
    MissingString,
    #[error("Expected an object id, received null.")]
    MissingObjectId,
    #[error("No file descriptor found.")]
    MissingFd,
}

#[sealed::sealed]
impl MarshalArgument for u32 {
    fn marshal(self, bag: &mut ArgumentBag<'_>) {
        bag.bytes.put_u32_ne(self);
    }
}

#[sealed::sealed]
impl ParseArgument for u32 {
    fn parse(bag: &mut ArgumentBag<'_>) -> Result<Self, ArgumentDecodeError> {
        bag.bytes.try_get_u32_ne().map_err(|_| ArgumentDecodeError::NotEnoughBytesLeft)
    }
}

#[sealed::sealed]
impl MarshalArgument for i32 {
    fn marshal(self, bag: &mut ArgumentBag<'_>) {
        bag.bytes.put_i32_ne(self);
    }
}

#[sealed::sealed]
impl ParseArgument for i32 {
    fn parse(bag: &mut ArgumentBag<'_>) -> Result<Self, ArgumentDecodeError> {
        bag.bytes.try_get_i32_ne().map_err(|_| ArgumentDecodeError::NotEnoughBytesLeft)
    }
}

#[sealed::sealed]
impl MarshalArgument for Fixed {
    fn marshal(self, bag: &mut ArgumentBag<'_>) {
        let Fixed { integer, decimal } = self;

        let integer_bytes = integer.to_be_bytes();

        u32::from_be_bytes([integer_bytes[0], integer_bytes[1], integer_bytes[2], decimal]).marshal(bag);
    }
}

#[sealed::sealed]
impl ParseArgument for Fixed {
    fn parse(bag: &mut ArgumentBag<'_>) -> Result<Self, ArgumentDecodeError> {
        let inner = u32::parse(bag)?.to_be_bytes();

        let fixed = Fixed {
            integer: i24::I24::from_be_bytes([inner[0], inner[1], inner[2]]),
            decimal: inner[3],
        };

        Ok(fixed)
    }
}

#[sealed::sealed]
impl MarshalArgument for Option<OpaqueObjectId> {
    fn marshal(self, bag: &mut ArgumentBag<'_>) {
        self.map(|id| id.0).unwrap_or(0).marshal(bag);
    }
}

#[sealed::sealed]
impl ParseArgument for Option<OpaqueObjectId> {
    fn parse(bag: &mut ArgumentBag<'_>) -> Result<Self, ArgumentDecodeError> {
        u32::parse(bag).map(|raw| match raw == 0 {
            true => None,
            false => Some(OpaqueObjectId(raw)),
        })
    }
}

#[sealed::sealed]
impl MarshalArgument for OpaqueObjectId {
    fn marshal(self, bag: &mut ArgumentBag<'_>) {
        self.0.marshal(bag);
    }
}

#[sealed::sealed]
impl ParseArgument for OpaqueObjectId {
    fn parse(bag: &mut ArgumentBag<'_>) -> Result<Self, ArgumentDecodeError> {
        Option::<OpaqueObjectId>::parse(bag)?.ok_or(ArgumentDecodeError::MissingObjectId)
    }
}

#[sealed::sealed]
impl<I> MarshalArgument for ObjectId<I> {
    fn marshal(self, bag: &mut ArgumentBag<'_>) {
        self.inner.marshal(bag);
    }
}

#[sealed::sealed]
impl<I> ParseArgument for ObjectId<I> {
    fn parse(bag: &mut ArgumentBag<'_>) -> Result<Self, ArgumentDecodeError> {
        OpaqueObjectId::parse(bag).map(|inner| Self { inner, interface_marker: PhantomData })
    }
}

#[sealed::sealed]
impl<I> MarshalArgument for Option<ObjectId<I>> {
    fn marshal(self, bag: &mut ArgumentBag<'_>) {
        self.map(|id| id.inner).marshal(bag);
    }
}

#[sealed::sealed]
impl<I> ParseArgument for Option<ObjectId<I>> {
    fn parse(bag: &mut ArgumentBag<'_>) -> Result<Self, ArgumentDecodeError> {
        let opaque = Option::<OpaqueObjectId>::parse(bag)?;
        let concrete = opaque.map(|inner| ObjectId { inner, interface_marker: PhantomData });
        Ok(concrete)
    }
}

#[sealed::sealed]
impl<I> MarshalArgument for NewObjectId<I> {
    fn marshal(self, bag: &mut ArgumentBag<'_>) {
        self.0.marshal(bag);
    }
}

#[sealed::sealed]
impl<I> ParseArgument for NewObjectId<I> {
    fn parse(bag: &mut ArgumentBag<'_>) -> Result<Self, ArgumentDecodeError> {
        ObjectId::parse(bag).map(Self)
    }
}

#[sealed::sealed]
impl MarshalArgument for OpaqueNewObjectId {
    fn marshal(self, bag: &mut ArgumentBag<'_>) {
        let OpaqueNewObjectId { interface_name, interface_version, inner } = self;

        interface_name.marshal(bag);
        interface_version.marshal(bag);
        inner.marshal(bag);
    }
}

#[sealed::sealed]
impl ParseArgument for OpaqueNewObjectId {
    fn parse(bag: &mut ArgumentBag<'_>) -> Result<Self, ArgumentDecodeError> {
        let interface_name = String::parse(bag)?;
        let interface_version = u32::parse(bag)?;
        let inner = u32::parse(bag)?;

        Ok(Self { interface_name, interface_version, inner })
    }
}

#[sealed::sealed]
impl MarshalArgument for OwnedFd {
    fn marshal(self, bag: &mut ArgumentBag<'_>) {
        // `push_back` as per `AncillaryBuffer::file_descriptors` instructions.
        bag.fd_buffer.push_back(self);
    }
}

#[sealed::sealed]
impl ParseArgument for OwnedFd {
    fn parse(bag: &mut ArgumentBag<'_>) -> Result<Self, ArgumentDecodeError> {
        // `pop_front` as per `AncillaryBuffer::file_descriptors` instructions.
        bag.fd_buffer.pop_front().ok_or(ArgumentDecodeError::MissingFd)
    }
}

#[sealed::sealed]
impl MarshalArgument for Vec<u8> {
    fn marshal(self, bag: &mut ArgumentBag<'_>) {
        let length = self.len();

        // FIXME: Handle potential overflow, ~4.2 GB
        // is not entirely unfeasable to create.
        bag.bytes.put_u32_ne(length as u32);
        bag.bytes.extend_from_slice(&self);
        bag.bytes.put_bytes(0, padding(length));
    }
}

#[sealed::sealed]
impl ParseArgument for Vec<u8> {
    fn parse(bag: &mut ArgumentBag<'_>) -> Result<Self, ArgumentDecodeError> {
        let length = bag.bytes.get_u32_ne() as usize;
        parse_vec_impl(length, bag)
    }
}

fn parse_vec_impl(length: usize, bag: &mut ArgumentBag<'_>) -> Result<Vec<u8>, ArgumentDecodeError> {
    let padding = padding(length);

    if bag.bytes.len() < length + padding {
        return Err(ArgumentDecodeError::NotEnoughBytesLeft);
    }

    let vec = bag.bytes.split_to(length).to_vec();

    bag.bytes.advance(padding);

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
    fn marshal(self, bag: &mut ArgumentBag<'_>) {
        let mut vec = Vec::from(self);
        vec.push(b'\0');
        vec.marshal(bag);
    }
}

#[sealed::sealed]
impl ParseArgument for String {
    fn parse(bag: &mut ArgumentBag<'_>) -> Result<Self, ArgumentDecodeError> {
        Option::<String>::parse(bag)?.ok_or(ArgumentDecodeError::MissingString)
    }
}

#[sealed::sealed]
impl MarshalArgument for Option<String> {
    fn marshal(self, bag: &mut ArgumentBag<'_>) {
        match self {
            Some(str) => str.marshal(bag),
            None => 0u32.marshal(bag),
        }
    }
}

#[sealed::sealed]
impl ParseArgument for Option<String> {
    fn parse(bag: &mut ArgumentBag<'_>) -> Result<Self, ArgumentDecodeError> {
        let length = u32::parse(bag)? as usize;

        match length == 0 {
            true => Ok(None),
            false => {
                let mut vec = parse_vec_impl(length, bag)?;

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
    use crate::WaylandMessageStreamError::Opaque;

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
    fn opaque_object_id_encoding() {
        assert_bijective_encoding(OpaqueObjectId(123456));
    }

    #[test]
    fn optional_opaque_object_id_encoding() {
        assert_bijective_encoding(Some(OpaqueObjectId(123456)));
        assert_bijective_encoding(Option::<OpaqueObjectId>::None);
    }

    #[test]
    fn expected_opaque_object_id_err() {
        let mut bytes = BytesMut::new();
        let mut fd_buffer = VecDeque::new();
        let mut bag = ArgumentBag { bytes: &mut bytes, fd_buffer: &mut fd_buffer };

        0.marshal(&mut bag);

        let error = OpaqueObjectId::parse(&mut bag).unwrap_err();

        assert_matches!(error, ArgumentDecodeError::MissingObjectId);
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
        let new_id = NewObjectId::<()>(mock_id());
        assert_bijective_encoding(new_id);
    }

    #[test]
    fn opaque_new_object_id_encoding() {
        let new_id = OpaqueNewObjectId { interface_name: "wl_xxx".to_string(), interface_version: 1, inner: 1 };
        assert_bijective_encoding(new_id);
    }

    #[test]
    fn fd_fifo_encoding() {
        let mut bytes = BytesMut::new();
        let mut fd_buffer = VecDeque::new();
        let mut bag = ArgumentBag { bytes: &mut bytes, fd_buffer: &mut fd_buffer };

        // SAFETY: fds not used for any syscalls
        let (fd_0, fd_1) = unsafe { (OwnedFd::from_raw_fd(1), OwnedFd::from_raw_fd(2)) };

        let raw_0 = fd_0.as_raw_fd();
        let raw_1 = fd_1.as_raw_fd();

        fd_0.marshal(&mut bag);
        fd_1.marshal(&mut bag);

        let returned_fd_0 = <OwnedFd as ParseArgument>::parse(&mut bag).unwrap();
        let returned_fd_1 = <OwnedFd as ParseArgument>::parse(&mut bag).unwrap();

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
        let mut bag = ArgumentBag { bytes: &mut bytes, fd_buffer: &mut fd_buffer };

        let vec = vec![9];

        assert_ne!(WORD_SIZE, vec.len());

        vec.marshal(&mut bag);

        let expected = &[
            1, 0, 0, 0, // length
            9, 0, 0, 0, // value + padding
        ];
        assert_eq!(expected, &bag.bytes[..]);
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
        let mut bag = ArgumentBag { bytes: &mut bytes, fd_buffer: &mut fd_buffer };

        "ab".to_string().marshal(&mut bag);

        let expected = [
            3, 0, 0, 0, // length
            b'a', b'b', b'\0', 0, // string + padding
        ];

        assert_eq!(&expected, &bag.bytes[..])
    }

    #[test]
    fn string_len_overflow_err() {
        let mut bytes = BytesMut::from_iter([
            5, 0, 0, 0, // length
            b'a', 0, 0, 0,
        ]);
        let mut fd_buffer = VecDeque::new();
        let mut bag = ArgumentBag { bytes: &mut bytes, fd_buffer: &mut fd_buffer };

        let err = String::parse(&mut bag).unwrap_err();

        assert_matches!(err, ArgumentDecodeError::NotEnoughBytesLeft);
    }

    #[test]
    fn optional_string_none_bytes() {
        let mut bytes = BytesMut::new();
        let mut fd_buffer = VecDeque::new();
        let mut bag = ArgumentBag { bytes: &mut bytes, fd_buffer: &mut fd_buffer };

        Option::<String>::None.marshal(&mut bag);

        assert_eq!(&[0, 0, 0, 0], &bag.bytes[..])
    }

    fn mock_id() -> ObjectId<()> {
        ObjectId { inner: OpaqueObjectId(1), interface_marker: PhantomData }
    }

    fn assert_bijective_encoding<T>(value: T)
    where
        T: MarshalArgument + ParseArgument + std::fmt::Debug + PartialEq + Clone,
    {
        let mut bytes = BytesMut::new();
        let mut fd_buffer = VecDeque::new();

        let mut bag = ArgumentBag { bytes: &mut bytes, fd_buffer: &mut fd_buffer };

        value.clone().marshal(&mut bag);

        let output_value = ParseArgument::parse(&mut bag).unwrap();

        assert_eq!(value, output_value);

        assert!(bag.bytes.is_empty(), "remaining bytes found in buffer")
    }
}

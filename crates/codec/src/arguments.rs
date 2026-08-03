use std::{collections::VecDeque, os::fd::OwnedFd};

use bytes::{Buf, BytesMut};

use crate::*;

pub struct ArgumentBody<'a> {
    bytes: &'a mut BytesMut,
    fd_buffer: &'a mut VecDeque<OwnedFd>,
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
pub trait ParseArgument: Sized {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError>;
}

#[sealed::sealed]
impl ParseArgument for u32 {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        body.bytes.try_get_u32_ne().map_err(|_| ArgumentDecodeError::NotEnoughBytesLeft)
    }
}

#[sealed::sealed]
impl ParseArgument for i32 {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        body.bytes.try_get_i32_ne().map_err(|_| ArgumentDecodeError::NotEnoughBytesLeft)
    }
}

#[sealed::sealed]
impl ParseArgument for Fixed {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        let inner = <u32 as ParseArgument>::parse(body)?.to_be_bytes();

        let fixed = Fixed {
            integer: i24::I24::from_be_bytes([inner[0], inner[1], inner[2]]),
            decimal: inner[3],
        };

        Ok(fixed)
    }
}

#[sealed::sealed]
impl ParseArgument for String {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        <Option<String> as ParseArgument>::parse(body)?.ok_or(ArgumentDecodeError::MissingString)
    }
}

#[sealed::sealed]
impl ParseArgument for Option<String> {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        let length = <u32 as ParseArgument>::parse(body)? as usize;

        match length == 0 {
            true => Ok(None),
            false => {
                // FIXME: assert length before proceeding

                let padding = length % std::mem::size_of::<u32>();

                // -1 for null terminator
                let string_bytes = body.bytes.split_to(length - 1);

                let string = String::from_utf8(string_bytes.to_vec())?;

                // +1 for null terminator
                body.bytes.advance(1 + padding);

                Ok(Some(string))
            }
        }
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
impl<E: ObjectIdBounds> ParseArgument for ObjectId<E> {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        <Option<ObjectId<E>> as ParseArgument>::parse(body)?.ok_or(ArgumentDecodeError::MissingObjectId)
    }
}

#[sealed::sealed]
impl<E: ObjectIdBounds, I> ParseArgument for NewObjectId<E, I> {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        <ObjectId<E> as ParseArgument>::parse(body).map(NewObjectId::new)
    }
}

#[sealed::sealed]
impl<E: ObjectIdBounds> ParseArgument for OpaqueNewObjectId<E> {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        let interface_name = <String as ParseArgument>::parse(body)?;
        let interface_version = <u32 as ParseArgument>::parse(body)?;
        let inner = <ObjectId<E> as ParseArgument>::parse(body)?;

        Ok(Self { interface_name, interface_version, inner })
    }
}

#[sealed::sealed]
impl ParseArgument for OwnedFd {
    fn parse(body: &mut ArgumentBody<'_>) -> Result<Self, ArgumentDecodeError> {
        // `pop_front` as per `AncillaryRead::buffer` instructions.
        body.fd_buffer.pop_front().ok_or(ArgumentDecodeError::MissingFd)
    }
}

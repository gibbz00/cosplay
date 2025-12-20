use bytes::{Buf, BufMut, BytesMut};

use crate::*;

#[sealed::sealed]
pub trait ArgumentEncode {
    /// Size which needs to be added to the header size.
    fn size(&self) -> usize;
    fn encode(&self, dst: &mut bytes::BytesMut);
}

/// An argument of type `T` will have an `ArgumentDecoder<T>` assigned
/// by the generator. Two things need to be declared in order for that
/// to work:
///
/// 1. An [`ArgumentDecoderState`] needs to be implemented for `T`.
/// 2. An [`ArgumentDecode`] needs to be implemented for `ArgumentDecoder<T>`
#[derive(Default)]
pub struct ArgumentDecoder<T: ArgumentDecoderState> {
    state: T::State,
}

/// Type association trait is used to remove the need for the generator
/// to map a decoder state for a given argument type. A decoder for
/// an `String` argument is therefore `ArgumentDecoder<String>` rather than
/// something like `ArgumentDecoder<String, StringDecoderState>`.
#[sealed::sealed]
pub trait ArgumentDecoderState {
    type State;
}

#[sealed::sealed]
pub trait ArgumentDecode {
    type Item;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, MessageDecoderError>;
}

#[sealed::sealed]
impl ArgumentEncode for u32 {
    fn size(&self) -> usize {
        std::mem::size_of::<Self>()
    }

    fn encode(&self, dst: &mut bytes::BytesMut) {
        dst.put_u32_ne(*self);
    }
}

#[sealed::sealed]
impl ArgumentDecoderState for u32 {
    type State = ();
}

#[sealed::sealed]
impl ArgumentDecode for ArgumentDecoder<u32> {
    type Item = u32;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<u32>, MessageDecoderError> {
        Ok(src.try_get_u32_ne().ok())
    }
}

#[sealed::sealed]
impl ArgumentEncode for String {
    fn size(&self) -> usize {
        // length for string length, length of string, +1 for null terminator
        std::mem::size_of::<u32>() + self.len() + 1
    }

    fn encode(&self, dst: &mut bytes::BytesMut) {
        // +1 for null terminator
        let length = self.len() + 1;

        // Cast should be ok, potential overflows `FullMessageEncoder::encode` from `Message::size()`.
        dst.put_u32_ne(length as u32);

        dst.put_slice(self.as_bytes());
        dst.put_u8(b'\0');

        let padding = length % std::mem::size_of::<u32>();
        dst.put_bytes(0, padding);
    }
}

#[sealed::sealed]
impl ArgumentDecoderState for String {
    // Some(usize) represents decoded string length, null terminator included.
    type State = Option<usize>;
}

#[sealed::sealed]
impl ArgumentDecode for ArgumentDecoder<String> {
    type Item = String;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<String>, MessageDecoderError> {
        match self.state {
            None => match src.try_get_u32_ne() {
                Ok(length) => {
                    self.state = Some(length as usize);
                    self.decode(src)
                }
                Err(_) => Ok(None),
            },
            Some(length) => {
                let padding = length % std::mem::size_of::<u32>();

                if src.len() < length + padding {
                    return Ok(None);
                }

                // -1 for null terminator
                let string_bytes = src.copy_to_bytes(length - 1);

                let string = String::from_utf8(string_bytes.to_vec())?;

                // +1 for null terminator
                src.advance(1 + padding);

                Ok(Some(string))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u32_encoding() {
        assert_bijective_encoding(123u32);
    }

    #[test]
    fn string_encoding() {
        let string = "Löwe 老虎 Léopard".to_string();
        assert_bijective_encoding(string);
    }

    #[test]
    fn string_padding() {
        let string = "a".to_string();

        let mut buffer = BytesMut::new();
        string.encode(&mut buffer);

        let expected = [
            2, 0, 0, 0, // length
            b'a', b'\0', 0, 0, // string + padding
        ];
        assert_eq!(&expected, buffer.to_vec().as_slice())
    }

    fn assert_bijective_encoding<T>(value: T)
    where
        T: ArgumentEncode + ArgumentDecoderState + std::fmt::Debug + PartialEq,
        ArgumentDecoder<T>: Default + ArgumentDecode<Item = T>,
    {
        let mut buffer = BytesMut::new();

        value.encode(&mut buffer);

        let output_value = ArgumentDecoder::<T>::default()
            .decode(&mut buffer)
            .expect("decoder errored")
            .expect("insufficient bytes in buffer");

        assert_eq!(value, output_value);

        assert!(buffer.is_empty(), "remaining bytes found in buffer")
    }
}

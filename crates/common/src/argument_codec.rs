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
pub struct ArgumentDecoder<T: ArgumentDecoderState> {
    state: T::State,
}

impl<T: ArgumentDecoderState> Default for ArgumentDecoder<T>
where
    T::State: Default,
{
    fn default() -> Self {
        Self { state: Default::default() }
    }
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

macro_rules! stateless_decoder_impl {
    ($type:ty) => {
        #[sealed::sealed]
        impl ArgumentDecoderState for $type {
            type State = ();
        }
    };
}

macro_rules! num_impl {
    ($num:ty) => {
        #[sealed::sealed]
        impl ArgumentEncode for $num {
            fn size(&self) -> usize {
                std::mem::size_of::<Self>()
            }

            fn encode(&self, dst: &mut bytes::BytesMut) {
                paste::paste! {
                    dst.[<put_ $num _ne>](*self)
                }
            }
        }

        stateless_decoder_impl!($num);
        #[sealed::sealed]
        impl ArgumentDecode for ArgumentDecoder<$num> {
            type Item = $num;

            fn decode(&mut self, src: &mut BytesMut) -> Result<Option<$num>, MessageDecoderError> {
                let item = paste::paste! {
                    src.[<try_get_ $num _ne>]().ok()
                };
                Ok(item)
            }
        }
    };
}

num_impl!(u32);
num_impl!(i32);

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
            Some(length) => decode_string_impl(length, src),
        }
    }
}

#[sealed::sealed]
impl ArgumentEncode for Option<String> {
    fn size(&self) -> usize {
        self.as_ref()
            .map(String::size)
            // length byte only
            .unwrap_or(std::mem::size_of::<u32>())
    }

    fn encode(&self, dst: &mut bytes::BytesMut) {
        match self {
            Some(str) => str.encode(dst),
            None => dst.put_u32_ne(0),
        }
    }
}

#[sealed::sealed]
impl ArgumentDecoderState for Option<String> {
    type State = Option<usize>;
}

#[sealed::sealed]
impl ArgumentDecode for ArgumentDecoder<Option<String>> {
    type Item = Option<String>;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Option<String>>, MessageDecoderError> {
        match self.state {
            None => match src.try_get_u32_ne() {
                Ok(length) => {
                    if length == 0 {
                        return Ok(Some(None));
                    }

                    self.state = Some(length as usize);
                    self.decode(src)
                }
                Err(_) => Ok(None),
            },
            Some(length) => decode_string_impl(length, src).map(Some),
        }
    }
}

fn decode_string_impl(length: usize, src: &mut BytesMut) -> Result<Option<String>, MessageDecoderError> {
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

/// Signed 24.8 decimal numbers. It is a signed decimal type which offers a
/// sign bit, 23 bits of integer precision and 8 bits of decimal precision.
/// Conversions from i32 and f64 mimic those done by
/// [libwayland](libwayland_impl).
///
/// [libwayland_impl]: https://gitlab.freedesktop.org/wayland/wayland/-/blob/99638501a1314e68c79176fa2cafa3bbe6cf55ea/src/wayland-util.h#L621-673
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fixed(i32);

impl Fixed {
    pub const fn from_f64(f: f64) -> Self {
        Self((f * 256.0).round() as i32)
    }

    pub const fn as_f64(&self) -> f64 {
        self.0 as f64 / 256.0
    }

    /// # Panics
    ///
    /// If `i * 256` overflows.
    pub const fn from_i32(i: i32) -> Self {
        Self(i * 256)
    }

    pub const fn as_i32(&self) -> i32 {
        self.0 / 256
    }
}

#[sealed::sealed]
impl ArgumentEncode for Fixed {
    fn size(&self) -> usize {
        self.0.size()
    }

    fn encode(&self, dst: &mut bytes::BytesMut) {
        self.0.encode(dst);
    }
}

stateless_decoder_impl!(Fixed);
#[sealed::sealed]
impl ArgumentDecode for ArgumentDecoder<Fixed> {
    type Item = Fixed;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Fixed>, MessageDecoderError> {
        let inner = src.try_get_i32_ne().ok().map(Fixed);
        Ok(inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn num_encoding() {
        assert_bijective_encoding(123u32);
        assert_bijective_encoding(-41i32);
    }

    #[test]
    fn string_encoding() {
        let string = "Löwe 老虎 Léopard".to_string();
        assert_bijective_encoding(string);
    }

    #[test]
    fn optional_string_encoding() {
        let string = "🦀".to_string();
        assert_bijective_encoding(Some(string));
        assert_bijective_encoding(None);
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

    #[test]
    fn optional_string_none_bytes() {
        let string = Option::<String>::None;

        let mut buffer = BytesMut::new();
        string.encode(&mut buffer);

        assert_eq!(&[0, 0, 0, 0], buffer.to_vec().as_slice())
    }

    #[test]
    fn fixed_encoding() {
        let fixed = Fixed::from_i32(123);
        assert_bijective_encoding(fixed);
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

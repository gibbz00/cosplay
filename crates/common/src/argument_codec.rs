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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u32() {
        assert_bijective_encoding(123u32);
    }

    fn assert_bijective_encoding<T>(value: T)
    where
        T: ArgumentEncode + ArgumentDecoderState + std::fmt::Debug + PartialEq,
        ArgumentDecoder<T>: Default + ArgumentDecode<Item = T>,
    {
        let mut buffer = BytesMut::new();

        value.encode(&mut buffer);

        let output_value = ArgumentDecoder::<T>::default().decode(&mut buffer).unwrap().unwrap();

        assert_eq!(value, output_value);
    }
}

use bytes::BytesMut;

use crate::*;

pub trait ArgumentEncode {
    /// Size which needs to be added to the header size.
    fn size(&self) -> usize;
    fn encode(&self, dst: &mut bytes::BytesMut);
}

/// An argument of type `T` will have an `ArgumentDecoder<T>` assigned
/// by the generator. Two things need to be declared in order for that
/// to work:
///
/// 1. An `ArgumentDecoderState` needs to be implemented for `T`.
/// 2. A "overloaded" method to Self needs to be added for the concrete type `T` with following
///    signature: `fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<T>,
///    MessageArgumentDecoderError>`.
///
/// A sealed trait could technically be used for a less error-prone step
/// two, but the current method should hopefully lead to simpler code.
#[derive(Default)]
pub struct ArgumentDecoder<T: ArgumentDecoderState> {
    state: T::State,
}

/// Type association trait is used to remove the need for the generator
/// to map a decoder state for a given argument type. A decoder for
/// an `String` argument is therefore `ArgumentDecoder<String>` rather than
/// something like `ArgumentDecoder<String, StringDecoderState>`.
pub trait ArgumentDecoderState {
    type State;
}

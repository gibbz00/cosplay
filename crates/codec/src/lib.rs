// TEMP:
#![allow(missing_docs)]

//! # `async-wayland-codec` - Opaque and typed wire format encoding.

// Layer 1. Bytes <--> Opaque Frame

mod opaque;
pub use opaque::{OpaqueFrame, OpaqueFrameDecodeError, OpaqueFrameDecoder, OpaqueFrameEncoder};

// Layer 2. Opaque Frame <--> Arguments

mod arguments;
pub use arguments::{ArgumentBag, ArgumentDecodeError, MarshalArgument, ParseArgument};

mod object_id;
pub(crate) use object_id::*;

mod fixed;
pub(crate) use fixed::Fixed;

mod entity;
pub(crate) use entity::{Client, Entity, Server};

// Layer 3. Arguments <--> Rust Structs

mod message;
pub use message::{DecodeMessage, DecodeMessageError, EncodeMessage, Message, OpaqueMessage};

// Combines layer 1 to 3 into one cohesive API.

mod stream;
pub use stream::{WaylandMessageSink, WaylandMessageStream, WaylandMessageStreamError};

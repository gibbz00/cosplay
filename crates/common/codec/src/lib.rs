//! # `async-wayland-codec` - Opaque and typed wire format encoding.

// Layer 1. Bytes <--> Opaque Frame

mod opaque;
pub use opaque::OpaqueFrameDecodeError;
pub(crate) use opaque::{OpaqueFrame, OpaqueFrameDecoder, OpaqueFrameEncoder};

// Layer 2. Opaque Frame <--> Arguments

mod arguments;
pub use arguments::{ArgumentBag, ArgumentDecodeError, MarshalArgument, ParseArgument};

mod object_id;
pub(crate) use object_id::*;
pub use object_id::{NewObjectId, ObjectId, OpaqueNewObjectId, OpaqueObjectId};

mod fixed;
pub use fixed::Fixed;

mod entity;
pub(crate) use entity::{Client, Entity, Server};

// Layer 3. Arguments <--> Rust Structs

mod enumeration;
pub use enumeration::Enumeration;

mod message;
pub use message::{DecodeMessage, DecodeMessageError, EncodeMessage, Message, OpaqueMessage};

// Combines layer 1 to 3 into one cohesive API.

mod stream;
pub use stream::{WaylandMemoryBuffer, WaylandMessageSink, WaylandMessageStream, WaylandMessageStreamError};

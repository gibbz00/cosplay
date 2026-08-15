//! # `cosplay-codec` - Opaque and typed wire format encoding.

// Layer 1. Bytes <--> Opaque Frame

mod opaque;
pub use opaque::OpaqueFrameDecodeError;
pub(crate) use opaque::{OpaqueFrame, OpaqueFrameDecoder, OpaqueFrameEncoder};

// Layer 2. Opaque Frame <--> Arguments

mod arguments;
pub use arguments::{ArgumentBag, ArgumentDecodeError, MarshalArgument, ParseArgument};

mod object_id;
pub use object_id::{NewObjectId, ObjectId, OpaqueNewObjectId, OpaqueObjectId};

mod fixed;
pub use fixed::Fixed;

// Layer 3. Interfaces, Messages and, Arguments <--> Rust Structs

mod enumeration;
pub(crate) use enumeration::EnumRepr;
pub use enumeration::{EnumArg, Enumeration};

mod message;
pub use message::{DecodeMessage, DecodeMessageError, EncodeMessage, Message, OpaqueMessage};

mod interface;
pub use interface::Interface;

// Combines layer 1 to 3 into one cohesive API.

mod stream;
pub use stream::{WaylandMemoryBuffer, WaylandMessageSink, WaylandMessageStream, WaylandMessageStreamError};

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

mod enumeration;
pub(crate) use enumeration::EnumRepr;
pub use enumeration::{EnumArg, Enumeration};

// Layer 3. Interfaces and Messages <--> Rust Structs

mod message;
pub use message::{DecodeMessage, EncodeMessage, Message};

mod opaque_message;
pub use opaque_message::{OpapueMessageMismatchError, OpaqueMessage};

mod interface;
pub use interface::Interface;

mod inbound;
pub use inbound::{Direction, Event, Inbound, IntoInboundError, Request};

// Combines layer 1 to 3 into one cohesive API.

mod stream;
pub use stream::{WaylandMemoryBuffer, WaylandMessageSink, WaylandMessageStream};

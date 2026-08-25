//! # `cosplay-generator` - Generate `cosplay-codec` trait implementations from XML protocol definitions.
//!
//! ## Features
//!
//! (None are enabled by default.)
//!
//! - `serde`: Implements `serde::de::Deserialize` for [`GeneratorConfig`].

mod core;
pub use core::Generator;

mod protocol;
pub(crate) use protocol::ProtocolItem;

mod interface;
pub(crate) use interface::{InterfaceContext, InterfaceItem};

mod message;
pub(crate) use message::{MessageContext, MessageItem, MessageType};

mod enumeration;
pub(crate) use enumeration::{EnumContext, EnumItem};

mod documentation;
pub(crate) use documentation::DocumentationItem;

mod identifier;
pub(crate) use identifier::IdentifierItem;

pub mod config;
pub(crate) use config::*;

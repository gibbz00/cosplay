//! # `async-wayland-generator` - Generate `async-wayland-codec` trait implementations from XML protocol definitions.
//!
//! ## Features
//!
//! (None are enabled by default.)
//!
//! - `serde`: Implements `serde::de::Deserialize` for [`GeneratorConfig`].

mod core;
pub use core::{Generator, GeneratorError};

mod protocol;
pub(crate) use protocol::ProtocolItem;

mod interface;
pub(crate) use interface::{InterfaceContext, InterfaceModule};

mod message;
pub(crate) use message::{MessageContext, MessageItem};

mod enumeration;
pub(crate) use enumeration::{EnumContext, EnumItem};

mod documentation;
pub(crate) use documentation::Documentation;

mod formatting;
pub(crate) use formatting::Formatter;

mod identifier;
pub(crate) use identifier::IdentifierItem;

pub mod config;
pub(crate) use config::{GeneratorConfig, ItemType, NameMappings};

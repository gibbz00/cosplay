//! # `async-wayland-generator` - Generate `async-wayland-codec` trait implementations from XML protocol definitions.

mod core;
pub use core::Generator;

mod interface;
pub(crate) use interface::InterfaceModule;

mod message;
pub(crate) use message::MessageItem;

mod documentation;
pub(crate) use documentation::Documentation;

mod formatting;
pub(crate) use formatting::Formatter;

mod identifier;
pub(crate) use identifier::IdentifierItem;

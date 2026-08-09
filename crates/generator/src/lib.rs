//! # `async-wayland-generator` - Generate `async-wayland-codec` trait implementations from XML protocol definitions.

mod documentation;
pub(crate) use documentation::Documentation;

mod formatting;
pub(crate) use formatting::Formatter;

mod interface;
pub(crate) use interface::InterfaceModule;

mod core;
pub use core::Generator;

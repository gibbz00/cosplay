//! # `async-wayland-scanner` - Generate codec implementations from wayland XML specifications.
//!
//! Type declarations for deserializing wayland XML protocol declarations.
//!
//! Check out [Message_XML] for further information on the wayland XML document structure.
//!
//! [Message_XML]: https://wayland.freedesktop.org/docs/book/Message_XML.html

mod protocol;
pub(crate) use protocol::Protocol;

mod interface;
pub(crate) use interface::Interface;

mod message;
pub(crate) use message::Message;

mod enumeration;
pub(crate) use enumeration::{Entry, Enum, EnumPath};

mod arg;
pub(crate) use arg::{Argument, ArgumentVariant};

mod description;
pub(crate) use description::Description;

mod version;
pub(crate) use version::Version;

mod cname;
pub(crate) use cname::{Cname, CnameSuffix};

//! # `async-wayland-xml` - Items for deserializing Wayland XML protocol declarations.
//!
//! Intended to be usable by different implementations of wayland protocol code generators, with
//! [`Protocol::from_xml`] being the starting point.
//!
//! ### Correctness
//!
//! The deserializer conforms to most conventions described <https://wayland.freedesktop.org/docs/book/Message_XML.html>.
//!
//! Exceptions in validity checking revolve around asserting "cross element" invariants. For
//! example:
//!
//! * Uniqueness checking of [`Cname`]s. Be it within interfaces, requests, or arguments etc.
//!
//! * "since" and "deprecated since" values are not verified against the corresponding interface
//!   version.
//!
//! * An i32 argument type referencing an enum is not verified to not be pointing to a bitfield enum
//!   declaration.

mod protocol;
pub use protocol::Protocol;

mod interface;
pub use interface::Interface;

mod message;
pub use message::Message;

mod argument;
pub use argument::{Argument, ArgumentVariant};

mod enumeration;
pub use enumeration::{Enum, EnumEntry, EnumPath};

mod description;
pub use description::Description;

mod version;
pub use version::Version;

mod cname;
pub use cname::{Cname, CnameParseError, CnameSuffix, CnameSuffixParseError};

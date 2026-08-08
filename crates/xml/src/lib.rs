//! # `async-wayland-xml` - Items for deserializing Wayland XML protocol declarations.
//!
//! Intended to be usable by multiple implementations of wayland protocol code generators.
//!
//! Based on `serde` and `quick-xml`. Invoke the combination of both with [`Protocol::from_xml`].
//!
//! Most item and field documentation has been adapted from <https://wayland.freedesktop.org/docs/book/Message_XML.html>.

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

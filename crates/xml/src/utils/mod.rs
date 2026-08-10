//! Generator-agnostic utilities for inspecting the [`Protocol`](crate::Protocol) struct.

mod enumeration;
pub use enumeration::{EnumRepr, EnumReprMap, EnumReprMapBuildError};

use crate::*;

/// Interface declaration defined under a [`Protocol`].
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct Interface {
    /// Top-level interface name.
    ///
    /// Protocols to be included in `wayland-protocols` must follow the interface naming rules set
    /// there. Other protocols should use a unique prefix for the name, e.g. referring to the owning
    /// project’s name.
    #[serde(rename = "@name")]
    pub name: Cname,
    /// Current interface version.
    #[serde(rename = "@version")]
    pub version: Version,
    /// If set the interface is frozen and forever stuck at version 1.
    ///
    /// This attribute should be applied to interfaces that have multiple parent interfaces with
    /// independent ancestor global interfaces, for example wl_buffer and wl_callback.
    #[serde(rename = "@frozen", default)]
    pub frozen: bool,
    /// Optional interface description.
    pub description: Option<Description>,
    /// Collection of interface requests.
    #[serde(rename = "request", default)]
    pub requests: Vec<Message>,
    /// Collection of interface events.
    #[serde(rename = "event", default)]
    pub events: Vec<Message>,
    /// Collection of interface enumerations.
    #[serde(rename = "enum", default)]
    pub enums: Vec<Enum>,
}

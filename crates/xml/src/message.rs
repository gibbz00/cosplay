use serde::{Deserialize, de::Unexpected};

use crate::*;

/// A message representing either and interface request ([`Interface::requests`]) or event
/// ([`Interface::events`]).
///
/// Message opcodes are assigned in the order they appear the respective lists
/// in [`Interface`], or in other words; the corresponding list index values.
///
/// Therefore the only backwards-compatible way to add requests to an interface is to add
/// them to the end.
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct Message {
    /// The name of the request or event.
    #[serde(rename = "@name")]
    pub name: Cname,
    /// When this attribute is present, it shall destroy the protocol object it is sent on.
    ///
    /// Protocol IPC libraries may use this for bookkeeping protocol object lifetimes.
    ///
    /// Libwayland-client uses this information to ignore incoming events for destroyed protocol
    /// objects. Such events may occur due to a natural race condition between the client destroying
    /// a protocol object and the server sending events before processing the destroy request.
    #[serde(flatten, deserialize_with = "is_destructor")]
    pub destructor: bool,
    /// Defines at which [`Interface::version`] the message was added.
    #[serde(rename = "@since", default)]
    pub since: Version,
    /// Defines if and at which [`Interface::version`] the message was marked as deprecated.
    #[serde(rename = "@deprecated-since")]
    pub deprecated_since: Option<Version>,
    /// Optional message description.
    pub description: Option<Description>,
    /// List of arguments passed over the wire for a given message.
    #[serde(rename = "arg", default)]
    pub arguments: Vec<Argument>,
}

fn is_destructor<'de, D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    #[derive(Deserialize)]
    struct Proxy {
        #[serde(rename = "@type")]
        inner: Option<String>,
    }

    const TYPE_STR: &str = "destructor";

    let Some(string) = Proxy::deserialize(deserializer)?.inner else {
        return Ok(false);
    };

    match string.contains(TYPE_STR) {
        true => Ok(true),
        false => Err(serde::de::Error::invalid_value(Unexpected::Str(&string), &TYPE_STR)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_defaults() -> Message {
        Message {
            name: Cname("req".to_string()),
            destructor: false,
            since: Version::default(),
            deprecated_since: None,
            description: None,
            arguments: Vec::new(),
        }
    }

    #[test]
    fn defaults() {
        let xml = "<request name=\"req\" />";

        let actual = quick_xml::de::from_str(xml).unwrap();

        let expected = mock_defaults();

        assert_eq!(expected, actual);
    }

    #[test]
    fn destructor_ok() {
        let xml = "<request name=\"req\" type=\"destructor\" />";

        let actual = quick_xml::de::from_str(xml).unwrap();

        let expected = Message { destructor: true, ..mock_defaults() };

        assert_eq!(expected, actual);
    }

    #[test]
    fn destructor_err() {
        let xml = "<request name=\"req\" type=\"other\" />";
        assert!(quick_xml::de::from_str::<Message>(xml).is_err());
    }
}

use crate::*;

/// Top-level XML element.
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct Protocol {
    /// The protocol name should be globally unique. Protocols to be included in wayland-protocols
    /// must follow the naming rules set there. Other protocols should use a unique prefix for
    /// the name, e.g. referring to the owning project’s name.
    #[serde(rename = "@name")]
    pub name: Cname,
    /// Used to indicate the copyrights and the license of the XML file.
    ///
    /// Contains free-form, pre-formatted text for copyright and license notices.
    pub copyright: Option<String>,
    /// The top-level protocol description.
    ///
    /// The description element should be used to document the intended purpose of the protocol,
    /// give an overview, and give any development stage notices if applicable.
    pub description: Option<Description>,
    /// List of interfaces provided by the protocol.
    #[serde(rename = "interface")]
    pub interfaces: Vec<Interface>,
}

impl Protocol {
    /// Shorthand for invoking `quick-xml` to avoid the need to explicitly add it as a project
    /// dependency.
    pub fn from_xml(str: &str) -> Result<Protocol, quick_xml::de::DeError> {
        quick_xml::de::from_str(str)
    }

    /// Construct a new `Protocol` with all fields but `name` set to their unset default.
    pub fn new(name: Cname) -> Self {
        Self {
            name,
            copyright: Default::default(),
            description: Default::default(),
            interfaces: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use super::*;

    #[test]
    fn empty() {
        let xml = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <protocol name="foo">
                <interface name="bar" version="2">
                </interface>
            </protocol>
        "#;

        let actual = quick_xml::de::from_str(xml).unwrap();

        let mut expected = Protocol::new(Cname("foo".to_string()));
        expected.interfaces = vec![Interface::new(Cname("bar".to_string()), Version(NonZeroU32::new(2).unwrap()))];

        assert_eq!(expected, actual);
    }
}

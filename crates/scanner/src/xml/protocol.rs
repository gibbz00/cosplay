use crate::*;

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct Protocol {
    #[serde(rename = "@name")]
    pub(crate) name: Cname,
    pub(crate) copyright: Option<String>,
    pub(crate) description: Option<Description>,
    #[serde(rename = "interface")]
    pub(crate) interfaces: Vec<Interface>,
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

        let expected = Protocol {
            name: Cname("foo".to_string()),
            copyright: None,
            description: None,
            interfaces: vec![Interface {
                name: Cname("bar".to_string()),
                version: Version(NonZeroU32::new(2).unwrap()),
                frozen: false,
                description: None,
                requests: Default::default(),
                events: Default::default(),
                enums: Default::default(),
            }],
        };

        assert_eq!(expected, actual);
    }
}

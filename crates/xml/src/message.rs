use serde::{Deserialize, de::Unexpected};

use crate::*;

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct Message {
    #[serde(rename = "@name")]
    pub(crate) name: Cname,
    #[serde(flatten, deserialize_with = "is_destructor")]
    pub(crate) destructor: bool,
    #[serde(rename = "@since", default)]
    pub(crate) since: Version,
    #[serde(rename = "@deprecated-since")]
    pub(crate) deprecated_since: Option<Version>,
    pub(crate) description: Option<Description>,
    #[serde(rename = "arg", default)]
    pub(crate) args: Vec<Argument>,
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
            args: Vec::new(),
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

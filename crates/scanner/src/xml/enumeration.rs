use serde::Deserialize;

use crate::*;

#[derive(Deserialize)]
pub struct Enum {
    #[serde(rename = "@name")]
    pub(crate) name: CnameSuffix,
    #[serde(rename = "@bitfield", default)]
    pub(crate) bitfield: bool,
    #[serde(rename = "@since", default)]
    pub(crate) since: Version,
    pub(crate) description: Option<Description>,
    #[serde(rename = "entry", default)]
    pub(crate) entries: Vec<Entry>,
}

#[derive(Deserialize)]
pub struct Entry {
    #[serde(rename = "@name")]
    pub(crate) name: CnameSuffix,
    #[serde(rename = "@value", deserialize_with = "entry_value")]
    pub(crate) value: usize,
    #[serde(flatten, deserialize_with = "Description::deserialize_flattened")]
    pub(crate) description: Description,
    #[serde(rename = "@since", default)]
    pub(crate) since: Version,
    #[serde(rename = "@deprecated-since")]
    pub(crate) deprecated_since: Option<Version>,
}

/// The Message_XML documentation states that decimal, octal, and hex representations are possible,
/// but doesn't say how :/
///
/// `libwayland` [seems] to expect the value string to as valid C. So we assume that the
/// representation based on how integer literals are written there.
///
/// [seems]: https://gitlab.freedesktop.org/wayland/wayland/-/blob/main/src/scanner.c#L1431-1435
fn entry_value<'de, D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<usize, D::Error> {
    let string = String::deserialize(deserializer)?;

    let parse_result = if let Some(hex_str) = string.strip_prefix("0x") {
        usize::from_str_radix(hex_str, 16)
    } else if string.len() > 1
        && let Some(octal_str) = string.strip_prefix('0')
    {
        usize::from_str_radix(octal_str, 8)
    } else {
        string.parse()
    };

    parse_result.map_err(serde::de::Error::custom)
}

#[derive(Debug, PartialEq)]
pub struct EnumPath {
    pub(crate) interface: Option<Cname>,
    pub(crate) enumeration: CnameSuffix,
}

impl<'de> serde::de::Deserialize<'de> for EnumPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut enumeration_string = String::deserialize(deserializer)?;

        let mut interface = None;

        if let Some((interface_str, enumeration_str)) = enumeration_string.split_once('.') {
            interface = Cname::parse(interface_str.to_string())
                .map(Some)
                .map_err(serde::de::Error::custom)?;

            enumeration_string = enumeration_str.to_string();
        }

        let enumeration = CnameSuffix::parse(enumeration_string).map_err(serde::de::Error::custom)?;

        Ok(EnumPath { interface, enumeration })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_repr() {
        let xml = r#"
            <enum name="foo">
                <entry name="first" value="0" />
                <entry name="second" value="100" />
                <entry name="third" value="0100" />
                <entry name="forth" value="0x100" />
            </enum>
        "#;

        let actual = quick_xml::de::from_str::<Enum>(xml)
            .unwrap()
            .entries
            .iter()
            .map(|entry| entry.value)
            .collect::<Vec<_>>();

        let expected = [0usize, 100, 0o100, 0x100];

        assert_eq!(expected.as_slice(), actual);
    }

    #[derive(Deserialize)]
    struct Wrapper {
        #[serde(rename = "@enum")]
        path: EnumPath,
    }

    fn assert_enum_path(xml: &str, expected: EnumPath) {
        let actual = quick_xml::de::from_str::<Wrapper>(xml).unwrap().path;
        assert_eq!(expected, actual)
    }

    #[test]
    fn local_enum_path() {
        assert_enum_path(
            "<wrapper enum=\"bar\" />",
            EnumPath { interface: None, enumeration: CnameSuffix("bar".to_string()) },
        );

        assert_enum_path(
            "<wrapper enum=\"foo.bar\" />",
            EnumPath {
                interface: Some(Cname("foo".to_string())),
                enumeration: CnameSuffix("bar".to_string()),
            },
        );
    }

    #[test]
    fn qualified_enum_path() {
        assert_enum_path(
            "<wrapper enum=\"foo.bar\" />",
            EnumPath {
                interface: Some(Cname("foo".to_string())),
                enumeration: CnameSuffix("bar".to_string()),
            },
        );
    }
}

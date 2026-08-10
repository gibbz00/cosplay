use serde::Deserialize;

use crate::*;

/// Description for integers as (unit-only) enums or bitflags.
#[derive(Debug, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct Enum {
    /// The name must be unique within all enumerations in the containing interface. The name is
    /// then used as the namespace for all the contained [`Entry`] elements.
    #[serde(rename = "@name")]
    pub name: CnameSuffix,
    #[serde(rename = "@bitfield", default)]
    pub bitfield: bool,
    #[serde(rename = "@since", default)]
    pub since: Version,
    pub description: Option<Description>,
    #[serde(rename = "entry", default)]
    pub entries: Vec<EnumEntry>,
}

/// [`Enum`] variant name, description, and value.
#[derive(Debug, PartialEq, Deserialize)]
#[allow(missing_docs)]
pub struct EnumEntry {
    /// The name must be unique within all entry elements in the containing enum.
    #[serde(rename = "@name")]
    pub name: CnameSuffix,
    /// The value can be given in decimal, hexadecimal, or octal representation.
    ///
    /// Stored as an i64 to accommodate for both u32 and i32 representations.
    ///
    /// ### Extra - String Representation
    ///
    /// The upstream documentation does say how different representations are differentiated, but
    /// `libwayland` [seems] to expect the string value to treatable as valid C, so it is assumed
    /// that the representation based on how integer literals are written there.
    ///
    /// [seems]: https://gitlab.freedesktop.org/wayland/wayland/-/blob/main/src/scanner.c#L1431-1435
    #[serde(rename = "@value", deserialize_with = "entry_value")]
    pub value: i64,
    #[serde(flatten, deserialize_with = "Description::deserialize_flattened")]
    pub description: Description,
    #[serde(rename = "@since", default)]
    pub since: Version,
    #[serde(rename = "@deprecated-since")]
    pub deprecated_since: Option<Version>,
}

fn entry_value<'de, D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
    let string = String::deserialize(deserializer)?;

    let negative = string.starts_with('-');

    let num_str = match negative {
        true => &string[1..],
        false => &string,
    };

    // NB: avoid deserializing directly to isize to error on invalid input such as "0x-10".
    let parse_result = if let Some(hex_str) = num_str.strip_prefix("0x") {
        i64::from_str_radix(hex_str, 16)
    } else if num_str.len() > 1
        && let Some(octal_str) = num_str.strip_prefix('0')
    {
        i64::from_str_radix(octal_str, 8)
    } else {
        num_str.parse()
    };

    let num = parse_result.map_err(serde::de::Error::custom)?;

    let signed_num = match negative {
        true => -num,
        false => num,
    };

    Ok(signed_num)
}

/// Used by [`ArgumentVariant`]s to reference [`Enum`]s.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct EnumPath {
    /// Used to refer to an enumeration from another [`Interface`]. Assumed otherwise to refer to an
    /// enum within the same interface as the corresponding [`Message`] in which the
    /// [`ArgumentVariant`] is used.
    pub interface: Option<Cname>,
    /// Name of the enum the path points to.
    pub enumeration: CnameSuffix,
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

        let actual = deserialize_values(xml);

        let expected = [0, 100, 0o100, 0x100];

        assert_eq!(expected.as_slice(), actual);
    }

    #[test]
    fn negative_value_repr() {
        let xml = r#"
            <enum name="foo">
                <entry name="first" value="-0" />
                <entry name="second" value="-100" />
                <entry name="third" value="-0100" />
                <entry name="forth" value="-0x100" />
            </enum>
        "#;

        let actual = deserialize_values(xml);

        let expected = [-0, -100, -0o100, -0x100];

        assert_eq!(expected.as_slice(), actual);
    }

    fn deserialize_values(xml: &str) -> Vec<i64> {
        quick_xml::de::from_str::<Enum>(xml)
            .unwrap()
            .entries
            .iter()
            .map(|entry| entry.value)
            .collect()
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

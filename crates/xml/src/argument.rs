use serde::Deserialize;

use crate::*;

/// Argument declaration of [`Message::arguments`].
#[allow(missing_docs)]
#[derive(Debug, PartialEq, Deserialize)]
pub struct Argument {
    /// The name must be unique within all the arguments of the parent element.
    #[serde(rename = "@name")]
    pub name: Cname,

    #[serde(flatten, deserialize_with = "argument_type")]
    pub variant: ArgumentVariant,

    #[serde(flatten, deserialize_with = "Description::deserialize_flattened")]
    pub description: Description,
}

/// Argument type as part of [`Argument`].
///
/// Describe both the marshalling to and from the byte-wire format, but also the higher level
/// interpretations such as interface parameter and enum / bitflag interpretations.
#[allow(missing_docs)]
#[derive(Debug, PartialEq)]
pub enum ArgumentVariant {
    I32 { enumeration: Option<EnumPath> },
    U32 { enumeration: Option<EnumPath> },
    Fixed,
    String { nullable: bool },
    ObjectId { concrete: Option<Cname>, nullable: bool },
    NewObjectId { concrete: Option<Cname> },
    Array,
    Fd,
}

#[derive(Deserialize)]
struct VariantProxy {
    #[serde(rename = "@type")]
    ty: Arg,
    #[serde(rename = "@interface")]
    interface: Option<Cname>,
    #[serde(rename = "@allow-null", default)]
    nullable: Option<String>,
    #[serde(rename = "@enum")]
    enumeration: Option<EnumPath>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Arg {
    Int,
    Uint,
    Fixed,
    String,
    Object,
    NewId,
    Array,
    Fd,
}

fn argument_type<'de, D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<ArgumentVariant, D::Error> {
    let VariantProxy { ty, interface, nullable, enumeration } = VariantProxy::deserialize(deserializer)?;

    let nullable = match nullable.as_deref().unwrap_or("false") {
        "true" => true,
        "false" => false,
        other => return Err(serde::de::Error::invalid_value(serde::de::Unexpected::Str(other), &"true or false")),
    };

    let ty = match (ty, interface, nullable, enumeration) {
        (Arg::Int, None, false, enumeration) => ArgumentVariant::I32 { enumeration },
        (Arg::Uint, None, false, enumeration) => ArgumentVariant::U32 { enumeration },
        (Arg::String, None, nullable, None) => ArgumentVariant::String { nullable },
        (Arg::Object, concrete, nullable, None) => ArgumentVariant::ObjectId { concrete, nullable },
        (Arg::NewId, concrete, false, None) => ArgumentVariant::NewObjectId { concrete },
        (Arg::Fixed, None, false, None) => ArgumentVariant::Fixed,
        (Arg::Array, None, false, None) => ArgumentVariant::Array,
        (Arg::Fd, None, false, None) => ArgumentVariant::Fd,
        (ty, _, _, Some(_)) => {
            return Err(serde::de::Error::custom(format!(
                "The enum attribute is not allowed in combination with the {ty:?} type attribute."
            )));
        }
        (ty, Some(interface), _, _) => {
            return Err(serde::de::Error::custom(format!(
                "The interface attribute ('{interface}') is not allowed in combination with the {ty:?} type attribute."
            )));
        }
        (ty, _, true, _) => {
            return Err(serde::de::Error::custom(format!(
                "The allow-null attribute is not allowed in combination with the {ty:?} type attributes."
            )));
        }
    };

    Ok(ty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_interface_ok() {
        assert_arg_ok(
            r#"type="object" interface="bar""#,
            ArgumentVariant::ObjectId { concrete: Some(Cname("bar".to_string())), nullable: false },
        );
    }

    #[test]
    fn new_object_interface_ok() {
        assert_arg_ok(
            r#"type="new_id" interface="bar""#,
            ArgumentVariant::NewObjectId { concrete: Some(Cname("bar".to_string())) },
        );
    }

    #[test]
    fn other_interface_err() {
        assert_arg_err(r#"type="int" interface="bar""#);
    }

    #[test]
    fn nullable_str_ok() {
        assert_arg_ok(r#"type="string" allow-null="true""#, ArgumentVariant::String { nullable: true });
        assert_arg_ok(r#"type="string" allow-null="false""#, ArgumentVariant::String { nullable: false });
        assert_arg_ok(r#"type="string""#, ArgumentVariant::String { nullable: false });
    }

    #[test]
    fn nullable_object_ok() {
        assert_arg_ok(
            r#"type="object" allow-null="true""#,
            ArgumentVariant::ObjectId { concrete: None, nullable: true },
        );
    }

    #[test]
    fn other_nullable_err() {
        assert_arg_err(r#"type="int" allow-null="true""#);
    }

    #[test]
    fn uint_enumeration_ok() {
        assert_arg_ok(
            r#"type="uint" enum="bar""#,
            ArgumentVariant::U32 {
                enumeration: Some(EnumPath { interface: None, enumeration: CnameSuffix("bar".to_string()) }),
            },
        );
    }

    #[test]
    fn int_enumeration_ok() {
        assert_arg_ok(
            r#"type="int" enum="baz""#,
            ArgumentVariant::I32 {
                enumeration: Some(EnumPath { interface: None, enumeration: CnameSuffix("baz".to_string()) }),
            },
        );
    }

    #[test]
    fn other_enumeration_err() {
        assert_arg_err(r#"type="string" enum="bar""#);
    }

    #[derive(Deserialize)]
    struct Wrapper {
        #[serde(rename = "arg")]
        arg: Argument,
    }

    fn assert_arg_ok(attribute_str: &str, variant: ArgumentVariant) {
        let actual = prepare_deserialize(attribute_str).unwrap().arg;

        let expected = Argument {
            name: Cname("foo".to_string()),
            variant,
            description: Description { summary: None, text: None },
        };

        assert_eq!(expected, actual);
    }

    fn assert_arg_err(attribute_str: &str) {
        assert!(prepare_deserialize(attribute_str).is_err())
    }

    fn prepare_deserialize(attribute_str: &str) -> Result<Wrapper, quick_xml::de::DeError> {
        let xml = format!("<box><arg name=\"foo\" {attribute_str}/></box>");
        quick_xml::de::from_str(&xml)
    }
}

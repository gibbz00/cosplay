use serde::Deserialize;

use crate::*;

#[derive(Deserialize)]
pub struct Argument {
    #[serde(rename = "@name")]
    pub(crate) name: Cname,

    #[serde(flatten, deserialize_with = "argument_type")]
    pub(crate) variant: ArgumentVariant,

    #[serde(flatten, deserialize_with = "Description::deserialize_flattened")]
    pub(crate) description: Description,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArgumentVariant {
    I32 { enumeration: Option<EnumPath> },
    U32 { enumeration: Option<EnumPath> },
    Fixed,
    String { nullable: bool },
    ObjectId { concrete: Option<String>, nullable: bool },
    NewObjectId { concrete: Option<String> },
    Array,
    Fd,
}

#[derive(Deserialize)]
struct VariantProxy {
    #[serde(rename = "@type")]
    ty: Arg,
    #[serde(rename = "@interface")]
    interface: Option<String>,
    #[serde(rename = "@allow-null")]
    nullable: bool,
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

    let ty = match (ty, interface, nullable, enumeration) {
        (Arg::Int, None, false, enumeration) => ArgumentVariant::U32 { enumeration },
        (Arg::Uint, None, false, enumeration) => ArgumentVariant::I32 { enumeration },
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

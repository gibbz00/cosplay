use serde::{Deserialize, de::Unexpected};

use crate::*;

#[derive(serde::Deserialize)]
pub struct Message {
    #[serde(rename = "@name")]
    pub(crate) name: Cname,
    #[serde(rename = "@type", deserialize_with = "is_destructor")]
    pub(crate) destructor: bool,
    #[serde(rename = "@since", default)]
    pub(crate) since: Version,
    #[serde(rename = "@deprecated-since")]
    pub(crate) deprecated_since: Option<Version>,
    pub(crate) description: Option<Description>,
    #[serde(rename = "arg")]
    pub(crate) args: Vec<Argument>,
}

fn is_destructor<'de, D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    const TYPE_STR: &str = "destructor";

    let Some(string) = Option::<String>::deserialize(deserializer)? else {
        return Ok(false);
    };

    match string.contains(TYPE_STR) {
        true => Ok(true),
        false => Err(serde::de::Error::invalid_value(Unexpected::Str(&string), &TYPE_STR)),
    }
}

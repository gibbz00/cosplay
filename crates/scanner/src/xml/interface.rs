use crate::*;

#[derive(serde::Deserialize)]
pub struct Interface {
    #[serde(rename = "@name")]
    pub(crate) name: Cname,
    #[serde(rename = "@version")]
    pub(crate) version: Version,
    #[serde(rename = "@frozen", default)]
    pub(crate) frozen: bool,
    pub(crate) description: Option<Description>,
    #[serde(rename = "request")]
    pub(crate) requests: Vec<Message>,
    #[serde(rename = "event")]
    pub(crate) events: Vec<Message>,
    #[serde(rename = "enum")]
    pub(crate) enums: Vec<Enum>,
}

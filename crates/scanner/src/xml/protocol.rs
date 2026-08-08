use crate::*;

#[derive(serde::Deserialize)]
pub struct Protocol {
    #[serde(rename = "@name")]
    pub(crate) name: Cname,
    pub(crate) copyright: Option<String>,
    pub(crate) description: Option<Description>,
    #[serde(rename = "interface")]
    pub(crate) interfaces: Vec<Interface>,
}

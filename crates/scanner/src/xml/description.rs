use serde::Deserialize;

#[derive(Deserialize)]
pub struct Description {
    #[serde(rename = "@summary")]
    pub summary: Option<String>,
    #[serde(rename = "$text")]
    pub text: String,
}

impl Description {
    /// For when the summary attribute is part of the parent element itself.
    pub(crate) fn deserialize_flattened<'de, D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Option<Description>, D::Error> {
        #[derive(Deserialize)]
        struct Proxy {
            #[serde(rename = "@summary")]
            summary: Option<String>,
            description: Option<String>,
        }

        let Proxy { summary, description } = Proxy::deserialize(deserializer)?;

        let description = match (summary, description) {
            (None, Some(text)) => Some(Description { summary: None, text }),
            (Some(summary), None) => Some(Description { summary: Some(summary), text: Default::default() }),
            (Some(summary), Some(text)) => Some(Description { summary: Some(summary), text }),
            (None, None) => None,
        };

        Ok(description)
    }
}

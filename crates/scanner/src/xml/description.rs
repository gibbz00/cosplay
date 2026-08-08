use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
pub struct Description {
    #[serde(rename = "@summary")]
    pub summary: Option<String>,
    #[serde(rename = "$text")]
    pub text: Option<String>,
}

impl Description {
    /// For when the summary attribute is part of the parent element itself.
    ///
    /// Probably due to both historical and convenience reasons, the summary attribute can appear
    /// both in the parent element, and on the description itself. This implementation returns the
    /// concatenation of both.
    pub(crate) fn deserialize_flattened<'de, D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Description, D::Error> {
        #[derive(Deserialize)]
        struct Parent {
            #[serde(rename = "@summary")]
            parent_summary: Option<String>,
            #[serde(rename = "description")]
            inner: Option<Inner>,
        }

        #[derive(Deserialize)]
        struct Inner {
            #[serde(rename = "@summary")]
            inner_summary: Option<String>,
            #[serde(rename = "$text")]
            inner_text: Option<String>,
        }

        let Parent { parent_summary, inner } = Parent::deserialize(deserializer)?;

        let mut description = Description { summary: parent_summary, text: None };

        if let Some(Inner { inner_summary, inner_text }) = inner {
            if let Some(inner) = inner_summary {
                description.summary = match description.summary {
                    Some(mut current) => {
                        current.push(' ');
                        current.push_str(&inner);
                        Some(current)
                    }
                    None => Some(inner),
                }
            }

            description.text = inner_text;
        }

        Ok(description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_regular() {}

    #[derive(Deserialize)]
    struct Flatten {
        #[serde(flatten, deserialize_with = "Description::deserialize_flattened")]
        pub(crate) description: Description,
    }

    #[test]
    fn deserialize_flattened_none() {
        assert_flattened("<top><blah /></top>", Description { summary: None, text: None });
    }

    #[test]
    fn deserialize_flattened_parent_summary() {
        let xml = "<top summary=\"abc\"><blah /></top>";
        assert_flattened(xml, Description { summary: Some("abc".to_string()), text: None });
    }

    #[test]
    fn deserialize_flattened_inner_summary() {
        assert_flattened(
            "<top><description summary=\"def\" /></top>",
            Description { summary: Some("def".to_string()), text: None },
        );
    }

    #[test]
    fn deserialize_flattened_concat_summary() {
        assert_flattened(
            "<top summary=\"abc\"><description summary=\"def\" /></top>",
            Description { summary: Some("abc def".to_string()), text: None },
        );
    }

    #[test]
    fn deserialize_flattened_inner_description() {
        assert_flattened(
            "<top><description>Hello</description></top>",
            Description { summary: None, text: Some("Hello".to_string()) },
        );
    }

    fn assert_flattened(xml: &str, expected: Description) {
        let actual = quick_xml::de::from_str::<Flatten>(xml).unwrap().description;
        assert_eq!(expected, actual);
    }
}

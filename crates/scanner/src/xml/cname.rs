//! Shared character restrictions declared item names.

const fn valid_first_char(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

const fn valid_suffix_char(ch: char) -> bool {
    valid_first_char(ch) || ch.is_ascii_digit()
}

const EMPTY_MSG: &str = "The name can not be empty.";
const INVALID_FIRST_MSG: &str = "Invalid first character. Only a-z, A-Z, and underscores are allowed.";
const INVALID_SUFFIX_MSG: &str = "Invalid suffix character. Only ASCII alphanumberics and underscores are allowed.";

/// The name must start with one of the ASCII characters a-z, A-Z, or underscore, and the
/// following characters may additionally include numbers 0-9.
pub struct Cname(pub(crate) String);

#[derive(Debug, thiserror::Error)]
pub(crate) enum CnameParseError {
    #[error("{EMPTY_MSG}")]
    Empty,
    #[error("{INVALID_FIRST_MSG}")]
    InvalidFirst,
    #[error("{INVALID_SUFFIX_MSG}")]
    InvalidSuffix,
}

impl Cname {
    pub(crate) fn parse(string: String) -> Result<Self, CnameParseError> {
        let mut chars = string.chars();

        let first = chars.next().ok_or(CnameParseError::Empty)?;
        valid_first_char(first).ok_or(CnameParseError::InvalidFirst)?;

        for suffix_char in chars {
            valid_suffix_char(suffix_char).ok_or(CnameParseError::InvalidSuffix)?;
        }

        Ok(Cname(string))
    }
}

impl<'de> serde::de::Deserialize<'de> for Cname {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Self::parse(string).map_err(serde::de::Error::custom)
    }
}

/// The name must contain only the ASCII characters a-z, A-Z, 0-9, or underscore.
pub struct CnameSuffix(pub(crate) String);

#[derive(Debug, thiserror::Error)]
pub(crate) enum CnameSuffixParseError {
    #[error("{EMPTY_MSG}")]
    Empty,
    #[error("{INVALID_SUFFIX_MSG}")]
    InvalidSuffix,
}

impl CnameSuffix {
    pub(crate) fn parse(string: String) -> Result<Self, CnameSuffixParseError> {
        if string.is_empty() {
            return Err(CnameSuffixParseError::Empty);
        }

        for suffix_char in string.chars() {
            valid_suffix_char(suffix_char).ok_or(CnameSuffixParseError::InvalidSuffix)?;
        }

        Ok(CnameSuffix(string))
    }
}

impl<'de> serde::de::Deserialize<'de> for CnameSuffix {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Self::parse(string).map_err(serde::de::Error::custom)
    }
}

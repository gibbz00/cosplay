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
#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
pub struct CnameSuffix(pub(crate) String);

#[derive(Debug, thiserror::Error)]
pub(crate) enum CnameSuffixParseError {
    #[error("{EMPTY_MSG}")]
    Empty,
    #[error("{INVALID_SUFFIX_MSG}")]
    InvalidChar,
}

impl CnameSuffix {
    pub(crate) fn parse(string: String) -> Result<Self, CnameSuffixParseError> {
        if string.is_empty() {
            return Err(CnameSuffixParseError::Empty);
        }

        for suffix_char in string.chars() {
            valid_suffix_char(suffix_char).ok_or(CnameSuffixParseError::InvalidChar)?;
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

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use super::*;

    #[test]
    fn name_empty_error() {
        let error = Cname::parse(String::new()).unwrap_err();
        assert_matches!(error, CnameParseError::Empty);
    }

    #[test]
    fn name_invalid_first_error() {
        assert_error("0");
        assert_error("🦀");

        fn assert_error(str: &str) {
            let error = Cname::parse(str.to_string()).unwrap_err();
            assert_matches!(error, CnameParseError::InvalidFirst);
        }
    }

    #[test]
    fn name_invalid_suffix_error() {
        assert_error("a.");
        assert_error("a🦀");

        fn assert_error(str: &str) {
            let error = Cname::parse(str.to_string()).unwrap_err();
            assert_matches!(error, CnameParseError::InvalidSuffix);
        }
    }

    #[test]
    fn name_ok() {
        assert_ok("_a");
        assert_ok("Ab");
        assert_ok("b0");

        fn assert_ok(str: &str) {
            let string = str.to_string();
            let cname = Cname::parse(string.clone()).unwrap();
            assert_eq!(string, cname.0)
        }
    }

    #[test]
    fn suffix_name_empty_error() {
        let error = CnameSuffix::parse(String::new()).unwrap_err();
        assert_matches!(error, CnameSuffixParseError::Empty);
    }

    #[test]
    fn suffix_name_invalid_char_error() {
        assert_error(".");
        assert_error("-");
        assert_error("🦀");

        fn assert_error(str: &str) {
            let error = CnameSuffix::parse(str.to_string()).unwrap_err();
            assert_matches!(error, CnameSuffixParseError::InvalidChar);
        }
    }

    #[test]
    fn suffix_name_ok() {
        assert_ok("_a");
        assert_ok("Ab");
        assert_ok("b0");
        assert_ok("0_");

        fn assert_ok(str: &str) {
            let string = str.to_string();
            let suffix = CnameSuffix::parse(string.clone()).unwrap();
            assert_eq!(string, suffix.0)
        }
    }
}

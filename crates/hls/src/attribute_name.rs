use std::{fmt::Display, str::FromStr};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct AttributeName(String);

#[derive(thiserror::Error, Debug)]
#[error("invalid attribute name")]
pub struct InvalidAttributeNameError;

impl TryFrom<String> for AttributeName {
    type Error = InvalidAttributeNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if !value
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(InvalidAttributeNameError);
        }

        Ok(Self(value))
    }
}

impl FromStr for AttributeName {
    type Err = InvalidAttributeNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s.to_string())
    }
}

impl Display for AttributeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#![allow(dead_code)]

use clap::ValueEnum;
use std::fmt;
use std::str::FromStr;

/// Represents the temporal type of a field, such as date, time, or timestamp.
#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum JavaFieldTemporal {
    #[value(name = "date")]
    Date,

    #[value(name = "time")]
    Time,

    #[value(name = "timestamp")]
    Timestamp,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemporalFromStringError {
    pub value: String,
}

impl fmt::Display for TemporalFromStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No matching enum member for value '{}'", self.value)
    }
}

impl std::error::Error for TemporalFromStringError {}

impl JavaFieldTemporal {
    /// Gets the string value of the enum constant.
    pub fn as_str(&self) -> &'static str {
        match self {
            JavaFieldTemporal::Date => "DATE",
            JavaFieldTemporal::Time => "TIME",
            JavaFieldTemporal::Timestamp => "TIMESTAMP",
        }
    }
}

impl TryFrom<&str> for JavaFieldTemporal {
    type Error = TemporalFromStringError;

    /// Finds a JavaFieldTemporal constant that matches the given string value.
    ///
    /// This lookup is case-sensitive.
    ///
    /// # Arguments
    /// * `value` - The string value to find.
    ///
    /// # Returns
    /// The corresponding JavaFieldTemporal enum constant.
    ///
    /// # Errors
    /// Returns `TemporalFromStringError` if no matching constant is found for the given value.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "date" => Ok(JavaFieldTemporal::Date),
            "time" => Ok(JavaFieldTemporal::Time),
            "timestamp" => Ok(JavaFieldTemporal::Timestamp),
            _ => Err(TemporalFromStringError { value: value.to_string() }),
        }
    }
}

impl FromStr for JavaFieldTemporal {
    type Err = TemporalFromStringError;

    /// Finds a JavaFieldTemporal constant that matches the given string value.
    ///
    /// This lookup is case-sensitive.
    ///
    /// # Arguments
    /// * `s` - The string value to parse.
    ///
    /// # Returns
    /// The corresponding JavaFieldTemporal enum constant.
    ///
    /// # Errors
    /// Returns `TemporalFromStringError` if no matching constant is found for the given value.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

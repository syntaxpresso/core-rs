#![allow(dead_code)]

use clap::ValueEnum;
use std::fmt;
use std::str::FromStr;

/// Defines strategies for handling time zone information in date-time fields.
#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum JavaFieldTimeZoneStorage {
    #[value(name = "native")]
    Native,

    #[value(name = "normalize")]
    Normalize,

    #[value(name = "normalize_utc")]
    NormalizeUtc,

    #[value(name = "column")]
    Column,

    #[value(name = "auto")]
    Auto,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeZoneStorageFromStringError {
    pub value: String,
}

impl fmt::Display for TimeZoneStorageFromStringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No matching enum member for value '{}'", self.value)
    }
}

impl std::error::Error for TimeZoneStorageFromStringError {}

impl JavaFieldTimeZoneStorage {
    /// Gets the string value associated with the enum constant.
    pub fn as_str(&self) -> &'static str {
        match self {
            JavaFieldTimeZoneStorage::Native => "native",
            JavaFieldTimeZoneStorage::Normalize => "normalize",
            JavaFieldTimeZoneStorage::NormalizeUtc => "normalize_utc",
            JavaFieldTimeZoneStorage::Column => "column",
            JavaFieldTimeZoneStorage::Auto => "auto",
        }
    }
}

impl TryFrom<&str> for JavaFieldTimeZoneStorage {
    type Error = TimeZoneStorageFromStringError;

    /// Finds a JavaFieldTimeZoneStorage constant that matches the given string value.
    ///
    /// This lookup is case-sensitive.
    ///
    /// # Arguments
    /// * `value` - The string value to find.
    ///
    /// # Returns
    /// The corresponding JavaFieldTimeZoneStorage enum constant.
    ///
    /// # Errors
    /// Returns `TimeZoneStorageFromStringError` if no matching constant is found for the given value.
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "native" => Ok(JavaFieldTimeZoneStorage::Native),
            "normalize" => Ok(JavaFieldTimeZoneStorage::Normalize),
            "normalize_utc" => Ok(JavaFieldTimeZoneStorage::NormalizeUtc),
            "column" => Ok(JavaFieldTimeZoneStorage::Column),
            "auto" => Ok(JavaFieldTimeZoneStorage::Auto),
            _ => Err(TimeZoneStorageFromStringError {
                value: value.to_string(),
            }),
        }
    }
}

impl FromStr for JavaFieldTimeZoneStorage {
    type Err = TimeZoneStorageFromStringError;

    /// Finds a JavaFieldTimeZoneStorage constant that matches the given string value.
    ///
    /// This lookup is case-sensitive.
    ///
    /// # Arguments
    /// * `s` - The string value to parse.
    ///
    /// # Returns
    /// The corresponding JavaFieldTimeZoneStorage enum constant.
    ///
    /// # Errors
    /// Returns `TimeZoneStorageFromStringError` if no matching constant is found for the given value.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}

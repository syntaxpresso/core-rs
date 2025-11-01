use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum CascadeType {
  #[value(name = "all")]
  All,
  #[value(name = "persist")]
  Persist,
  #[value(name = "merge")]
  Merge,
  #[value(name = "remove")]
  Remove,
  #[value(name = "refresh")]
  Refresh,
  #[value(name = "detach")]
  Detach,
}

impl CascadeType {
  pub fn from_value(value: &str) -> Result<Self, String> {
    match value {
      "all" => Ok(CascadeType::All),
      "persist" => Ok(CascadeType::Persist),
      "merge" => Ok(CascadeType::Merge),
      "remove" => Ok(CascadeType::Remove),
      "refresh" => Ok(CascadeType::Refresh),
      "detach" => Ok(CascadeType::Detach),
      _ => Err(format!("No matching enum member for value '{}'", value)),
    }
  }

  pub fn as_str(&self) -> &'static str {
    match self {
      CascadeType::All => "ALL",
      CascadeType::Persist => "PERSIST",
      CascadeType::Merge => "MERGE",
      CascadeType::Remove => "REMOVE",
      CascadeType::Refresh => "REFRESH",
      CascadeType::Detach => "DETACH",
    }
  }
}

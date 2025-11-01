use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum FetchType {
  #[value(name = "lazy")]
  Lazy,
  #[value(name = "eager")]
  Eager,
  #[value(name = "none")]
  None,
}

impl FetchType {
  pub fn from_value(value: &str) -> Result<Self, String> {
    match value {
      "lazy" => Ok(FetchType::Lazy),
      "eager" => Ok(FetchType::Eager),
      "none" => Ok(FetchType::None),
      _ => Err(format!("No matching enum member for value '{}'", value)),
    }
  }

  pub fn as_str(&self) -> &'static str {
    match self {
      FetchType::Lazy => "LAZY",
      FetchType::Eager => "EAGER",
      FetchType::None => "NONE",
    }
  }
}

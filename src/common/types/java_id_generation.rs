use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum JavaIdGeneration {
  #[value(name = "none")]
  None,
  #[value(name = "auto")]
  Auto,
  #[value(name = "identity")]
  Identity,
  #[value(name = "sequence")]
  Sequence,
  #[value(name = "uuid")]
  Uuid,
}

impl JavaIdGeneration {
  pub fn from_value(value: &str) -> Result<Self, String> {
    match value {
      "none" => Ok(JavaIdGeneration::None),
      "auto" => Ok(JavaIdGeneration::Auto),
      "identity" => Ok(JavaIdGeneration::Identity),
      "sequence" => Ok(JavaIdGeneration::Sequence),
      "uuid" => Ok(JavaIdGeneration::Uuid),
      _ => Err(format!("No matching enum member for value '{}'", value)),
    }
  }
  pub fn as_str(&self) -> &'static str {
    match self {
      JavaIdGeneration::None => "NONE",
      JavaIdGeneration::Auto => "AUTO",
      JavaIdGeneration::Identity => "IDENTITY",
      JavaIdGeneration::Sequence => "SEQUENCE",
      JavaIdGeneration::Uuid => "UUID",
    }
  }
}

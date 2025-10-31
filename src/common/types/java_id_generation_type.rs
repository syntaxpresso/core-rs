use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum JavaIdGenerationType {
  #[value(name = "none")]
  None,
  #[value(name = "orm_provided")]
  OrmProvided,
  #[value(name = "entity_exclusive_generation")]
  EntityExclusiveGeneration,
}

impl JavaIdGenerationType {
  pub fn from_value(value: &str) -> Result<Self, String> {
    match value {
      "none" => Ok(JavaIdGenerationType::None),
      "orm_provided" => Ok(JavaIdGenerationType::OrmProvided),
      "entity_exclusive_generation" => Ok(JavaIdGenerationType::EntityExclusiveGeneration),
      _ => Err(format!("No matching enum member for value '{}'", value)),
    }
  }
}

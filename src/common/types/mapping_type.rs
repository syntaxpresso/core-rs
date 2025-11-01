use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum MappingType {
  #[value(name = "unidirectional_join_column")]
  UnidirectionalJoinColumn,
  #[value(name = "bidirectional_join_column")]
  BidirectionalJoinColumn,
}

impl MappingType {
  pub fn from_value(value: &str) -> Result<Self, String> {
    match value {
      "unidirectional_join_column" => Ok(MappingType::UnidirectionalJoinColumn),
      "bidirectional_join_column" => Ok(MappingType::BidirectionalJoinColumn),
      _ => Err(format!("No matching enum member for value '{}'", value)),
    }
  }
}

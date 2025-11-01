use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum OtherType {
  #[value(name = "mandatory")]
  Mandatory,
  #[value(name = "unique")]
  Unique,
  #[value(name = "orphan_removal")]
  OrphanRemoval,
  #[value(name = "large_object")]
  LargeObject,
  #[value(name = "equals_hashcode")]
  EqualsHashcode,
  #[value(name = "mutable")]
  Mutable,
}

impl OtherType {
  pub fn from_value(value: &str) -> Result<Self, String> {
    match value {
      "mandatory" => Ok(OtherType::Mandatory),
      "unique" => Ok(OtherType::Unique),
      "orphan_removal" => Ok(OtherType::OrphanRemoval),
      "large_object" => Ok(OtherType::LargeObject),
      "equals_hashcode" => Ok(OtherType::EqualsHashcode),
      "mutable" => Ok(OtherType::Mutable),
      _ => Err(format!("No matching enum member for value '{}'", value)),
    }
  }
}

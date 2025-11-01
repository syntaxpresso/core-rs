use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum CollectionType {
  #[value(name = "set")]
  Set,
  #[value(name = "list")]
  List,
  #[value(name = "collection")]
  Collection,
}

impl CollectionType {
  pub fn from_value(value: &str) -> Result<Self, String> {
    match value {
      "set" => Ok(CollectionType::Set),
      "list" => Ok(CollectionType::List),
      "collection" => Ok(CollectionType::Collection),
      _ => Err(format!("No matching enum member for value '{}'", value)),
    }
  }

  pub fn as_java_type(&self) -> &'static str {
    match self {
      CollectionType::Set => "Set",
      CollectionType::List => "List",
      CollectionType::Collection => "Collection",
    }
  }

  pub fn as_java_import(&self) -> &'static str {
    match self {
      CollectionType::Set => "java.util.Set",
      CollectionType::List => "java.util.List",
      CollectionType::Collection => "java.util.Collection",
    }
  }
}

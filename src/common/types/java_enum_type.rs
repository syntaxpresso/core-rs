#![allow(dead_code)]

use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum JavaEnumType {
  #[value(name = "ordinal")]
  Ordinal,
  #[value(name = "string")]
  String,
}

impl JavaEnumType {
  pub fn as_str(&self) -> &'static str {
    match self {
      JavaEnumType::Ordinal => "ORDINAL",
      JavaEnumType::String => "STRING",
    }
  }
}
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct JavaBasicTypeResponse {
  pub id: String,
  pub name: String,
  pub package_path: Option<String>,
}

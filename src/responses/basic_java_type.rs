use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BasicJavaType {
  pub id: String,
  pub name: String,
  pub package_path: String,
  #[serde(rename = "type")]
  pub type_: String,
}

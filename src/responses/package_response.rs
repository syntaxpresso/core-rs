use serde::Serialize;

#[derive(Serialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PackageResponse {
  pub package_name: String,
}

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileResponse {
  pub file_type: String,
  pub file_package_name: String,
  pub file_path: String,
}

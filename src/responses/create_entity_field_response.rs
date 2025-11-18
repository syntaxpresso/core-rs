use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEntityFieldResponse {
  pub entity_file_path: String,
  pub field_name: String,
  pub field_type: String,
}

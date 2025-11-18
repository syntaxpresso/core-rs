use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateManyToOneRelationshipResponse {
  pub success: bool,
  pub message: String,
  pub owning_side_entity_path: String,
  pub inverse_side_entity_path: Option<String>,
  pub owning_side_updated: bool,
  pub inverse_side_updated: bool,
}

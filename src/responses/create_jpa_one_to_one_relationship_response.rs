use serde::Serialize;

#[derive(Serialize)]
pub struct CreateJPAOneToOneRelationshipResponse {
  pub success: bool,
  pub message: String,
  pub owning_side_entity_updated: bool,
  pub inverse_side_entity_updated: bool,
  pub owning_side_entity_path: Option<String>,
  pub inverse_side_entity_path: Option<String>,
}

impl CreateJPAOneToOneRelationshipResponse {
  pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(self)
  }
}

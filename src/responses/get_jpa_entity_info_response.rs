#![allow(dead_code)]

use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetJpaEntityInfoResponse {
    pub is_jpa_entity: bool,
    pub entity_type: String,
    pub entity_package_name: String,
    pub superclass_type: Option<String>,
    pub entity_path: Option<String>,
    pub id_field_type: Option<String>,
    pub id_field_package_name: Option<String>,
}

#[derive(Debug, Clone)]
pub enum IdFieldSearchResult {
    Found { field_type: String, package_name: String },
    NotFound,
    ExternalSuperclass { superclass_name: String },
}

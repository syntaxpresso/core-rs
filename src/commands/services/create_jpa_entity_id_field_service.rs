use crate::common::services::package_declaration_service::{
  get_package_class_scope_node, get_package_declaration_node,
};
use crate::common::ts_file::TSFile;
use crate::common::types::id_field_config::IdFieldConfig;
use crate::responses::file_response::FileResponse;
use std::path::Path;

fn step_parse_entity_file(entity_file_path: &Path) -> Result<TSFile, String> {
  TSFile::from_file(entity_file_path).map_err(|_| "Unable to parse JPA Entity file".to_string())
}

fn step_build_file_response(ts_file: &TSFile) -> Result<FileResponse, String> {
  let file_type = ts_file.get_file_name_without_ext().unwrap_or_default();
  let file_path = ts_file.file_path().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
  let file_package_node = get_package_declaration_node(ts_file)
    .ok_or_else(|| "Unable to get JPA Entity's package node".to_string())?;
  let file_package_scope_node = get_package_class_scope_node(ts_file, file_package_node);
  let file_package_name = file_package_scope_node
    .and_then(|node| ts_file.get_text_from_node(&node))
    .unwrap_or("")
    .to_string();
  Ok(FileResponse { file_type, file_package_name, file_path })
}
pub fn run(
  _cwd: &Path,
  entity_file_path: &Path,
  _field_config: IdFieldConfig,
) -> Result<FileResponse, String> {
  // Step 1: Process field config
  // let processed_field_config = process_field_config(&field_config);

  let entity_ts_file = step_parse_entity_file(entity_file_path)?;

  step_build_file_response(&entity_ts_file)
}

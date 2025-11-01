use std::path::Path;

use crate::{
  commands::services::create_jpa_entity_enum_field_service::run,
  common::types::enum_field_config::EnumFieldConfig,
  responses::{file_response::FileResponse, response::Response},
};

pub fn execute(
  cwd: &Path,
  entity_file_path: &Path,
  field_config: EnumFieldConfig,
) -> Response<FileResponse> {
  let cwd_string = cwd.display().to_string();
  let cmd_name = String::from("create-jpa-entity-enum-field");
  match run(cwd, entity_file_path, field_config) {
    Ok(response) => Response::success(cmd_name, cwd_string, response),
    Err(error_msg) => Response::error(cmd_name, cwd_string, error_msg),
  }
}
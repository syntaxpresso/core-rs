use std::path::Path;

use crate::{
  commands::services::create_jpa_entity_basic_field_service::run,
  common::types::basic_field_config::BasicFieldConfig,
  responses::{file_response::FileResponse, response::Response},
};

pub fn execute(
  cwd: &Path,
  entity_file_b64_src: &str,
  entity_file_path: &Path,
  field_config: &BasicFieldConfig,
) -> Response<FileResponse> {
  let cwd_string = cwd.display().to_string();
  let cmd_name = String::from("create-jpa-entity-basic-field");

  // Note: We don't validate entity_file_path containment within cwd because
  // we're editing an existing file that the user has opened, which may be
  // located anywhere on the filesystem. The path is trusted as it comes from
  // the user's editor context.

  match run(entity_file_b64_src, entity_file_path, field_config) {
    Ok(response) => Response::success(cmd_name, cwd_string, response),
    Err(error_msg) => Response::error(cmd_name, cwd_string, error_msg),
  }
}

use std::path::Path;

use crate::{
  commands::services::get_jpa_entity_info_service::run,
  responses::{get_jpa_entity_info_response::GetJpaEntityInfoResponse, response::Response},
};

pub fn execute(
  cwd: &Path,
  entity_file_path: Option<&Path>,
  b64_source_code: Option<&str>,
) -> Response<GetJpaEntityInfoResponse> {
  let cwd_string = cwd.display().to_string();
  let cmd_name = String::from("get-entity-info");
  match run(entity_file_path, b64_source_code) {
    Ok(response) => Response::success(cmd_name, cwd_string, response),
    Err(error_msg) => Response::error(cmd_name, cwd_string, error_msg),
  }
}

use std::path::Path;

use crate::{
  commands::{
    services::create_jpa_repository_service::{run, run_with_manual_id},
    validators::directory_validator::validate_file_path_within_base,
  },
  responses::{
    create_jpa_repository_response::CreateJPARepositoryResponse, file_response::FileResponse,
    response::Response,
  },
};

pub fn execute(
  cwd: &Path,
  entity_file_b64_src: &str,
  entity_file_path: &Path,
  b64_superclass_source: Option<&str>,
) -> Response<CreateJPARepositoryResponse> {
  let cwd_string = cwd.display().to_string();
  let cmd_name = String::from("create-jpa-repository");
  // Security validation: ensure entity file path is within the cwd
  let file_path_str = entity_file_path.display().to_string();
  if let Err(error_msg) = validate_file_path_within_base(&file_path_str, cwd) {
    return Response::error(
      cmd_name,
      cwd_string,
      format!("Entity file path security validation failed: {}", error_msg),
    );
  }

  match run(cwd, entity_file_b64_src, entity_file_path, b64_superclass_source) {
    Ok(response) => Response::success(cmd_name, cwd_string, response),
    Err(error_msg) => Response::error(cmd_name, cwd_string, error_msg),
  }
}

/// Creates a JPA repository with manually provided ID field information.
/// This is used as a fallback when automatic ID field detection fails.
pub fn execute_with_manual_id(
  cwd: &Path,
  entity_file_b64_src: &str,
  entity_file_path: &Path,
  id_field_type: &str,
  id_field_package_name: &str,
) -> Response<FileResponse> {
  let cwd_string = cwd.display().to_string();
  let cmd_name = String::from("create-jpa-repository-manual");
  // Security validation: ensure entity file path is within the cwd
  let file_path_str = entity_file_path.display().to_string();
  if let Err(error_msg) = validate_file_path_within_base(&file_path_str, cwd) {
    return Response::error(
      cmd_name,
      cwd_string,
      format!("Entity file path security validation failed: {}", error_msg),
    );
  }

  match run_with_manual_id(
    cwd,
    entity_file_b64_src,
    entity_file_path,
    id_field_type,
    id_field_package_name,
  ) {
    Ok(file_response) => Response::success(cmd_name, cwd_string, file_response),
    Err(error_msg) => Response::error(cmd_name, cwd_string, error_msg),
  }
}
